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

    // ==========================================
    // NEW: Payment Confirmation Email (M-Pesa Simulation)
    // ==========================================
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

    // ==========================================
    // NEW: Commission Notification Email (M-Pesa Simulation)
    // ==========================================
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

    fn verification_email_template(&self, code: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8">
<style>
body {{ font-family: Arial, sans-serif; background-color: #f4f4f4; margin: 0; padding: 0; }}
.container {{ max-width: 600px; margin: 0 auto; background-color: #ffffff; padding: 40px; }}
.header {{ text-align: center; padding-bottom: 30px; }}
.header h1 {{ color: #2563eb; margin: 0; }}
.code {{ font-size: 32px; font-weight: bold; color: #2563eb; text-align: center; padding: 20px; background-color: #eff6ff; border-radius: 8px; letter-spacing: 4px; margin: 20px 0; }}
.footer {{ text-align: center; color: #6b7280; font-size: 12px; margin-top: 30px; }}
.warning {{ color: #dc2626; font-size: 13px; margin-top: 20px; }}
</style></head><body>
<div class="container">
<div class="header"><h1>Rento</h1><p>Verify your email address</p></div>
<p>Hello,</p>
<p>Use the verification code below to complete your registration:</p>
<div class="code">{}</div>
<p class="warning">This code expires in 10 minutes. Do not share it with anyone.</p>
<div class="footer"><p>If you didn't request this code, you can safely ignore this email.</p><p>&copy; 2026 Rento. All rights reserved.</p></div>
</div></body></html>"#,
            code
        )
    }

    fn password_reset_template(&self, code: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8">
<style>
body {{ font-family: Arial, sans-serif; background-color: #f4f4f4; margin: 0; padding: 0; }}
.container {{ max-width: 600px; margin: 0 auto; background-color: #ffffff; padding: 40px; }}
.header {{ text-align: center; padding-bottom: 30px; }}
.header h1 {{ color: #2563eb; margin: 0; }}
.code {{ font-size: 32px; font-weight: bold; color: #2563eb; text-align: center; padding: 20px; background-color: #eff6ff; border-radius: 8px; letter-spacing: 4px; margin: 20px 0; }}
.footer {{ text-align: center; color: #6b7280; font-size: 12px; margin-top: 30px; }}
</style></head><body>
<div class="container">
<div class="header"><h1>Rento</h1><p>Password Reset</p></div>
<p>Hello,</p>
<p>You requested a password reset. Use the code below:</p>
<div class="code">{}</div>
<p>This code expires in 30 minutes.</p>
<div class="footer"><p>If you didn't request this, please ignore this email.</p><p>&copy; 2026 Rento. All rights reserved.</p></div>
</div></body></html>"#,
            code
        )
    }

    fn handshake_email_template(&self, owner_name: &str, agent_name: &str, code: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8">
<style>
body {{ font-family: Arial, sans-serif; background-color: #f4f4f4; margin: 0; padding: 0; }}
.container {{ max-width: 600px; margin: 0 auto; background-color: #ffffff; padding: 40px; }}
.header {{ text-align: center; padding-bottom: 30px; }}
.header h1 {{ color: #2563eb; margin: 0; }}
.code {{ font-size: 32px; font-weight: bold; color: #2563eb; text-align: center; padding: 20px; background-color: #eff6ff; border-radius: 8px; letter-spacing: 4px; margin: 20px 0; }}
.info-box {{ background-color: #eff6ff; border-left: 4px solid #2563eb; padding: 15px; margin: 20px 0; }}
.footer {{ text-align: center; color: #6b7280; font-size: 12px; margin-top: 30px; }}
.warning {{ color: #dc2626; font-size: 13px; margin-top: 20px; }}
</style></head><body>
<div class="container">
<div class="header"><h1>🛡️ Digital Title Deed Protection</h1></div>
<p>Dear <strong>{}</strong>,</p>
<p>An agent, <strong>{}</strong>, has requested authorization to manage your property on the Rento platform.</p>
<div class="info-box"><p><strong>What is this?</strong><br>This 6-digit code is your <em>Digital Title Deed Protection</em>. By sharing it with the agent, you legally authorize them to list and manage your property.</p></div>
<p>Your verification code:</p>
<div class="code">{}</div>
<p class="warning">⚠️ This code expires in 15 minutes. Never share it with anyone other than your assigned Rento agent.</p>
<div class="footer"><p>If you did not request this, please ignore this email or contact support immediately.</p><p>&copy; 2026 Rento. All rights reserved.</p></div>
</div></body></html>"#,
            owner_name, agent_name, code
        )
    }

    fn payment_confirmation_template(&self, amount: f64, receipt_number: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8">
<style>
body {{ font-family: Arial, sans-serif; background-color: #f4f4f4; margin: 0; padding: 0; }}
.container {{ max-width: 600px; margin: 0 auto; background-color: #ffffff; padding: 40px; }}
.header {{ text-align: center; padding-bottom: 30px; }}
.header h1 {{ color: #10b981; margin: 0; }}
.receipt {{ font-size: 24px; font-weight: bold; color: #1f2937; text-align: center; padding: 20px; background-color: #d1fae5; border-radius: 8px; letter-spacing: 2px; margin: 20px 0; }}
.amount {{ font-size: 32px; font-weight: bold; color: #10b981; text-align: center; }}
.footer {{ text-align: center; color: #6b7280; font-size: 12px; margin-top: 30px; }}
</style></head><body>
<div class="container">
<div class="header"><h1>✅ Payment Received</h1></div>
<p>Dear Customer,</p>
<p>Your payment has been successfully processed:</p>
<div class="amount">KES {:.2}</div>
<p>Receipt Number:</p>
<div class="receipt">{}</div>
<p>Your property listing is now active. Thank you for choosing Rento!</p>
<div class="footer"><p>&copy; 2026 Rento. All rights reserved.</p></div>
</div></body></html>"#,
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
<html><head><meta charset="UTF-8">
<style>
body {{ font-family: Arial, sans-serif; background-color: #f4f4f4; margin: 0; padding: 0; }}
.container {{ max-width: 600px; margin: 0 auto; background-color: #ffffff; padding: 40px; }}
.header {{ text-align: center; padding-bottom: 30px; }}
.header h1 {{ color: #f59e0b; margin: 0; }}
.commission {{ font-size: 36px; font-weight: bold; color: #10b981; text-align: center; padding: 20px; background-color: #d1fae5; border-radius: 8px; margin: 20px 0; }}
.details {{ background-color: #f9fafb; padding: 20px; border-radius: 8px; margin: 20px 0; }}
.footer {{ text-align: center; color: #6b7280; font-size: 12px; margin-top: 30px; }}
</style></head><body>
<div class="container">
<div class="header"><h1>💰 Commission Credited!</h1></div>
<p>Dear Agent,</p>
<p>A new commission has been credited to your wallet:</p>
<div class="commission">KES {:.2}</div>
<div class="details">
<p><strong>Type:</strong> {}</p>
<p><strong>Payment Amount:</strong> KES {:.2}</p>
<p><strong>Your Commission:</strong> KES {:.2}</p>
</div>
<p>Log in to your agent dashboard to view your wallet balance and request a payout.</p>
<div class="footer"><p>&copy; 2026 Rento. All rights reserved.</p></div>
</div></body></html>"#,
            commission_amount, commission_label, payment_amount, commission_amount
        )
    }
}