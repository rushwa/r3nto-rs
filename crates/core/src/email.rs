use lettre::{
    message::{Message, SinglePart},
    transport::smtp::authentication::Credentials,
    SmtpTransport, Transport,
};
use crate::error::RentoError;

pub struct EmailService {
    smtp_host: String,
    smtp_port: u16,
    smtp_username: Option<String>,
    smtp_password: Option<String>,
    from_email: String,
    from_name: String,
}

impl EmailService {
    pub fn new(
        smtp_host: String,
        smtp_port: u16,
        smtp_username: Option<String>,
        smtp_password: Option<String>,
        from_email: String,
        from_name: String,
    ) -> Self {
        Self {
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            from_email,
            from_name,
        }
    }

    pub fn from_env() -> Result<Self, RentoError> {
        let smtp_host = std::env::var("SMTP_HOST")
            .unwrap_or_else(|_| "localhost".to_string());
        let smtp_port = std::env::var("SMTP_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(1025);
        let smtp_username = std::env::var("SMTP_USERNAME").ok();
        let smtp_password = std::env::var("SMTP_PASSWORD").ok();
        let from_email = std::env::var("FROM_EMAIL")
            .unwrap_or_else(|_| "noreply@rento.local".to_string());
        let from_name = std::env::var("FROM_NAME")
            .unwrap_or_else(|_| "Rento".to_string());

        Ok(Self::new(
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            from_email,
            from_name,
        ))
    }

    fn build_transport(&self) -> Result<SmtpTransport, RentoError> {
        let builder = if self.smtp_username.is_some() && self.smtp_password.is_some() {
            let creds = Credentials::new(
                self.smtp_username.clone().unwrap(),
                self.smtp_password.clone().unwrap(),
            );
            SmtpTransport::starttls_relay(&self.smtp_host)
                .map_err(|e| RentoError::Email(format!("SMTP relay error: {}", e)))?
                .credentials(creds)
                .port(self.smtp_port)
        } else {
            SmtpTransport::builder_dangerous(&self.smtp_host)
                .port(self.smtp_port)
        };
        Ok(builder.build())
    }

    // ───────────────────────────────────────────
    // Verification Code Email
    // ───────────────────────────────────────────
    pub async fn send_verification_code(
        &self,
        to_email: &str,
        to_name: Option<&str>,
        code: &str,
    ) -> Result<(), RentoError> {
        let recipient = if let Some(name) = to_name {
            format!("{} <{}>", name, to_email)
        } else {
            to_email.to_string()
        };

        let email = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_email).parse()
                .map_err(|e| RentoError::Email(format!("Invalid from address: {}", e)))?)
            .to(recipient.parse()
                .map_err(|e| RentoError::Email(format!("Invalid to address: {}", e)))?)
            .subject("Your Rento Verification Code")
            .singlepart(
                SinglePart::builder()
                    .header(lettre::message::header::ContentType::TEXT_HTML)
                    .body(self.verification_email_template(code)),
            )
            .map_err(|e| RentoError::Email(format!("Email build error: {}", e)))?;

        let transport = self.build_transport()?;
        transport.send(&email)
            .map_err(|e| RentoError::Email(format!("Failed to send email: {}", e)))?;

        tracing::info!("Verification email sent to {}", to_email);
        Ok(())
    }

    // ───────────────────────────────────────────
    // Password Reset Email
    // ───────────────────────────────────────────
    pub async fn send_password_reset(
        &self,
        to_email: &str,
        code: &str,
    ) -> Result<(), RentoError> {
        let email = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_email).parse()
                .map_err(|e| RentoError::Email(format!("Invalid from address: {}", e)))?)
            .to(to_email.parse()
                .map_err(|e| RentoError::Email(format!("Invalid to address: {}", e)))?)
            .subject("Password Reset Request")
            .singlepart(
                SinglePart::builder()
                    .header(lettre::message::header::ContentType::TEXT_HTML)
                    .body(self.password_reset_template(code)),
            )
            .map_err(|e| RentoError::Email(format!("Email build error: {}", e)))?;

        let transport = self.build_transport()?;
        transport.send(&email)
            .map_err(|e| RentoError::Email(format!("Failed to send email: {}", e)))?;

        tracing::info!("Password reset email sent to {}", to_email);
        Ok(())
    }

    // ───────────────────────────────────────────
    // Handshake OTP Email
    // ───────────────────────────────────────────
    pub async fn send_handshake_otp(
        &self,
        to_email: &str,
        owner_name: &str,
        agent_name: &str,
        code: &str,
    ) -> Result<(), RentoError> {
        let recipient = format!("{} <{}>", owner_name, to_email);

        let email = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_email).parse()
                .map_err(|e| RentoError::Email(format!("Invalid from address: {}", e)))?)
            .to(recipient.parse()
                .map_err(|e| RentoError::Email(format!("Invalid to address: {}", e)))?)
            .subject("Your Rento Digital Handshake Code")
            .singlepart(
                SinglePart::builder()
                    .header(lettre::message::header::ContentType::TEXT_HTML)
                    .body(self.handshake_email_template(owner_name, agent_name, code)),
            )
            .map_err(|e| RentoError::Email(format!("Email build error: {}", e)))?;

        let transport = self.build_transport()?;
        transport.send(&email)
            .map_err(|e| RentoError::Email(format!("Failed to send email: {}", e)))?;

        tracing::info!("Handshake OTP email sent to {}", to_email);
        Ok(())
    }

    // ───────────────────────────────────────────
    // Payment Confirmation Email
    // ───────────────────────────────────────────
    pub async fn send_payment_confirmation(
        &self,
        to_email: &str,
        amount: f64,
        receipt_number: &str,
    ) -> Result<(), RentoError> {
        let email = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_email).parse()
                .map_err(|e| RentoError::Email(format!("Invalid from address: {}", e)))?)
            .to(to_email.parse()
                .map_err(|e| RentoError::Email(format!("Invalid to address: {}", e)))?)
            .subject("Payment Confirmation - Rento")
            .singlepart(
                SinglePart::builder()
                    .header(lettre::message::header::ContentType::TEXT_HTML)
                    .body(self.payment_confirmation_template(amount, receipt_number)),
            )
            .map_err(|e| RentoError::Email(format!("Email build error: {}", e)))?;

        let transport = self.build_transport()?;
        transport.send(&email)
            .map_err(|e| RentoError::Email(format!("Failed to send email: {}", e)))?;

        tracing::info!("Payment confirmation email sent to {}", to_email);
        Ok(())
    }

    // ───────────────────────────────────────────
    // Commission Notification Email
    // ───────────────────────────────────────────
    pub async fn send_commission_notification(
        &self,
        to_email: &str,
        commission_amount: f64,
        payment_amount: f64,
        commission_type: &str,
    ) -> Result<(), RentoError> {
        let email = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_email).parse()
                .map_err(|e| RentoError::Email(format!("Invalid from address: {}", e)))?)
            .to(to_email.parse()
                .map_err(|e| RentoError::Email(format!("Invalid to address: {}", e)))?)
            .subject("Commission Credited - Rento")
            .singlepart(
                SinglePart::builder()
                    .header(lettre::message::header::ContentType::TEXT_HTML)
                    .body(self.commission_notification_template(commission_amount, payment_amount, commission_type)),
            )
            .map_err(|e| RentoError::Email(format!("Email build error: {}", e)))?;

        let transport = self.build_transport()?;
        transport.send(&email)
            .map_err(|e| RentoError::Email(format!("Failed to send email: {}", e)))?;

        tracing::info!("Commission notification email sent to {}", to_email);
        Ok(())
    }

    // ───────────────────────────────────────────
    // ✅ NEW: Payout Approved Email
    // ───────────────────────────────────────────
    pub async fn send_payout_approved(
        &self,
        to_email: &str,
        agent_name: &str,
        amount: f64,
        phone: &str,
    ) -> Result<(), RentoError> {
        let recipient = format!("{} <{}>", agent_name, to_email);

        let email = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_email).parse()
                .map_err(|e| RentoError::Email(format!("Invalid from address: {}", e)))?)
            .to(recipient.parse()
                .map_err(|e| RentoError::Email(format!("Invalid to address: {}", e)))?)
            .subject("✅ Your Payout Has Been Approved - Rento")
            .singlepart(
                SinglePart::builder()
                    .header(lettre::message::header::ContentType::TEXT_HTML)
                    .body(self.payout_approved_template(agent_name, amount, phone)),
            )
            .map_err(|e| RentoError::Email(format!("Email build error: {}", e)))?;

        let transport = self.build_transport()?;
        transport.send(&email)
            .map_err(|e| RentoError::Email(format!("Failed to send email: {}", e)))?;

        tracing::info!("Payout approval email sent to {}", to_email);
        Ok(())
    }

    // ───────────────────────────────────────────
    // ✅ NEW: Payout Rejected Email
    // ───────────────────────────────────────────
    pub async fn send_payout_rejected(
        &self,
        to_email: &str,
        agent_name: &str,
        amount: f64,
    ) -> Result<(), RentoError> {
        let recipient = format!("{} <{}>", agent_name, to_email);

        let email = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_email).parse()
                .map_err(|e| RentoError::Email(format!("Invalid from address: {}", e)))?)
            .to(recipient.parse()
                .map_err(|e| RentoError::Email(format!("Invalid to address: {}", e)))?)
            .subject("Payout Request Update - Rento")
            .singlepart(
                SinglePart::builder()
                    .header(lettre::message::header::ContentType::TEXT_HTML)
                    .body(self.payout_rejected_template(agent_name, amount)),
            )
            .map_err(|e| RentoError::Email(format!("Email build error: {}", e)))?;

        let transport = self.build_transport()?;
        transport.send(&email)
            .map_err(|e| RentoError::Email(format!("Failed to send email: {}", e)))?;

        tracing::info!("Payout rejection email sent to {}", to_email);
        Ok(())
    }

    // ═══════════════════════════════════════════
    // EMAIL TEMPLATES
    // ═══════════════════════════════════════════

    fn verification_email_template(&self, code: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head><meta charset="UTF-8"></head>
<body style="font-family: Arial, sans-serif; background-color: #f4f4f4; margin: 0; padding: 0;">
<div style="max-width: 600px; margin: 0 auto; background-color: #ffffff; padding: 40px;">
    <div style="text-align: center; padding-bottom: 30px;">
        <h1 style="color: #2563eb; margin: 0;">Rento</h1>
        <p>Verify your email address</p>
    </div>
    <p>Hello,</p>
    <p>Use the verification code below to complete your registration:</p>
    <div style="font-size: 32px; font-weight: bold; color: #2563eb; text-align: center; padding: 20px; background-color: #eff6ff; border-radius: 8px; letter-spacing: 4px; margin: 20px 0;">{}</div>
    <p style="color: #dc2626; font-size: 13px;">This code expires in 10 minutes. Do not share it with anyone.</p>
    <div style="text-align: center; color: #6b7280; font-size: 12px; margin-top: 30px;">
        <p>If you didn't request this code, you can safely ignore this email.</p>
        <p>&copy; 2026 Rento. All rights reserved.</p>
    </div>
</div>
</body>
</html>"#,
            code
        )
    }

    fn password_reset_template(&self, code: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head><meta charset="UTF-8"></head>
<body style="font-family: Arial, sans-serif; background-color: #f4f4f4; margin: 0; padding: 0;">
<div style="max-width: 600px; margin: 0 auto; background-color: #ffffff; padding: 40px;">
    <div style="text-align: center; padding-bottom: 30px;">
        <h1 style="color: #2563eb; margin: 0;">Rento</h1>
        <p>Password Reset</p>
    </div>
    <p>Hello,</p>
    <p>You requested a password reset. Use the code below:</p>
    <div style="font-size: 32px; font-weight: bold; color: #2563eb; text-align: center; padding: 20px; background-color: #eff6ff; border-radius: 8px; letter-spacing: 4px; margin: 20px 0;">{}</div>
    <p>This code expires in 30 minutes.</p>
    <div style="text-align: center; color: #6b7280; font-size: 12px; margin-top: 30px;">
        <p>If you didn't request this, please ignore this email.</p>
        <p>&copy; 2026 Rento. All rights reserved.</p>
    </div>
</div>
</body>
</html>"#,
            code
        )
    }

    fn handshake_email_template(&self, owner_name: &str, agent_name: &str, code: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head><meta charset="UTF-8"></head>
<body style="font-family: Arial, sans-serif; background-color: #f4f4f4; margin: 0; padding: 0;">
<div style="max-width: 600px; margin: 0 auto; background-color: #ffffff; padding: 40px;">
    <div style="text-align: center; padding-bottom: 30px;">
        <h1 style="color: #2563eb; margin: 0;">🛡️ Digital Title Deed Protection</h1>
    </div>
    <p>Dear <strong>{}</strong>,</p>
    <p>An agent, <strong>{}</strong>, has requested authorization to manage your property on the Rento platform.</p>
    <div style="background-color: #eff6ff; border-left: 4px solid #2563eb; padding: 15px; margin: 20px 0;">
        <p><strong>What is this?</strong><br>This 6-digit code is your <em>Digital Title Deed Protection</em>. By sharing it with the agent, you legally authorize them to list and manage your property.</p>
    </div>
    <p>Your verification code:</p>
    <div style="font-size: 32px; font-weight: bold; color: #2563eb; text-align: center; padding: 20px; background-color: #eff6ff; border-radius: 8px; letter-spacing: 4px; margin: 20px 0;">{}</div>
    <p style="color: #dc2626; font-size: 13px;">⚠️ This code expires in 15 minutes. Never share it with anyone other than your assigned Rento agent.</p>
    <div style="text-align: center; color: #6b7280; font-size: 12px; margin-top: 30px;">
        <p>If you did not request this, please ignore this email or contact support immediately.</p>
        <p>&copy; 2026 Rento. All rights reserved.</p>
    </div>
</div>
</body>
</html>"#,
            owner_name, agent_name, code
        )
    }

    fn payment_confirmation_template(&self, amount: f64, receipt_number: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head><meta charset="UTF-8"></head>
<body style="font-family: Arial, sans-serif; background-color: #f4f4f4; margin: 0; padding: 0;">
<div style="max-width: 600px; margin: 0 auto; background-color: #ffffff; padding: 40px;">
    <div style="text-align: center; padding-bottom: 30px;">
        <h1 style="color: #10b981; margin: 0;">✅ Payment Received</h1>
    </div>
    <p>Dear Customer,</p>
    <p>Your payment has been successfully processed:</p>
    <div style="font-size: 32px; font-weight: bold; color: #10b981; text-align: center;">KES {:.2}</div>
    <p>Receipt Number:</p>
    <div style="font-size: 24px; font-weight: bold; color: #1f2937; text-align: center; padding: 20px; background-color: #d1fae5; border-radius: 8px; letter-spacing: 2px; margin: 20px 0;">{}</div>
    <p>Your property listing is now active. Thank you for choosing Rento!</p>
    <div style="text-align: center; color: #6b7280; font-size: 12px; margin-top: 30px;">
        <p>&copy; 2026 Rento. All rights reserved.</p>
    </div>
</div>
</body>
</html>"#,
            amount, receipt_number
        )
    }

    fn commission_notification_template(&self, commission_amount: f64, payment_amount: f64, commission_type: &str) -> String {
        let commission_label = if commission_type.contains("registration") {
            "Registration Fee Commission (30%)"
        } else {
            "Subscription Renewal Commission (10%)"
        };
        format!(
            r#"<!DOCTYPE html>
<html>
<head><meta charset="UTF-8"></head>
<body style="font-family: Arial, sans-serif; background-color: #f4f4f4; margin: 0; padding: 0;">
<div style="max-width: 600px; margin: 0 auto; background-color: #ffffff; padding: 40px;">
    <div style="text-align: center; padding-bottom: 30px;">
        <h1 style="color: #f59e0b; margin: 0;">💰 Commission Credited!</h1>
    </div>
    <p>Dear Agent,</p>
    <p>A new commission has been credited to your wallet:</p>
    <div style="font-size: 36px; font-weight: bold; color: #10b981; text-align: center; padding: 20px; background-color: #d1fae5; border-radius: 8px; margin: 20px 0;">KES {:.2}</div>
    <div style="background-color: #f9fafb; padding: 20px; border-radius: 8px; margin: 20px 0;">
        <p><strong>Type:</strong> {}</p>
        <p><strong>Payment Amount:</strong> KES {:.2}</p>
        <p><strong>Your Commission:</strong> KES {:.2}</p>
    </div>
    <p>Log in to your agent dashboard to view your wallet balance and request a payout.</p>
    <div style="text-align: center; color: #6b7280; font-size: 12px; margin-top: 30px;">
        <p>&copy; 2026 Rento. All rights reserved.</p>
    </div>
</div>
</body>
</html>"#,
            commission_amount, commission_label, payment_amount, commission_amount
        )
    }

    // ───────────────────────────────────────────
    // ✅ NEW: Payout Approved Template
    // ───────────────────────────────────────────
    fn payout_approved_template(&self, agent_name: &str, amount: f64, phone: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head><meta charset="UTF-8"></head>
<body style="font-family: Arial, sans-serif; background-color: #f4f4f4; margin: 0; padding: 0;">
<div style="max-width: 600px; margin: 0 auto; background-color: #ffffff; padding: 40px;">
    <div style="text-align: center; padding-bottom: 30px;">
        <h1 style="color: #10b981; margin: 0;">✅ Payout Approved</h1>
    </div>
    <p>Hi <strong>{}</strong>,</p>
    <p>Great news! Your payout request has been approved and is being processed.</p>
    <div style="background-color: #d1fae5; border-left: 4px solid #10b981; padding: 20px; margin: 20px 0; border-radius: 8px;">
        <p style="margin: 0; font-size: 14px; color: #065f46;"><strong>Amount:</strong></p>
        <p style="margin: 5px 0; font-size: 32px; font-weight: bold; color: #10b981;">KES {:.2}</p>
        <p style="margin: 10px 0 0 0; font-size: 14px; color: #065f46;"><strong>Sending to:</strong> {}</p>
    </div>
    <p>The funds will be sent to your M-Pesa account shortly. You should receive an SMS confirmation from Safaricom once the transfer is complete.</p>
    <div style="background-color: #f9fafb; padding: 15px; border-radius: 8px; margin: 20px 0;">
        <p style="margin: 0; font-size: 13px; color: #6b7280;"><strong>What's next?</strong><br>You can request another payout once your wallet balance is sufficient. Keep converting clients to earn more commissions!</p>
    </div>
    <p>Thank you for being a valued Rento agent!</p>
    <div style="text-align: center; color: #6b7280; font-size: 12px; margin-top: 30px;">
        <p>&copy; 2026 Rento. All rights reserved.</p>
    </div>
</div>
</body>
</html>"#,
            agent_name, amount, phone
        )
    }

    // ───────────────────────────────────────────
    // ✅ NEW: Payout Rejected Template
    // ───────────────────────────────────────────
    fn payout_rejected_template(&self, agent_name: &str, amount: f64) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head><meta charset="UTF-8"></head>
<body style="font-family: Arial, sans-serif; background-color: #f4f4f4; margin: 0; padding: 0;">
<div style="max-width: 600px; margin: 0 auto; background-color: #ffffff; padding: 40px;">
    <div style="text-align: center; padding-bottom: 30px;">
        <h1 style="color: #dc2626; margin: 0;">Payout Request Update</h1>
    </div>
    <p>Hi <strong>{}</strong>,</p>
    <p>Your payout request of <strong>KES {:.2}</strong> has been reviewed and unfortunately could not be processed at this time.</p>
    <div style="background-color: #fef2f2; border-left: 4px solid #dc2626; padding: 20px; margin: 20px 0; border-radius: 8px;">
        <p style="margin: 0; font-size: 14px; color: #991b1b;"><strong>Amount Refunded:</strong></p>
        <p style="margin: 5px 0; font-size: 28px; font-weight: bold; color: #dc2626;">KES {:.2}</p>
        <p style="margin: 10px 0 0 0; font-size: 13px; color: #991b1b;">✅ Funds have been returned to your wallet</p>
    </div>
    <p><strong>What happened?</strong><br>Payout requests may be declined for various reasons including verification checks or system reviews. The funds have been fully refunded to your wallet balance and are available for future payout requests.</p>
    <div style="background-color: #f9fafb; padding: 15px; border-radius: 8px; margin: 20px 0;">
        <p style="margin: 0; font-size: 13px; color: #6b7280;"><strong>Need help?</strong><br>If you believe this was a mistake, please contact our support team at support@rento.com with your payout request details.</p>
    </div>
    <p>We apologize for any inconvenience and appreciate your understanding.</p>
    <div style="text-align: center; color: #6b7280; font-size: 12px; margin-top: 30px;">
        <p>&copy; 2026 Rento. All rights reserved.</p>
    </div>
</div>
</body>
</html>"#,
            agent_name, amount, amount
        )
    }

    // ───────────────────────────────────────────
    // ✅ NEW: Subscription Confirmation Email
    // ───────────────────────────────────────────
    pub async fn send_subscription_confirmation(
        &self,
        to_email: &str,
        amount: f64,
        plan_name: &str,
        receipt_number: &str,
        end_date: &str,
    ) -> Result<(), RentoError> {
        let email = Message::builder()
            .from(format!("{} <{}>", self.from_name, self.from_email).parse()
                .map_err(|e| RentoError::Email(format!("Invalid from address: {}", e)))?)
            .to(to_email.parse()
                .map_err(|e| RentoError::Email(format!("Invalid to address: {}", e)))?)
            .subject(format!("✅ Subscription Activated - {} Plan", plan_name))
            .singlepart(
                SinglePart::builder()
                    .header(lettre::message::header::ContentType::TEXT_HTML)
                    .body(self.subscription_confirmation_template(amount, plan_name, receipt_number, end_date)),
            )
            .map_err(|e| RentoError::Email(format!("Email build error: {}", e)))?;

        let transport = self.build_transport()?;
        transport.send(&email)
            .map_err(|e| RentoError::Email(format!("Failed to send email: {}", e)))?;

        tracing::info!("Subscription confirmation email sent to {}", to_email);
        Ok(())
    }

    fn subscription_confirmation_template(&self, amount: f64, plan_name: &str, receipt_number: &str, end_date: &str) -> String {
        let end_date_display = if end_date.len() > 10 { &end_date[..10] } else { end_date };
        format!(
            r#"<!DOCTYPE html>
<html>
<head><meta charset="UTF-8"></head>
<body style="font-family: Arial, sans-serif; background-color: #f4f4f4; margin: 0; padding: 0;">
<div style="max-width: 600px; margin: 0 auto; background-color: #ffffff; padding: 40px;">
    <div style="text-align: center; padding-bottom: 30px;">
        <h1 style="color: #2563eb; margin: 0;">⭐ Subscription Activated</h1>
    </div>
    <p>Dear Property Owner,</p>
    <p>Your property has been successfully subscribed to the <strong>{}</strong> plan!</p>
    <div style="background-color: #eff6ff; border-left: 4px solid #2563eb; padding: 20px; margin: 20px 0; border-radius: 8px;">
        <p style="margin: 0; font-size: 14px; color: #1e40af;"><strong>Plan:</strong> {}</p>
        <p style="margin: 10px 0 0 0; font-size: 14px; color: #1e40af;"><strong>Amount Paid:</strong> KES {:.2}</p>
        <p style="margin: 10px 0 0 0; font-size: 14px; color: #1e40af;"><strong>Valid Until:</strong> {}</p>
    </div>
    <p>Receipt Number:</p>
    <div style="font-size: 20px; font-weight: bold; color: #1f2937; text-align: center; padding: 15px; background-color: #d1fae5; border-radius: 8px; letter-spacing: 2px; margin: 20px 0;">{}</div>
    <p>Your property now enjoys all the benefits of the {} plan, including boosted visibility and premium features.</p>
    <div style="text-align: center; color: #6b7280; font-size: 12px; margin-top: 30px;">
        <p>&copy; 2026 Rento. All rights reserved.</p>
    </div>
</div>
</body>
</html>"#,
            plan_name, plan_name, amount, end_date_display, receipt_number, plan_name
        )
    }
}