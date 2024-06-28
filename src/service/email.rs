use std::env;

use lettre::{
    address::AddressError,
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};

use crate::config;

#[derive(Clone)]
pub struct EmailService;

impl EmailService {
    pub async fn send_password_reset_email(
        receiver_email: String,
        token: String,
    ) -> Result<(), EmailServiceError> {
        let reset_link = format!(
            "{}/admin/reset-password?token={}",
            env::var("BASE_URL").expect("BASE_URL env var not defined"),
            token
        );
        let email = Message::builder()
            .from(Mailbox::new(
                Some(config::APP_NAME.into()),
                env::var("SMTP_FROM")
                    .expect("No SMTP_FROM env var provided")
                    .parse()?,
            ))
            .to(receiver_email.parse()?)
            .subject("Reset your password")
            .header(ContentType::TEXT_HTML)
            .body(format!(
                r#"<p>Reset your password by clicking this link: <a href="{reset_link}">{reset_link}</a></p>"#,
            ))?;

        let credentials = Credentials::new(
            env::var("SMTP_USER").expect("SMTP_USER env var not defined"),
            env::var("SMTP_PASS").expect("SMTP_PASS env var not defined"),
        );
        let mailer =
            SmtpTransport::relay(&env::var("SMTP_HOST").expect("SMTP_HOST env var not defined"))?
                .credentials(credentials)
                .build();
        match mailer.send(&email) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum EmailServiceError {
    #[error("Invalid address format")]
    Address(AddressError),

    #[error("Error when building email")]
    Lettre(lettre::error::Error),

    #[error("Transport error when sending email")]
    Transport(lettre::transport::smtp::Error),
}

impl From<AddressError> for EmailServiceError {
    fn from(value: AddressError) -> Self {
        EmailServiceError::Address(value)
    }
}

impl From<lettre::error::Error> for EmailServiceError {
    fn from(value: lettre::error::Error) -> Self {
        EmailServiceError::Lettre(value)
    }
}
impl From<lettre::transport::smtp::Error> for EmailServiceError {
    fn from(value: lettre::transport::smtp::Error) -> Self {
        EmailServiceError::Transport(value)
    }
}
