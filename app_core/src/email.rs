//! Modular, reusable email interface.
//!
//! Mirrors the layering used by [`crate::config`] and [`crate::db`]:
//! - shared types + connector config
//! - a transport-agnostic [`Mailer`] trait
//! - concrete backends (Lettre SMTP, in-memory stub for tests)

use std::fmt;
use std::sync::{Arc, Mutex};

use lettre::message::{Mailbox, MessageBuilder, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

pub use crate::config::{EmailSettings, EmailTlsMode};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmailErrors {
    InvalidAddress(String),
    InvalidMessage(String),
    InvalidConfig(String),
    ConnectionFailed(String),
    SendFailed(String),
}

impl fmt::Display for EmailErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAddress(msg) => write!(f, "invalid email address: {msg}"),
            Self::InvalidMessage(msg) => write!(f, "invalid email message: {msg}"),
            Self::InvalidConfig(msg) => write!(f, "invalid email config: {msg}"),
            Self::ConnectionFailed(msg) => write!(f, "email connection failed: {msg}"),
            Self::SendFailed(msg) => write!(f, "email send failed: {msg}"),
        }
    }
}

impl std::error::Error for EmailErrors {}

// ---------------------------------------------------------------------------
// Config / connector settings
// ---------------------------------------------------------------------------

/// Backend-agnostic connector settings (analogous to [`crate::db::DBConnectorConfig`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailConnectorConfig {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub tls: EmailTlsMode,
}

impl EmailConnectorConfig {
    pub fn from_settings(settings: &EmailSettings) -> Self {
        Self {
            host: settings.smtp_host.clone(),
            port: settings.smtp_port,
            username: settings.username.clone(),
            password: settings.password.clone(),
            tls: settings.tls,
        }
    }
}

impl From<&EmailSettings> for EmailConnectorConfig {
    fn from(settings: &EmailSettings) -> Self {
        Self::from_settings(settings)
    }
}

// ---------------------------------------------------------------------------
// Message model (transport-agnostic)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailAddress {
    pub email: String,
    pub name: Option<String>,
}

impl EmailAddress {
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: None,
        }
    }

    pub fn with_name(email: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: Some(name.into()),
        }
    }

    fn to_mailbox(&self) -> Result<Mailbox, EmailErrors> {
        let parsed: Mailbox = match &self.name {
            Some(name) => format!("{name} <{}>", self.email)
                .parse()
                .map_err(|err| EmailErrors::InvalidAddress(format!("{err}")))?,
            None => self
                .email
                .parse()
                .map_err(|err| EmailErrors::InvalidAddress(format!("{err}")))?,
        };
        Ok(parsed)
    }
}

impl From<&str> for EmailAddress {
    fn from(email: &str) -> Self {
        Self::new(email)
    }
}

impl From<String> for EmailAddress {
    fn from(email: String) -> Self {
        Self::new(email)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmailBody {
    Text(String),
    Html(String),
    Both { text: String, html: String },
}

/// Transport-agnostic email payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailMessage {
    pub from: EmailAddress,
    pub to: Vec<EmailAddress>,
    pub cc: Vec<EmailAddress>,
    pub bcc: Vec<EmailAddress>,
    pub reply_to: Option<EmailAddress>,
    pub subject: String,
    pub body: EmailBody,
}

impl EmailMessage {
    pub fn builder() -> EmailMessageBuilder {
        EmailMessageBuilder::default()
    }

    /// Builds a message using the configured default `from` address.
    pub fn from_settings(settings: &EmailSettings) -> EmailMessageBuilder {
        let from = match &settings.from_name {
            Some(name) => EmailAddress::with_name(&settings.from_address, name),
            None => EmailAddress::new(&settings.from_address),
        };
        EmailMessageBuilder::default().from(from)
    }
}

#[derive(Debug, Clone, Default)]
pub struct EmailMessageBuilder {
    from: Option<EmailAddress>,
    to: Vec<EmailAddress>,
    cc: Vec<EmailAddress>,
    bcc: Vec<EmailAddress>,
    reply_to: Option<EmailAddress>,
    subject: Option<String>,
    body: Option<EmailBody>,
}

impl EmailMessageBuilder {
    pub fn from(mut self, address: impl Into<EmailAddress>) -> Self {
        self.from = Some(address.into());
        self
    }

    pub fn to(mut self, address: impl Into<EmailAddress>) -> Self {
        self.to.push(address.into());
        self
    }

    pub fn cc(mut self, address: impl Into<EmailAddress>) -> Self {
        self.cc.push(address.into());
        self
    }

    pub fn bcc(mut self, address: impl Into<EmailAddress>) -> Self {
        self.bcc.push(address.into());
        self
    }

    pub fn reply_to(mut self, address: impl Into<EmailAddress>) -> Self {
        self.reply_to = Some(address.into());
        self
    }

    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn text_body(mut self, text: impl Into<String>) -> Self {
        self.body = Some(EmailBody::Text(text.into()));
        self
    }

    pub fn html_body(mut self, html: impl Into<String>) -> Self {
        self.body = Some(EmailBody::Html(html.into()));
        self
    }

    pub fn both_bodies(mut self, text: impl Into<String>, html: impl Into<String>) -> Self {
        self.body = Some(EmailBody::Both {
            text: text.into(),
            html: html.into(),
        });
        self
    }

    pub fn body(mut self, body: EmailBody) -> Self {
        self.body = Some(body);
        self
    }

    pub fn build(self) -> Result<EmailMessage, EmailErrors> {
        let from = self
            .from
            .ok_or_else(|| EmailErrors::InvalidMessage("from address is required".into()))?;
        if self.to.is_empty() {
            return Err(EmailErrors::InvalidMessage(
                "at least one recipient is required".into(),
            ));
        }
        let subject = self
            .subject
            .ok_or_else(|| EmailErrors::InvalidMessage("subject is required".into()))?;
        let body = self
            .body
            .ok_or_else(|| EmailErrors::InvalidMessage("body is required".into()))?;

        Ok(EmailMessage {
            from,
            to: self.to,
            cc: self.cc,
            bcc: self.bcc,
            reply_to: self.reply_to,
            subject,
            body,
        })
    }
}

// ---------------------------------------------------------------------------
// Traits (modular interface)
// ---------------------------------------------------------------------------

/// Sends composed [`EmailMessage`] values.
///
/// Concrete backends implement this trait (Lettre SMTP, stub, etc.).
pub trait Mailer: Send + Sync {
    fn send(
        &self,
        message: &EmailMessage,
    ) -> impl std::future::Future<Output = Result<(), EmailErrors>> + Send;
}

/// Connects a mailer from connector settings (analogous to [`crate::db::DBConnector`]).
pub trait EmailConnector {
    type Mailer: Mailer;

    fn connect(cfg: EmailConnectorConfig) -> Result<Self::Mailer, EmailErrors>;
}

// ---------------------------------------------------------------------------
// Lettre SMTP backend
// ---------------------------------------------------------------------------

/// Lettre-backed async SMTP mailer.
#[derive(Clone)]
pub struct LettreSmtpMailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
}

impl fmt::Debug for LettreSmtpMailer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LettreSmtpMailer").finish_non_exhaustive()
    }
}

/// Connector that builds a [`LettreSmtpMailer`] (analogous to `PostgresConnector`).
pub struct LettreSmtpConnector;

impl EmailConnector for LettreSmtpConnector {
    type Mailer = LettreSmtpMailer;

    fn connect(cfg: EmailConnectorConfig) -> Result<Self::Mailer, EmailErrors> {
        LettreSmtpMailer::connect(cfg)
    }
}

impl LettreSmtpMailer {
    pub fn connect(cfg: EmailConnectorConfig) -> Result<Self, EmailErrors> {
        if cfg.host.is_empty() {
            return Err(EmailErrors::InvalidConfig(
                "smtp host must not be empty".into(),
            ));
        }

        let mut builder = match cfg.tls {
            EmailTlsMode::None => AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&cfg.host)
                .port(cfg.port),
            EmailTlsMode::StartTls => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&cfg.host)
                .map_err(|err| EmailErrors::ConnectionFailed(format!("{err}")))?
                .port(cfg.port),
            EmailTlsMode::Tls => AsyncSmtpTransport::<Tokio1Executor>::relay(&cfg.host)
                .map_err(|err| EmailErrors::ConnectionFailed(format!("{err}")))?
                .port(cfg.port),
        };

        if let (Some(username), Some(password)) = (&cfg.username, &cfg.password) {
            builder = builder.credentials(Credentials::new(username.clone(), password.clone()));
        } else if cfg.username.is_some() ^ cfg.password.is_some() {
            return Err(EmailErrors::InvalidConfig(
                "username and password must both be set or both omitted".into(),
            ));
        }

        Ok(Self {
            transport: builder.build(),
        })
    }

    pub fn from_settings(settings: &EmailSettings) -> Result<Self, EmailErrors> {
        Self::connect(EmailConnectorConfig::from_settings(settings))
    }

    fn build_lettre_message(message: &EmailMessage) -> Result<Message, EmailErrors> {
        let mut builder: MessageBuilder = Message::builder().from(message.from.to_mailbox()?);

        for recipient in &message.to {
            builder = builder.to(recipient.to_mailbox()?);
        }
        for recipient in &message.cc {
            builder = builder.cc(recipient.to_mailbox()?);
        }
        for recipient in &message.bcc {
            builder = builder.bcc(recipient.to_mailbox()?);
        }
        if let Some(reply_to) = &message.reply_to {
            builder = builder.reply_to(reply_to.to_mailbox()?);
        }

        builder = builder.subject(&message.subject);

        let built = match &message.body {
            EmailBody::Text(text) => builder
                .singlepart(SinglePart::plain(text.clone()))
                .map_err(|err| EmailErrors::InvalidMessage(format!("{err}")))?,
            EmailBody::Html(html) => builder
                .singlepart(SinglePart::html(html.clone()))
                .map_err(|err| EmailErrors::InvalidMessage(format!("{err}")))?,
            EmailBody::Both { text, html } => builder
                .multipart(
                    MultiPart::alternative()
                        .singlepart(SinglePart::plain(text.clone()))
                        .singlepart(SinglePart::html(html.clone())),
                )
                .map_err(|err| EmailErrors::InvalidMessage(format!("{err}")))?,
        };

        Ok(built)
    }
}

impl Mailer for LettreSmtpMailer {
    async fn send(&self, message: &EmailMessage) -> Result<(), EmailErrors> {
        let lettre_message = Self::build_lettre_message(message)?;
        self.transport
            .send(lettre_message)
            .await
            .map_err(|err| EmailErrors::SendFailed(format!("{err}")))?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Stub backend (tests / local dry-runs)
// ---------------------------------------------------------------------------

/// In-memory mailer that records messages instead of sending them.
#[derive(Clone, Default)]
pub struct StubMailer {
    sent: Arc<Mutex<Vec<EmailMessage>>>,
}

impl StubMailer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sent(&self) -> Vec<EmailMessage> {
        self.sent
            .lock()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    pub fn clear(&self) {
        if let Ok(mut guard) = self.sent.lock() {
            guard.clear();
        }
    }
}

pub struct StubEmailConnector;

impl EmailConnector for StubEmailConnector {
    type Mailer = StubMailer;

    fn connect(_cfg: EmailConnectorConfig) -> Result<Self::Mailer, EmailErrors> {
        Ok(StubMailer::new())
    }
}

impl Mailer for StubMailer {
    async fn send(&self, message: &EmailMessage) -> Result<(), EmailErrors> {
        let mut guard = self
            .sent
            .lock()
            .map_err(|_| EmailErrors::SendFailed("stub mailer lock poisoned".into()))?;
        guard.push(message.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_builder_requires_from_to_subject_body() {
        let err = EmailMessage::builder()
            .to("a@example.com")
            .subject("Hi")
            .text_body("hello")
            .build()
            .unwrap_err();
        assert!(matches!(err, EmailErrors::InvalidMessage(_)));

        let msg = EmailMessage::builder()
            .from("noreply@example.com")
            .to("a@example.com")
            .subject("Hi")
            .text_body("hello")
            .build()
            .unwrap();

        assert_eq!(msg.from.email, "noreply@example.com");
        assert_eq!(msg.to.len(), 1);
        assert_eq!(msg.subject, "Hi");
    }

    #[test]
    fn connector_config_maps_from_settings() {
        let settings = EmailSettings {
            smtp_host: "smtp.example.com".into(),
            smtp_port: 587,
            username: Some("user".into()),
            password: Some("pass".into()),
            from_address: "noreply@example.com".into(),
            from_name: Some("Vaction".into()),
            tls: EmailTlsMode::StartTls,
        };

        let cfg = EmailConnectorConfig::from_settings(&settings);
        assert_eq!(cfg.host, "smtp.example.com");
        assert_eq!(cfg.port, 587);
        assert_eq!(cfg.username.as_deref(), Some("user"));
        assert_eq!(cfg.tls, EmailTlsMode::StartTls);
    }

    #[test]
    fn lettre_mailer_rejects_empty_host() {
        let err = LettreSmtpMailer::connect(EmailConnectorConfig {
            host: String::new(),
            port: 587,
            username: None,
            password: None,
            tls: EmailTlsMode::None,
        })
        .unwrap_err();
        assert!(matches!(err, EmailErrors::InvalidConfig(_)));
    }

    #[test]
    fn lettre_builds_plain_message() {
        let message = EmailMessage::builder()
            .from(EmailAddress::with_name("noreply@example.com", "App"))
            .to("user@example.com")
            .cc("ops@example.com")
            .subject("Welcome")
            .both_bodies("plain", "<p>html</p>")
            .build()
            .unwrap();

        let built = LettreSmtpMailer::build_lettre_message(&message).unwrap();
        assert!(built.headers().get::<lettre::message::header::Subject>().is_some());
    }

    #[tokio::test]
    async fn stub_mailer_records_sent_messages() {
        let mailer = StubEmailConnector::connect(EmailConnectorConfig {
            host: "unused".into(),
            port: 25,
            username: None,
            password: None,
            tls: EmailTlsMode::None,
        })
        .unwrap();

        let message = EmailMessage::builder()
            .from("noreply@example.com")
            .to("user@example.com")
            .subject("Ping")
            .text_body("pong")
            .build()
            .unwrap();

        mailer.send(&message).await.unwrap();
        assert_eq!(mailer.sent().len(), 1);
        assert_eq!(mailer.sent()[0].subject, "Ping");
    }
}
