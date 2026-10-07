use tokio_postgres::Client;

use crate::employees::EmployeeID;
use crate::users::UserID;
use crate::CommonErrors;

pub struct Session {
    sID: u32,
    sType: SessionType,
}

/// Temporary aliases until user/employee session types are split out.
pub type UserSession = Session;
pub type EmployeeSession = Session;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionType {
    User { userID: UserID },
    Employee { employeeID: EmployeeID },
}

#[derive(Debug, PartialEq, Eq)]
pub enum AuthError {
    DatabaseUnavailable,
    SessionNotFound,
    UserNotFound,
    EmployeeNotFound,
    InvalidCredentials,
    NotImplemented,
}

pub trait AuthConfig {
    fn database_url(&self) -> &str;
}

pub use crate::config::AuthSettings;

impl AuthConfig for crate::config::VactionAppConfigs {
    fn database_url(&self) -> &str {
        self.database.database_url()
    }
}

const SESSIONS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS sessions (
    "sID" SERIAL PRIMARY KEY,
    "UEid" INTEGER NOT NULL,
    "isUser" BOOLEAN NOT NULL
)
"#;

const USER_CREDENTIALS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS user_credentials (
    "userID" INTEGER PRIMARY KEY,
    "email" TEXT NOT NULL UNIQUE,
    "password" TEXT NOT NULL
)
"#;

const EMPLOYEE_CREDENTIALS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS employee_credentials (
    "employeeID" INTEGER PRIMARY KEY,
    "email" TEXT NOT NULL UNIQUE,
    "password" TEXT NOT NULL
)
"#;

impl From<CommonErrors> for AuthError {
    fn from(error: CommonErrors) -> Self {
        match error {
            CommonErrors::DBNotFound => AuthError::DatabaseUnavailable,
            CommonErrors::SessionNotFound => AuthError::SessionNotFound,
            CommonErrors::UserNotFound => AuthError::UserNotFound,
            CommonErrors::EmployeeNotFound => AuthError::EmployeeNotFound,
            CommonErrors::NotImplemented => AuthError::NotImplemented,
        }
    }
}

impl Session {
    pub fn id(&self) -> u32 {
        self.sID
    }

    pub fn session_type(&self) -> &SessionType {
        &self.sType
    }

    pub fn is_user(&self) -> bool {
        matches!(self.sType, SessionType::User { .. })
    }

    pub fn is_employee(&self) -> bool {
        matches!(self.sType, SessionType::Employee { .. })
    }

    /// Builds a session value for non-Postgres backends (e.g. prototype demo).
    pub fn from_parts(session_id: u32, session_type: SessionType) -> Self {
        Self {
            sID: session_id,
            sType: session_type,
        }
    }

    /// Checks that the client can talk to Postgres and that the `sessions` table exists.
    pub async fn check_db(client: &Client) -> Result<(), CommonErrors> {
        client
            .batch_execute("SELECT 1")
            .await
            .map_err(|_| CommonErrors::DBNotFound)?;

        let rows = client
            .query(
                r#"
                SELECT EXISTS (
                    SELECT FROM information_schema.columns
                    WHERE table_schema = 'public'
                      AND table_name = 'sessions'
                      AND column_name = 'isUser'
                )
                "#,
                &[],
            )
            .await
            .map_err(|_| CommonErrors::DBNotFound)?;

        let exists: bool = rows
            .first()
            .map(|row| row.get(0))
            .ok_or(CommonErrors::DBNotFound)?;

        if exists {
            Ok(())
        } else {
            Err(CommonErrors::DBNotFound)
        }
    }

    /// Ensures auth tables exist, creating them when missing.
    pub async fn check_db_or_create(client: &Client) -> Result<(), CommonErrors> {
        match Self::check_db(client).await {
            Ok(()) => Ok(()),
            Err(CommonErrors::DBNotFound) => {
                client
                    .batch_execute("DROP TABLE IF EXISTS sessions")
                    .await
                    .map_err(|_| CommonErrors::DBNotFound)?;
                client
                    .batch_execute(SESSIONS_TABLE_SQL)
                    .await
                    .map_err(|_| CommonErrors::DBNotFound)
            }
            Err(err) => Err(err),
        }
    }

    /// Reads a session by id.
    pub async fn read_session(session_id: u32, client: &Client) -> Result<Self, CommonErrors> {
        let row = client
            .query_opt(
                r#"SELECT "sID", "UEid", "isUser" FROM sessions WHERE "sID" = $1"#,
                &[&(session_id as i32)],
            )
            .await
            .map_err(|_| CommonErrors::DBNotFound)?
            .ok_or(CommonErrors::SessionNotFound)?;

        row_to_session(&row)
    }

    /// Writes a new session for a user or employee.
    pub async fn write_session(
        session_type: SessionType,
        client: &Client,
    ) -> Result<Self, CommonErrors> {
        let (ue_id, is_user) = session_parts(&session_type);

        let row = client
            .query_one(
                r#"
                INSERT INTO sessions ("UEid", "isUser")
                VALUES ($1, $2)
                RETURNING "sID", "UEid", "isUser"
                "#,
                &[&ue_id, &is_user],
            )
            .await
            .map_err(|_| CommonErrors::DBNotFound)?;

        row_to_session(&row)
    }

    /// Updates an existing session's linked user or employee.
    pub async fn update_session(
        session_id: u32,
        session_type: SessionType,
        client: &Client,
    ) -> Result<Self, CommonErrors> {
        let (ue_id, is_user) = session_parts(&session_type);

        let row = client
            .query_opt(
                r#"
                UPDATE sessions
                SET "UEid" = $2, "isUser" = $3
                WHERE "sID" = $1
                RETURNING "sID", "UEid", "isUser"
                "#,
                &[&(session_id as i32), &ue_id, &is_user],
            )
            .await
            .map_err(|_| CommonErrors::DBNotFound)?
            .ok_or(CommonErrors::SessionNotFound)?;

        row_to_session(&row)
    }

    /// Deletes a session by id.
    pub async fn delete_session(session_id: u32, client: &Client) -> Result<(), CommonErrors> {
        let deleted = client
            .execute(
                r#"DELETE FROM sessions WHERE "sID" = $1"#,
                &[&(session_id as i32)],
            )
            .await
            .map_err(|_| CommonErrors::DBNotFound)?;

        if deleted == 0 {
            Err(CommonErrors::SessionNotFound)
        } else {
            Ok(())
        }
    }

    /// Reads all sessions linked to a user id.
    pub async fn read_sessions_for_user(
        user_id: UserID,
        client: &Client,
    ) -> Result<Vec<Self>, CommonErrors> {
        let rows = client
            .query(
                r#"
                SELECT "sID", "UEid", "isUser"
                FROM sessions
                WHERE "UEid" = $1 AND "isUser" = TRUE
                ORDER BY "sID"
                "#,
                &[&(user_id.id() as i32)],
            )
            .await
            .map_err(|_| CommonErrors::DBNotFound)?;

        rows.iter().map(row_to_session).collect()
    }

    /// Reads all sessions linked to an employee id.
    pub async fn read_sessions_for_employee(
        employee_id: EmployeeID,
        client: &Client,
    ) -> Result<Vec<Self>, CommonErrors> {
        let rows = client
            .query(
                r#"
                SELECT "sID", "UEid", "isUser"
                FROM sessions
                WHERE "UEid" = $1 AND "isUser" = FALSE
                ORDER BY "sID"
                "#,
                &[&(employee_id.id() as i32)],
            )
            .await
            .map_err(|_| CommonErrors::DBNotFound)?;

        rows.iter().map(row_to_session).collect()
    }

    /// Backward-compatible alias for [`Self::read_session`].
    pub async fn getSession(sID: u32, client: &Client) -> Result<Self, CommonErrors> {
        Self::read_session(sID, client).await
    }

    /// Backward-compatible alias for [`Self::write_session`].
    pub async fn createSession(sType: SessionType, client: &Client) -> Result<Self, CommonErrors> {
        Self::write_session(sType, client).await
    }

    /// Backward-compatible alias for [`Self::delete_session`].
    pub async fn deleteSession(sID: u32, client: &Client) -> Result<(), CommonErrors> {
        Self::delete_session(sID, client).await
    }

    /// Backward-compatible alias for [`Self::check_db`].
    pub async fn checkDB(client: &Client) -> Result<(), CommonErrors> {
        Self::check_db(client).await
    }

    /// Backward-compatible alias for [`Self::check_db_or_create`].
    pub async fn checkDBorCreate(client: &Client) -> Result<(), CommonErrors> {
        Self::check_db_or_create(client).await
    }
}

pub async fn setup_auth(config: &impl AuthConfig) -> Result<(), AuthError> {
    let client = connect(config).await?;
    Session::check_db_or_create(&client)
        .await
        .map_err(AuthError::from)?;
    client
        .batch_execute(USER_CREDENTIALS_TABLE_SQL)
        .await
        .map_err(|_| AuthError::DatabaseUnavailable)?;
    client
        .batch_execute(EMPLOYEE_CREDENTIALS_TABLE_SQL)
        .await
        .map_err(|_| AuthError::DatabaseUnavailable)?;
    Ok(())
}

pub async fn read_session(
    config: &impl AuthConfig,
    session_id: u32,
) -> Result<Session, AuthError> {
    let client = connect(config).await?;
    Session::read_session(session_id, &client)
        .await
        .map_err(AuthError::from)
}

pub async fn write_user_session(
    config: &impl AuthConfig,
    user_id: UserID,
) -> Result<Session, AuthError> {
    let client = connect(config).await?;
    Session::write_session(SessionType::User { userID: user_id }, &client)
        .await
        .map_err(AuthError::from)
}

pub async fn write_employee_session(
    config: &impl AuthConfig,
    employee_id: EmployeeID,
) -> Result<Session, AuthError> {
    let client = connect(config).await?;
    Session::write_session(
        SessionType::Employee {
            employeeID: employee_id,
        },
        &client,
    )
    .await
    .map_err(AuthError::from)
}

pub async fn update_session(
    config: &impl AuthConfig,
    session_id: u32,
    session_type: SessionType,
) -> Result<Session, AuthError> {
    let client = connect(config).await?;
    Session::update_session(session_id, session_type, &client)
        .await
        .map_err(AuthError::from)
}

pub async fn delete_session(config: &impl AuthConfig, session_id: u32) -> Result<(), AuthError> {
    let client = connect(config).await?;
    Session::delete_session(session_id, &client)
        .await
        .map_err(AuthError::from)
}

pub async fn list_user_sessions(
    config: &impl AuthConfig,
    user_id: UserID,
) -> Result<Vec<Session>, AuthError> {
    let client = connect(config).await?;
    Session::read_sessions_for_user(user_id, &client)
        .await
        .map_err(AuthError::from)
}

pub async fn list_employee_sessions(
    config: &impl AuthConfig,
    employee_id: EmployeeID,
) -> Result<Vec<Session>, AuthError> {
    let client = connect(config).await?;
    Session::read_sessions_for_employee(employee_id, &client)
        .await
        .map_err(AuthError::from)
}

pub async fn login_user(config: &impl AuthConfig, user_id: UserID) -> Result<Session, AuthError> {
    write_user_session(config, user_id).await
}

pub async fn login_employee(
    config: &impl AuthConfig,
    employee_id: EmployeeID,
) -> Result<Session, AuthError> {
    write_employee_session(config, employee_id).await
}

pub async fn logout(config: &impl AuthConfig, session_id: u32) -> Result<(), AuthError> {
    delete_session(config, session_id).await
}

pub async fn validate_session(
    config: &impl AuthConfig,
    session_id: u32,
) -> Result<Session, AuthError> {
    read_session(config, session_id).await
}

pub async fn write_user_credentials(
    config: &impl AuthConfig,
    user_id: UserID,
    email: &str,
    password: &str,
) -> Result<(), AuthError> {
    let client = connect(config).await?;
    client
        .execute(
            r#"
            INSERT INTO user_credentials ("userID", "email", "password")
            VALUES ($1, $2, $3)
            ON CONFLICT ("userID") DO UPDATE
            SET "email" = EXCLUDED."email", "password" = EXCLUDED."password"
            "#,
            &[&(user_id.id() as i32), &email, &password],
        )
        .await
        .map_err(|_| AuthError::DatabaseUnavailable)?;
    Ok(())
}

pub async fn write_employee_credentials(
    config: &impl AuthConfig,
    employee_id: EmployeeID,
    email: &str,
    password: &str,
) -> Result<(), AuthError> {
    let client = connect(config).await?;
    client
        .execute(
            r#"
            INSERT INTO employee_credentials ("employeeID", "email", "password")
            VALUES ($1, $2, $3)
            ON CONFLICT ("employeeID") DO UPDATE
            SET "email" = EXCLUDED."email", "password" = EXCLUDED."password"
            "#,
            &[&(employee_id.id() as i32), &email, &password],
        )
        .await
        .map_err(|_| AuthError::DatabaseUnavailable)?;
    Ok(())
}

/// Updates login email, and password only when `password` is provided.
pub async fn update_employee_credentials(
    config: &impl AuthConfig,
    employee_id: EmployeeID,
    email: &str,
    password: Option<&str>,
) -> Result<(), AuthError> {
    let client = connect(config).await?;
    if let Some(password) = password {
        client
            .execute(
                r#"
                INSERT INTO employee_credentials ("employeeID", "email", "password")
                VALUES ($1, $2, $3)
                ON CONFLICT ("employeeID") DO UPDATE
                SET "email" = EXCLUDED."email", "password" = EXCLUDED."password"
                "#,
                &[&(employee_id.id() as i32), &email, &password],
            )
            .await
            .map_err(|_| AuthError::DatabaseUnavailable)?;
    } else {
        let updated = client
            .execute(
                r#"
                UPDATE employee_credentials
                SET "email" = $1
                WHERE "employeeID" = $2
                "#,
                &[&email, &(employee_id.id() as i32)],
            )
            .await
            .map_err(|_| AuthError::DatabaseUnavailable)?;
        if updated == 0 {
            return Err(AuthError::InvalidCredentials);
        }
    }
    Ok(())
}

pub async fn authenticate_user(
    config: &impl AuthConfig,
    email: &str,
    password: &str,
) -> Result<Session, AuthError> {
    let client = connect(config).await?;
    let row = client
        .query_opt(
            r#"
            SELECT "userID", "password"
            FROM user_credentials
            WHERE "email" = $1
            "#,
            &[&email],
        )
        .await
        .map_err(|_| AuthError::DatabaseUnavailable)?
        .ok_or(AuthError::InvalidCredentials)?;

    let user_id: i32 = row.get(0);
    let stored_password: String = row.get(1);
    if stored_password != password {
        return Err(AuthError::InvalidCredentials);
    }

    Session::write_session(
        SessionType::User {
            userID: UserID::new(user_id as u32),
        },
        &client,
    )
    .await
    .map_err(AuthError::from)
}

pub async fn authenticate_employee(
    config: &impl AuthConfig,
    email: &str,
    password: &str,
) -> Result<Session, AuthError> {
    let client = connect(config).await?;
    let row = client
        .query_opt(
            r#"
            SELECT "employeeID", "password"
            FROM employee_credentials
            WHERE "email" = $1
            "#,
            &[&email],
        )
        .await
        .map_err(|_| AuthError::DatabaseUnavailable)?
        .ok_or(AuthError::InvalidCredentials)?;

    let employee_id: i32 = row.get(0);
    let stored_password: String = row.get(1);
    if stored_password != password {
        return Err(AuthError::InvalidCredentials);
    }

    Session::write_session(
        SessionType::Employee {
            employeeID: EmployeeID::new(employee_id as u32),
        },
        &client,
    )
    .await
    .map_err(AuthError::from)
}

async fn connect(config: &impl AuthConfig) -> Result<Client, AuthError> {
    if config.database_url().is_empty() {
        return Err(AuthError::DatabaseUnavailable);
    }

    crate::createDBclient_from_url(config.database_url())
        .await
        .map_err(AuthError::from)
}

fn session_parts(session_type: &SessionType) -> (i32, bool) {
    match session_type {
        SessionType::User { userID } => (userID.id() as i32, true),
        SessionType::Employee { employeeID } => (employeeID.id() as i32, false),
    }
}

fn row_to_session(row: &tokio_postgres::Row) -> Result<Session, CommonErrors> {
    let session_id: i32 = row.get(0);
    let ue_id: i32 = row.get(1);
    let is_user: bool = row.get(2);

    let session_type = if is_user {
        SessionType::User {
            userID: UserID::new(ue_id as u32),
        }
    } else {
        SessionType::Employee {
            employeeID: EmployeeID::new(ue_id as u32),
        }
    };

    Ok(Session {
        sID: session_id as u32,
        sType: session_type,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::createDBclient;
    use crate::config::{load_config_from, VactionAppConfigs};
    use std::path::PathBuf;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    static DB_LOCK: Mutex<()> = Mutex::const_new(());

    fn workspace_config_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("server_configs.toml")
    }

    async fn connect_test_config() -> Option<VactionAppConfigs> {
        match load_config_from(workspace_config_path()) {
            Ok(config) => Some(config),
            Err(err) => {
                eprintln!("skipping auth DB tests: could not load config ({err:?})");
                None
            }
        }
    }

    async fn connect_test_client() -> Option<Client> {
        let config = connect_test_config().await?;
        match createDBclient(&config).await {
            Ok(client) => Some(client),
            Err(err) => {
                eprintln!(
                    "skipping auth DB tests: set database_url in server_configs.toml ({err:?})"
                );
                None
            }
        }
    }

    async fn with_db_test<F, Fut>(test: F)
    where
        F: FnOnce(Arc<Client>, VactionAppConfigs) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let _guard = DB_LOCK.lock().await;
        let Some(config) = connect_test_config().await else {
            return;
        };
        let Some(client) = connect_test_client().await else {
            return;
        };
        let client = Arc::new(client);

        client
            .batch_execute("BEGIN")
            .await
            .expect("begin transaction");

        test(Arc::clone(&client), config).await;

        client
            .batch_execute("ROLLBACK")
            .await
            .expect("rollback transaction");
    }

    #[tokio::test]
    async fn check_db_fails_when_sessions_table_missing() {
        with_db_test(|client, _| async move {
            client
                .batch_execute("DROP TABLE IF EXISTS sessions")
                .await
                .expect("drop sessions");

            let result = Session::check_db(&client).await;
            assert_eq!(result, Err(CommonErrors::DBNotFound));
        })
        .await;
    }

    #[tokio::test]
    async fn check_db_or_create_creates_sessions_table() {
        with_db_test(|client, _| async move {
            client
                .batch_execute("DROP TABLE IF EXISTS sessions")
                .await
                .expect("drop sessions");

            Session::check_db_or_create(&client)
                .await
                .expect("create sessions table");

            Session::check_db(&client)
                .await
                .expect("sessions table should exist");
        })
        .await;
    }

    #[tokio::test]
    async fn write_and_read_user_session() {
        with_db_test(|client, _| async move {
            Session::check_db_or_create(&client).await.expect("ensure table");

            let created = Session::write_session(
                SessionType::User {
                    userID: UserID::new(42),
                },
                &client,
            )
            .await
            .expect("write user session");

            assert_ne!(created.id(), 0);
            assert!(created.is_user());

            let loaded = Session::read_session(created.id(), &client)
                .await
                .expect("read session");
            assert_eq!(loaded.id(), created.id());
        })
        .await;
    }

    #[tokio::test]
    async fn update_session_changes_linked_user() {
        with_db_test(|client, _| async move {
            Session::check_db_or_create(&client).await.expect("ensure table");

            let created = Session::write_session(
                SessionType::User {
                    userID: UserID::new(1),
                },
                &client,
            )
            .await
            .expect("write session");

            let updated = Session::update_session(
                created.id(),
                SessionType::User {
                    userID: UserID::new(2),
                },
                &client,
            )
            .await
            .expect("update session");

            match updated.session_type() {
                SessionType::User { userID } => assert_eq!(userID.id(), 2),
                SessionType::Employee { .. } => panic!("expected user session"),
            }
        })
        .await;
    }

    #[tokio::test]
    async fn delete_session_removes_existing_row() {
        with_db_test(|client, _| async move {
            Session::check_db_or_create(&client).await.expect("ensure table");

            let created = Session::write_session(
                SessionType::User {
                    userID: UserID::new(1),
                },
                &client,
            )
            .await
            .expect("write session");

            Session::delete_session(created.id(), &client)
                .await
                .expect("delete session");

            let result = Session::read_session(created.id(), &client).await;
            assert_eq!(result.err(), Some(CommonErrors::SessionNotFound));
        })
        .await;
    }

    #[tokio::test]
    async fn auth_helpers_use_config() {
        with_db_test(|_, config| async move {
            setup_auth(&config).await.expect("setup auth");

            let session = login_user(&config, UserID::new(99))
                .await
                .expect("login user");
            assert!(session.is_user());

            let validated = validate_session(&config, session.id())
                .await
                .expect("validate session");
            assert_eq!(validated.id(), session.id());

            logout(&config, session.id()).await.expect("logout");
        })
        .await;
    }

    #[tokio::test]
    async fn authenticate_user_uses_credentials() {
        with_db_test(|_, config| async move {
            setup_auth(&config).await.expect("setup auth");
            write_user_credentials(&config, UserID::new(5), "user@test.com", "secret")
                .await
                .expect("write credentials");

            let session = authenticate_user(&config, "user@test.com", "secret")
                .await
                .expect("authenticate user");
            match session.session_type() {
                SessionType::User { userID } => assert_eq!(userID.id(), 5),
                SessionType::Employee { .. } => panic!("expected user session"),
            }
        })
        .await;
    }
}
