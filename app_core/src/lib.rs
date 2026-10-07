pub mod auth;
pub mod employees;
pub mod users;
pub mod events;
pub mod organizations;
pub mod groups;
pub mod config;
pub mod db;
pub mod postgres;
pub mod vacation_app_db;
pub mod api;
pub mod app_ext;
pub mod session_cookie;
pub mod email;
pub mod email_events;

pub use api::{AppError, PostgresVacationApp, VacationAppApi};
pub use app_ext::{
    AdminEmployeeDetail, EmployeePermissions, EmployeeProfile, GroupDetail, GroupMemberPermissions,
    MockPaymentDetails, VacationAppExt, VacationStats, VacationSummary,
};
pub use groups::GroupMembershipStore;
pub use session_cookie::{
    build_clear_cookie, build_set_cookie, read_optional_sessions, read_session_cookie,
    validate_cookie_session, DualSessions, SessionInfo, SessionKind, EMPLOYEE_SESSION_COOKIE,
    USER_SESSION_COOKIE,
};
pub use config::{
    load_config, load_config_from, load_prototype_config, load_prototype_config_from, AuthSettings,
    DatabaseConfig, EmailSettings, EmailTlsMode, PostgresSettings, PrototypeSettings, ServerSettings,
    VacationDbSettings, VactionAppConfigs,
};
pub use email::{
    EmailAddress, EmailBody, EmailConnector, EmailConnectorConfig, EmailErrors, EmailMessage,
    EmailMessageBuilder, LettreSmtpConnector, LettreSmtpMailer, Mailer, StubMailer,
};
pub use email_events::{
    event_sends_email, messages_for_event, password_changed_message, EmployeeContact,
    EventEmailContext,
};

use tokio_postgres::{Client, NoTls};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommonErrors {
    DBNotFound,
    UserNotFound,
    EmployeeNotFound,
    SessionNotFound,
    NotImplemented,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    year: u8,
    month: u8,
    day: u8,
}

impl Date {
    pub fn new(year: u8, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    pub fn year(&self) -> u8 {
        self.year
    }

    pub fn month(&self) -> u8 {
        self.month
    }

    pub fn day(&self) -> u8 {
        self.day
    }
}

pub async fn createDBclient(config: &VactionAppConfigs) -> Result<Client, CommonErrors> {
    createDBclient_from_url(config.database_url()).await
}

pub async fn createDBclient_from_url(database_url: &str) -> Result<Client, CommonErrors> {
    if database_url.is_empty() {
        return Err(CommonErrors::DBNotFound);
    }

    let (client, connection) = tokio_postgres::connect(database_url, NoTls)
        .await
        .map_err(|_| CommonErrors::DBNotFound)?;

    tokio::spawn(async move {
        let _ = connection.await;
    });

    Ok(client)
}

pub async fn checkDBs(_config: &VactionAppConfigs, client: &Client) -> Result<(), CommonErrors> {
    auth::Session::check_db_or_create(client).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn missing_database_url_returns_db_not_found() {
        let config = VactionAppConfigs::default();
        let result = createDBclient(&config).await;
        assert_eq!(result.err(), Some(CommonErrors::DBNotFound));
    }
}
