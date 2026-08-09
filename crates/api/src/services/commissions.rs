use uuid::Uuid;
use sqlx::PgPool;

use crate::errors::{ApiError, ApiResult};
use crate::models::mpesa::CommissionLedgerEntry;
use crate::services::wallet;

pub const REGISTRATION_COMMISSION_RATE: f64 = 30.0;  // 30%
pub const RENEWAL_COMMISSION_RATE: f64 = 10.0;       // 10%

// ───────────────────────────────────────────
// Initiate a payment (uses simulation, sends emails)
// ───────────────────────────────────────────
pub async fn initiate_payment(
    pool: &PgPool,
    email_service: &rento_core::email::EmailService,
    mpesa_client: &crate::services::mpesa::MpesaClient,
    payer_id: &Uuid,
    phone: &str,
    amount: u32,
    payment_type: &str,
    reference_id: Option<&str>,
    account_ref: &str,
) -> ApiResult<crate::models::mpesa::PaymentResponse> {
    // Parse reference_id to Uuid internally
    let ref_uuid = reference_id
        .map(|id| Uuid::parse_str(id))
        .transpose()
        .map_err(|e| ApiError::BadRequest(format!("Invalid reference_id: {}", e)))?;

    // 1. Simulate the payment (creates transaction as 'success' immediately)
    let (merchant_request_id, checkout_request_id, receipt_number) =
        mpesa_client.simulate_payment(pool, phone, amount, account_ref).await?;

    // 2. Find the transaction we just created
    let mpesa_tx_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM mpesa_transactions WHERE checkout_request_id = $1"
    )
        .bind(&checkout_request_id)
        .fetch_one(pool)
        .await?;

    // 3. Create payment record (already 'completed' since it's simulated)
    let payment_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO payments
            (payer_id, mpesa_transaction_id, payment_type, reference_id, amount, status, paid_at)
        VALUES ($1, $2, $3, $4, $5, 'completed', NOW())
        RETURNING id
        "#
    )
        .bind(payer_id)
        .bind(mpesa_tx_id)
        .bind(payment_type)
        .bind(ref_uuid.as_ref())
        .bind(amount as f64)
        .fetch_one(pool)
        .await?;

    // 4. Process the payment (credits commission, sends emails)
    process_successful_payment(pool, email_service, &payment_id, &mpesa_tx_id).await?;

    Ok(crate::models::mpesa::PaymentResponse {
        merchant_request_id,
        checkout_request_id,
        message: format!("Simulated payment successful. Receipt: {}", receipt_number),
    })
}

// ───────────────────────────────────────────
// Process a successful payment (called after simulation or real callback)
// ───────────────────────────────────────────
pub async fn process_successful_payment(
    pool: &PgPool,
    email_service: &rento_core::email::EmailService,
    payment_id: &Uuid,
    mpesa_tx_id: &Uuid,
) -> ApiResult<()> {
    // 1. Fetch payment details
    let payment: Option<(Uuid, Uuid, String, Option<Uuid>, f64)> = sqlx::query_as(
        "SELECT payer_id, mpesa_transaction_id, payment_type, reference_id, amount::float8 FROM payments WHERE id = $1"
    )
        .bind(payment_id)
        .fetch_optional(pool)
        .await?;

    let (payer_id, _, payment_type, reference_id, amount) = match payment {
        Some(p) => p,
        None => return Err(ApiError::NotFound("Payment not found".into())),
    };

    // 2. Get payer's email for notification
    let payer_email: String = sqlx::query_scalar(
        "SELECT email FROM account_users WHERE id = $1"
    )
        .bind(payer_id)
        .fetch_one(pool)
        .await?;

    // 3. Get receipt number
    let receipt_number: String = sqlx::query_scalar(
        "SELECT mpesa_receipt_number FROM mpesa_transactions WHERE id = $1"
    )
        .bind(mpesa_tx_id)
        .fetch_one(pool)
        .await?;

    // 4. Send payment confirmation email to payer
    email_service
        .send_payment_confirmation(&payer_email, amount, &receipt_number)
        .await
        .map_err(|e| {
            tracing::error!("Failed to send payment confirmation email: {}", e);
            ApiError::Internal(format!("Email send failed: {}", e))
        })?;

    // 5. Determine the agent (from agent_conversions table)
    let agent_id: Option<Uuid> = if payment_type == "registration_fee" || payment_type == "renewal" {
        sqlx::query_scalar(
            "SELECT agent_id FROM agent_conversions WHERE property_owner_id = $1"
        )
            .bind(payer_id)
            .fetch_optional(pool)
            .await?
    } else {
        None
    };

    // 6. Calculate commission
    let (commission_type, rate) = match payment_type.as_str() {
        "registration_fee" => ("registration_30pct", REGISTRATION_COMMISSION_RATE),
        "renewal" => ("renewal_10pct", RENEWAL_COMMISSION_RATE),
        _ => return Ok(()),
    };

    let commission_amount = amount * (rate / 100.0);

    // 7. Credit agent wallet if agent exists
    if let Some(agent_id) = agent_id {
        // ✅ FIX: Ensure wallet exists BEFORE starting the transaction
        // This prevents the "Wallet not found" error when crediting
        wallet::get_or_create_wallet(pool, &agent_id).await?;

        let mut tx = pool.begin().await?;

        let ledger_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO commission_ledger
                (agent_id, payment_id, property_owner_id, property_id,
                 commission_type, gross_amount, commission_rate, commission_amount,
                 status, credited_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'credited', NOW())
            RETURNING id
            "#
        )
            .bind(agent_id)
            .bind(payment_id)
            .bind(payer_id)
            .bind(reference_id)
            .bind(commission_type)
            .bind(amount)
            .bind(rate)
            .bind(commission_amount)
            .fetch_one(&mut *tx)
            .await?;

        wallet::credit_wallet(
            &mut tx,
            &agent_id,
            commission_amount,
            &ledger_id.to_string(),
            &format!("{} commission on KES {:.2} payment", commission_type, amount),
        )
            .await?;

        tx.commit().await?;

        // 8. Send commission notification email to agent
        let agent_email: String = sqlx::query_scalar(
            "SELECT email FROM account_users WHERE id = $1"
        )
            .bind(agent_id)
            .fetch_one(pool)
            .await?;

        email_service
            .send_commission_notification(
                &agent_email,
                commission_amount,
                amount,
                commission_type,
            )
            .await
            .map_err(|e| {
                tracing::error!("Failed to send commission notification email: {}", e);
                ApiError::Internal(format!("Email send failed: {}", e))
            })?;

        tracing::info!(
            "✅ Commission credited: KES {:.2} to agent {} (payment {})",
            commission_amount, agent_id, payment_id
        );
    } else {
        tracing::info!("✅ Payment completed (no agent commission): payment {}", payment_id);
    }

    Ok(())
}

// ───────────────────────────────────────────
// Get agent's commission history
// ───────────────────────────────────────────
pub async fn get_agent_commissions(
    pool: &PgPool,
    agent_id: &Uuid,
) -> ApiResult<Vec<CommissionLedgerEntry>> {
    let rows = sqlx::query(
        r#"
        SELECT
            cl.id::text, cl.agent_id::text, cl.payment_id::text,
            cl.property_owner_id::text, cl.property_id::text,
            cl.commission_type, cl.gross_amount::float8,
            cl.commission_rate::float8, cl.commission_amount::float8,
            cl.status, cl.credited_at, cl.created_at
        FROM commission_ledger cl
        WHERE cl.agent_id = $1
        ORDER BY cl.created_at DESC
        LIMIT 100
        "#
    )
        .bind(agent_id)
        .fetch_all(pool)
        .await?;

    let entries: Vec<CommissionLedgerEntry> = rows
        .into_iter()
        .map(|row| {
            use sqlx::Row;
            CommissionLedgerEntry {
                id: row.try_get("id").unwrap_or_default(),
                agent_id: row.try_get("agent_id").unwrap_or_default(),
                payment_id: row.try_get("payment_id").unwrap_or_default(),
                property_owner_id: row.try_get("property_owner_id").unwrap_or_default(),
                property_id: row.try_get::<Option<String>, _>("property_id").ok().flatten(),
                commission_type: row.try_get("commission_type").unwrap_or_default(),
                gross_amount: row.try_get::<f64, _>("gross_amount").unwrap_or(0.0),
                commission_rate: row.try_get::<f64, _>("commission_rate").unwrap_or(0.0),
                commission_amount: row.try_get::<f64, _>("commission_amount").unwrap_or(0.0),
                status: row.try_get("status").unwrap_or_default(),
                credited_at: row
                    .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("credited_at")
                    .ok()
                    .flatten()
                    .map(|d| d.to_string()),
                created_at: row
                    .try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                    .ok()
                    .map(|d| d.to_string())
                    .unwrap_or_default(),
            }
        })
        .collect();

    Ok(entries)
}