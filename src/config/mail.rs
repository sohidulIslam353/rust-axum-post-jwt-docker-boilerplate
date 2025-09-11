use anyhow::{Result, anyhow};
use lettre::transport::smtp::{AsyncSmtpTransport, authentication::Credentials};
use lettre::{AsyncTransport, Message, Tokio1Executor};

/// email sending client
pub struct EmailSender {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    from_email: String,
}

impl EmailSender {
    /// SMTP credentials initialize
    pub fn new() -> Result<Self> {
        let smtp_host = std::env::var("MAIL_HOST")?;
        let smtp_port: u16 = std::env::var("MAIL_PORT")?.parse()?;
        let smtp_user = std::env::var("MAIL_USERNAME")?;
        let smtp_pass = std::env::var("MAIL_PASSWORD")?;
        let from_email = std::env::var("MAIL_FROM")?;

        let creds = Credentials::new(smtp_user, smtp_pass);

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&smtp_host)
            .port(smtp_port)
            .credentials(creds)
            .build();

        Ok(Self { mailer, from_email })
    }

    /// anynchronux email send
    pub async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<()> {
        let email = Message::builder()
            .from(self.from_email.parse()?)
            .to(to.parse()?)
            .subject(subject)
            .body(body.to_owned())
            .map_err(|e| anyhow!("failed tos end the email: {}", e))?;

        self.mailer.send(email).await?;

        Ok(())
    }
}
