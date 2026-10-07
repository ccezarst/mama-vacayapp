use std::collections::HashMap;

use tokio_postgres::{Client, NoTls, Row};

use crate::db::{DBConnectorConfig, DBQuery, DBQueryOps, DBRow, DBTable, Db};

pub use crate::config::PostgresSettings;

#[derive(Debug, PartialEq, Eq)]
pub enum PostgresError {
    ConnectionFailed,
    QueryFailed,
    DatabaseNotFound,
    InvalidConfig,
    InvalidConnectionUrl,
    UnsupportedQueryOp,
    /// Connected, but the role cannot `CREATE DATABASE`.
    CreateDatabaseDenied,
}

pub struct PostgresDbMarker;

impl Db<SessionsRow, SessionsTable> for PostgresDbMarker {}

/// Connects to PostgreSQL and exposes database-level query targets.
pub struct PostgresServer {
    client: Client,
    connection_root: ConnectionRoot,
}

#[derive(Debug, Clone)]
struct ConnectionRoot {
    url: Option<String>,
    address: String,
    port: u16,
    auth: HashMap<String, String>,
    default_database: String,
}

/// Handle to one PostgreSQL database.
pub struct PostgresDatabase {
    client: Client,
    name: String,
}

crate::impl_db_query! {
    on: PostgresServer,
    db_name,
}

crate::impl_db_query! {
    on: PostgresDatabase,
    table_name,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionsRow;

impl DBRow for SessionsRow {}

pub struct SessionsTable;

impl DBTable<SessionsRow> for SessionsTable {
    const TABLE_NAME: &'static str = "sessions";
}

crate::impl_db_query! {
    on: SessionsTable,
    row_filter,
    schema,
}

pub struct PostgresConnector;

impl PostgresConnector {
    pub async fn connect(cfg: DBConnectorConfig) -> Result<PostgresServer, PostgresError> {
        PostgresServer::connect(cfg).await
    }

    pub async fn connect_url(url: &str) -> Result<PostgresServer, PostgresError> {
        PostgresServer::connect_url(url).await
    }

    /// Connects via the admin/maintenance database, ensures `db_name` exists, then opens it.
    pub async fn connect_url_ensuring_database(
        url: &str,
        db_name: &str,
    ) -> Result<PostgresDatabase, PostgresError> {
        let admin_url = replace_database_in_url(url, "postgres");
        let server = Self::connect_url(&admin_url).await?;
        server.ensure_database(db_name).await?;
        server
            .open(PostgresServer::query_db_name(DBQueryOps::Equal, db_name))
            .await
    }
}

impl PostgresServer {
    pub async fn connect(cfg: DBConnectorConfig) -> Result<Self, PostgresError> {
        let connection_root = ConnectionRoot::from_config(&cfg)?;
        let database = connection_root.default_database.clone();
        let url = connection_root.connection_string(&database);
        Self::connect_url(&url).await
    }

    pub async fn connect_url(url: &str) -> Result<Self, PostgresError> {
        let connection_root = ConnectionRoot::from_url(url)?;
        let (client, connection) = tokio_postgres::connect(url, NoTls)
            .await
            .map_err(|_| PostgresError::ConnectionFailed)?;

        tokio::spawn(async move {
            let _ = connection.await;
        });

        Ok(Self {
            client,
            connection_root,
        })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Opens a database from a server-level query such as
    /// `PostgresServer::query_db_name(DBQueryOps::Equal, "vacation_app")`.
    pub async fn open(
        &self,
        query: DBQuery<PostgresServer>,
    ) -> Result<PostgresDatabase, PostgresError> {
        if query.kind != "db_name" {
            return Err(PostgresError::QueryFailed);
        }

        let database = match query.op {
            DBQueryOps::Equal => query.filter.as_str(),
            _ => return Err(PostgresError::UnsupportedQueryOp),
        };

        let url = self.connection_root.connection_string(database);
        PostgresDatabase::connect_url(&url, database).await
    }

    /// Lists databases, optionally filtered by a server-level query.
    pub async fn list_databases(
        &self,
        query: DBQuery<PostgresServer>,
    ) -> Result<Vec<String>, PostgresError> {
        if query.kind != "db_name" {
            return Err(PostgresError::QueryFailed);
        }

        let rows = self
            .client
            .query(
                "SELECT datname FROM pg_database WHERE datallowconn ORDER BY datname",
                &[],
            )
            .await
            .map_err(|_| PostgresError::QueryFailed)?;

        let mut names: Vec<String> = rows.iter().map(|row| row.get::<_, String>(0)).collect();
        names.retain(|name| query_matches_string(name, query.op, &query.filter));
        Ok(names)
    }

    /// Creates `name` if it does not already exist on this server.
    pub async fn ensure_database(&self, name: &str) -> Result<(), PostgresError> {
        if name.is_empty() || name.contains('"') || name.contains(';') {
            return Err(PostgresError::InvalidConfig);
        }

        let existing = self
            .list_databases(Self::query_db_name(DBQueryOps::Equal, name))
            .await?;
        if !existing.is_empty() {
            return Ok(());
        }

        let sql = format!("CREATE DATABASE \"{name}\"");
        // Common failure here is missing CREATEDB privilege on the configured role.
        self.client
            .batch_execute(&sql)
            .await
            .map_err(|_| PostgresError::CreateDatabaseDenied)?;
        Ok(())
    }
}

impl PostgresDatabase {
    async fn connect_url(url: &str, name: &str) -> Result<Self, PostgresError> {
        let (client, connection) = tokio_postgres::connect(url, NoTls)
            .await
            .map_err(|_| PostgresError::ConnectionFailed)?;

        tokio::spawn(async move {
            let _ = connection.await;
        });

        Ok(Self {
            client,
            name: name.to_string(),
        })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Checks whether a table exists using a query such as
    /// `PostgresDatabase::query_table_name(DBQueryOps::Equal, "sessions")`.
    pub async fn table_exists(
        &self,
        query: DBQuery<PostgresDatabase>,
    ) -> Result<bool, PostgresError> {
        if query.kind != "table_name" || query.op != DBQueryOps::Equal {
            return Err(PostgresError::UnsupportedQueryOp);
        }

        let rows = self
            .client
            .query(
                r#"
                SELECT EXISTS (
                    SELECT FROM information_schema.tables
                    WHERE table_schema = 'public'
                      AND table_name = $1
                )
                "#,
                &[&query.filter],
            )
            .await
            .map_err(|_| PostgresError::QueryFailed)?;

        rows.first()
            .map(|row| row.get::<_, bool>(0))
            .ok_or(PostgresError::QueryFailed)
    }

    /// Returns row count for a table query such as
    /// `PostgresDatabase::query_table_name(DBQueryOps::Equal, "sessions")`.
    pub async fn count_rows(
        &self,
        query: DBQuery<PostgresDatabase>,
    ) -> Result<i64, PostgresError> {
        if query.kind != "table_name" || query.op != DBQueryOps::Equal {
            return Err(PostgresError::UnsupportedQueryOp);
        }

        let sql = format!(r#"SELECT COUNT(*) FROM "{}""#, query.filter);
        let rows = self
            .client
            .query(&sql, &[])
            .await
            .map_err(|_| PostgresError::QueryFailed)?;

        rows.first()
            .map(|row| row.get::<_, i64>(0))
            .ok_or(PostgresError::QueryFailed)
    }

    /// Runs a table-scoped query against this database connection.
    pub async fn run_table_query(
        &self,
        query: DBQuery<SessionsTable>,
    ) -> Result<Vec<Row>, PostgresError> {
        SessionsTable::execute(self, query).await
    }
}

impl SessionsTable {
    pub async fn execute(
        db: &PostgresDatabase,
        query: DBQuery<SessionsTable>,
    ) -> Result<Vec<Row>, PostgresError> {
        match query.kind {
            "row_filter" => {
                let (column, value) = split_column_value(&query.filter)?;
                let sql = format!(
                    r#"SELECT "sID", "UEid", "isUser" FROM sessions WHERE "{}" {} $1"#,
                    column,
                    sql_op(query.op)?
                );

                db.client
                    .query(&sql, &[&value])
                    .await
                    .map_err(|_| PostgresError::QueryFailed)
            }
            "schema" => {
                if query.op != DBQueryOps::Equal {
                    return Err(PostgresError::UnsupportedQueryOp);
                }

                db.client
                    .query(
                        r#"
                        SELECT table_schema, table_name
                        FROM information_schema.tables
                        WHERE table_schema = $1
                        ORDER BY table_name
                        "#,
                        &[&query.filter],
                    )
                    .await
                    .map_err(|_| PostgresError::QueryFailed)
            }
            _ => Err(PostgresError::QueryFailed),
        }
    }
}

fn split_column_value(filter: &str) -> Result<(&str, &str), PostgresError> {
    filter
        .split_once('=')
        .filter(|(column, value)| !column.is_empty() && !value.is_empty())
        .ok_or(PostgresError::QueryFailed)
}

fn sql_op(op: DBQueryOps) -> Result<&'static str, PostgresError> {
    match op {
        DBQueryOps::Equal => Ok("="),
        DBQueryOps::NotEqual => Ok("<>"),
        DBQueryOps::BiggerThan => Ok(">"),
        DBQueryOps::SmallerThan => Ok("<"),
        DBQueryOps::BiggerOrEqual => Ok(">="),
        DBQueryOps::SmallerOrEqual => Ok("<="),
    }
}

fn query_matches_string(value: &str, op: DBQueryOps, filter: &str) -> bool {
    match op {
        DBQueryOps::Equal => value == filter,
        DBQueryOps::NotEqual => value != filter,
        DBQueryOps::BiggerThan => value > filter,
        DBQueryOps::SmallerThan => value < filter,
        DBQueryOps::BiggerOrEqual => value >= filter,
        DBQueryOps::SmallerOrEqual => value <= filter,
    }
}

impl ConnectionRoot {
    fn from_config(cfg: &DBConnectorConfig) -> Result<Self, PostgresError> {
        if let Some(url) = cfg.authDetails.get("database_url") {
            return Self::from_url(url);
        }

        let default_database = cfg
            .authDetails
            .get("database")
            .cloned()
            .unwrap_or_else(|| "postgres".to_string());

        Ok(Self {
            url: None,
            address: cfg.address.clone(),
            port: cfg.port,
            auth: cfg.authDetails.clone(),
            default_database,
        })
    }

    fn from_url(url: &str) -> Result<Self, PostgresError> {
        let default_database = extract_dbname(url).unwrap_or_else(|| "postgres".to_string());

        Ok(Self {
            url: Some(url.to_string()),
            address: String::new(),
            port: 5432,
            auth: HashMap::new(),
            default_database,
        })
    }

    fn connection_string(&self, database: &str) -> String {
        if let Some(url) = &self.url {
            replace_database_in_url(url, database)
        } else {
            let user = self
                .auth
                .get("user")
                .map(String::as_str)
                .unwrap_or("postgres");
            let password = self.auth.get("password").map(String::as_str).unwrap_or("");
            format!(
                "postgresql://{}:{}@{}:{}/{}",
                user, password, self.address, self.port, database
            )
        }
    }
}

fn extract_dbname(url: &str) -> Option<String> {
    if let Some(value) = url
        .split_whitespace()
        .find_map(|part| part.strip_prefix("dbname="))
    {
        return Some(value.to_string());
    }

    url.rsplit('/')
        .next()
        .filter(|part| !part.is_empty() && !part.contains('@') && !part.contains('='))
        .map(|part| part.split('?').next().unwrap_or(part).to_string())
}

fn replace_database_in_url(url: &str, database: &str) -> String {
    if url.split_whitespace().any(|part| part.starts_with("dbname=")) {
        return url
            .split_whitespace()
            .map(|part| {
                if part.starts_with("dbname=") {
                    format!("dbname={database}")
                } else {
                    part.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
    }

    let (base, suffix) = match url.find('?') {
        Some(query_start) => (&url[..query_start], &url[query_start..]),
        None => (url, ""),
    };

    match base.rfind('/') {
        Some(slash) => format!("{}{}{}", &base[..=slash], database, suffix),
        None => format!("{}/{}{}", base, database, suffix),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DBConnectorConfig;

    #[test]
    fn server_query_stores_db_name_filter() {
        let q = PostgresServer::query_db_name(DBQueryOps::Equal, "vacation_app");
        assert_eq!(q.kind, "db_name");
        assert_eq!(q.op, DBQueryOps::Equal);
        assert_eq!(q.filter, "vacation_app");
    }

    #[test]
    fn database_query_stores_table_name_filter() {
        let q = PostgresDatabase::query_table_name(DBQueryOps::Equal, "sessions");
        assert_eq!(q.kind, "table_name");
        assert_eq!(q.filter, "sessions");
    }

    #[test]
    fn table_query_stores_schema_filter() {
        let q = SessionsTable::query_schema(DBQueryOps::Equal, "public");
        assert_eq!(q.kind, "schema");
        assert_eq!(q.filter, "public");
    }

    #[test]
    fn replace_database_in_url_swaps_path_component() {
        let url = "postgresql://user:pass@localhost:5432/olddb?sslmode=disable";
        assert_eq!(
            replace_database_in_url(url, "vacation_app"),
            "postgresql://user:pass@localhost:5432/vacation_app?sslmode=disable"
        );
    }

    #[test]
    fn replace_database_in_libpq_url_swaps_dbname() {
        let url = "host=127.0.0.1 port=5432 user=vaction password=secret dbname=vaction_app";
        assert_eq!(
            replace_database_in_url(url, "vaction_app_prototype"),
            "host=127.0.0.1 port=5432 user=vaction password=secret dbname=vaction_app_prototype"
        );
        assert_eq!(
            extract_dbname(url).as_deref(),
            Some("vaction_app")
        );
    }

    #[test]
    fn query_matches_string_honors_ops() {
        assert!(query_matches_string("vacation_app", DBQueryOps::Equal, "vacation_app"));
        assert!(!query_matches_string(
            "vacation_app",
            DBQueryOps::NotEqual,
            "vacation_app"
        ));
    }

    #[test]
    fn connection_root_from_config_builds_url_from_parts() {
        let cfg = DBConnectorConfig {
            authDetails: HashMap::new(),
            address: String::new(),
            port: 5432,
        };

        assert_eq!(
            ConnectionRoot::from_config(&cfg)
                .unwrap()
                .connection_string("postgres"),
            "postgresql://postgres:@:/5432/postgres"
        );
    }
}
