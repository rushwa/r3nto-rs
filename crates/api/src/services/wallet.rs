use uuid::Uuid;
use sqlx::PgPool;

use crate::errors::{ApiError, ApiResult};
use crate::models::mpesa::WalletInfo;

pub async fn get_or_create_wallet(pool: &PgPool, agent_id: &Uuid) -> ApiResult<WalletInfo> {
    let existing: Option<(String, f64, f64, f64, f64, Option<String>)> = sqlx::query_as(
        r#"
        SELECT
            agent_id::text, balance::float8, pending_balance::float8,
            total_earned::float8, total_withdrawn::float8, mpesa_phone
        FROM agent_wallets WHERE agent_id = $1
        "#
    )
        .bind(agent_id)
        .fetch_optional(pool)
        .await?;

    if let Some(row) = existing {
        return Ok(WalletInfo {
            agent_id: row.0,
            balance: row.1,
            pending_balance: row.2,
            total_earned: row.3,
            total_withdrawn: row.4,
            mpesa_phone: row.5,
        });
    }

    sqlx::query("INSERT INTO agent_wallets (agent_id) VALUES ($1)")
        .bind(agent_id)
        .execute(pool)
        .await?;

    Ok(WalletInfo {
        agent_id: agent_id.to_string(),
        balance: 0.0,
        pending_balance: 0.0,
        total_earned: 0.0,
        total_withdrawn: 0.0,
        mpesa_phone: None,
    })
}

pub async fn credit_wallet(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    agent_id: &Uuid,
    amount: f64,
    reference: &str,
    description: &str,
) -> ApiResult<()> {
    // ✅ Must update BOTH balance AND total_earned
    sqlx::query(
        r#"
        UPDATE agent_wallets
        SET balance = balance + $1,
            total_earned = total_earned + $1,
            updated_at = NOW()
        WHERE agent_id = $2
        "#
    )
        .bind(amount)
        .bind(agent_id)
        .execute(&mut **tx)
        .await?;

    // Record in wallet_transactions
    sqlx::query(
        r#"
        INSERT INTO wallet_transactions (agent_id, amount, transaction_type, reference, description, status)
        VALUES ($1, $2, 'credit', $3, $4, 'completed')
        "#
    )
        .bind(agent_id)
        .bind(amount)
        .bind(reference)
        .bind(description)
        .execute(&mut **tx)
        .await?;

    Ok(())
}

pub async fn debit_wallet(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    agent_id: &Uuid,
    amount: f64,
    reference: &str,
    description: &str,
) -> ApiResult<()> {
    let wallet_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM agent_wallets WHERE agent_id = $1 FOR UPDATE"
    )
        .bind(agent_id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| ApiError::Internal("Wallet not found".into()))?;

    let current_balance: f64 = sqlx::query_scalar(
        "SELECT balance::float8 FROM agent_wallets WHERE id = $1"
    )
        .bind(wallet_id)
        .fetch_one(&mut **tx)
        .await?;

    if current_balance < amount {
        return Err(ApiError::BadRequest(
            format!("Insufficient balance. Available: KES {:.2}, Requested: KES {:.2}", current_balance, amount)
        ));
    }

    let new_balance = current_balance - amount;

    sqlx::query(
        "UPDATE agent_wallets SET balance = $1, total_withdrawn = total_withdrawn + $2, updated_at = NOW() WHERE id = $3"
    )
        .bind(new_balance)
        .bind(amount)
        .bind(wallet_id)
        .execute(&mut **tx)
        .await?;

    sqlx::query(
        r#"
        INSERT INTO wallet_transactions
            (wallet_id, transaction_type, amount, balance_before, balance_after, reference, description)
        VALUES ($1, 'withdrawal', $2, $3, $4, $5, $6)
        "#
    )
        .bind(wallet_id)
        .bind(amount)
        .bind(current_balance)
        .bind(new_balance)
        .bind(reference)
        .bind(description)
        .execute(&mut **tx)
        .await?;

    Ok(())
}

pub async fn get_wallet_history(
    pool: &PgPool,
    agent_id: &Uuid,
    limit: i64,
) -> ApiResult<Vec<serde_json::Value>> {
    let rows = sqlx::query(
        r#"
        SELECT
            wt.id::text, wt.transaction_type, wt.amount::float8,
            wt.balance_before::float8, wt.balance_after::float8,
            wt.reference, wt.description, wt.created_at
        FROM wallet_transactions wt
        JOIN agent_wallets aw ON wt.wallet_id = aw.id
        WHERE aw.agent_id = $1
        ORDER BY wt.created_at DESC
        LIMIT $2
        "#
    )
        .bind(agent_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;

    let history: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        use sqlx::Row;
        serde_json::json!({
            "id": row.try_get::<String, _>("id").unwrap_or_default(),
            "type": row.try_get::<String, _>("transaction_type").unwrap_or_default(),
            "amount": row.try_get::<f64, _>("amount").unwrap_or(0.0),
            "balance_before": row.try_get::<f64, _>("balance_before").unwrap_or(0.0),
            "balance_after": row.try_get::<f64, _>("balance_after").unwrap_or(0.0),
            "reference": row.try_get::<Option<String>, _>("reference").unwrap_or_default(),
            "description": row.try_get::<Option<String>, _>("description").unwrap_or_default(),
            "created_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                .map(|d| d.to_string()).unwrap_or_default(),
        })
    }).collect();

    Ok(history)
}