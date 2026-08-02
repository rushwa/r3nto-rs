// crates/core/src/email.rs

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
            // Authenticated SMTP (production)
            let creds = Credentials::new(
                self.smtp_username.clone().unwrap(),
                self.smtp_password.clone().unwrap(),
            );
            SmtpTransport::starttls_relay(&self.smtp_host)
                .map_err(|e| RentoError::Email(format!("SMTP relay error: {}", e)))?
                .credentials(creds)
                .port(self.smtp_port)
        } else {
            // No-auth SMTP (MailHog local testing)
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

    fn verification_email_template(&self, code: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: Arial, sans-serif; background-color: #f4f4f4; margin: 0; padding: 0; }}
        .container {{ max-width: 600px; margin: 0 auto; background-color: #ffffff; padding: 40px; }}
        .header {{ text-align: center; padding-bottom: 30px; }}
        .header h1 {{ color: #2563eb; margin: 0; }}
        .code {{ font-size: 32px; font-weight: bold; color: #2563eb;
                 text-align: center; padding: 20px;
                 background-color: #eff6ff; border-radius: 8px;
                 letter-spacing: 4px; margin: 20px 0; }}
        .footer {{ text-align: center; color: #6b7280; font-size: 12px; margin-top: 30px; }}
        .warning {{ color: #dc2626; font-size: 13px; margin-top: 20px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Rento</h1>
            <p>Verify your email address</p>
        </div>
        <p>Hello,</p>
        <p>Use the verification code below to complete your registration:</p>
        <div class="code">{}</div>
        <p class="warning">This code expires in 10 minutes. Do not share it with anyone.</p>
        <div class="footer">
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
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: Arial, sans-serif; background-color: #f4f4f4; margin: 0; padding: 0; }}
        .container {{ max-width: 600px; margin: 0 auto; background-color: #ffffff; padding: 40px; }}
        .header {{ text-align: center; padding-bottom: 30px; }}
        .header h1 {{ color: #2563eb; margin: 0; }}
        .code {{ font-size: 32px; font-weight: bold; color: #2563eb;
                 text-align: center; padding: 20px;
                 background-color: #eff6ff; border-radius: 8px;
                 letter-spacing: 4px; margin: 20px 0; }}
        .footer {{ text-align: center; color: #6b7280; font-size: 12px; margin-top: 30px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Rento</h1>
            <p>Password Reset</p>
        </div>
        <p>Hello,</p>
        <p>You requested a password reset. Use the code below:</p>
        <div class="code">{}</div>
        <p>This code expires in 30 minutes.</p>
        <div class="footer">
            <p>If you didn't request this, please ignore this email.</p>
            <p>&copy; 2026 Rento. All rights reserved.</p>
        </div>
    </div>
</body>
</html>"#,
            code
        )
    }
}