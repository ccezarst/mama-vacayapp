use std::fmt;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::{fs, io};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Debug)]
pub enum ConfigErrors{
    DeserializationError(DError),
    SerializationError(SError),
}

#[derive(Debug)]
pub enum DError{
    FileNotFound(String),
    FilePermissionDenied(String),
    Miscellanious(String)
}

#[derive(Debug)]
pub enum SError{
    UnsupportedType(String),
    PermissionDenied(String),
    KeyNotString(String),
    Miscellanious(String)
}

impl fmt::Display for ConfigErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeserializationError(err) => write!(f, "config deserialization error: {err}"),
            Self::SerializationError(err) => write!(f, "config serialization error: {err}"),
        }
    }
}

impl fmt::Display for DError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileNotFound(msg) => write!(f, "file not found ({msg})"),
            Self::FilePermissionDenied(msg) => write!(f, "file permission denied ({msg})"),
            Self::Miscellanious(msg) => write!(f, "{msg}"),
        }
    }
}

impl fmt::Display for SError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedType(msg) => write!(f, "unsupported type ({msg})"),
            Self::PermissionDenied(msg) => write!(f, "permission denied ({msg})"),
            Self::KeyNotString(msg) => write!(f, "key not string ({msg})"),
            Self::Miscellanious(msg) => write!(f, "{msg}"),
        }
    }
}

pub trait ConfigLoader<T>{
    fn load(filePath: &Path) -> Result<T, ConfigErrors>;
    fn save(filePath: &Path, conf: &T) -> Result<(), ConfigErrors>;
}

pub struct TomlConfigLoader{}

impl<T > ConfigLoader<T> for TomlConfigLoader where T: Serialize + DeserializeOwned{
    fn load(filePath: &Path) -> Result<T, ConfigErrors> {
        match fs::read_to_string(filePath) {
            Ok(content) => {
                let res: Result<T, _> = toml::from_str(&content);
                match res {
                    Ok(t) => Ok(t),
                    Err(_) => Err(ConfigErrors::DeserializationError(DError::Miscellanious(String::from("Failed to convert toml to struct")))),
                }
            }

            Err(err) => {
                match err.kind(){
                    io::ErrorKind::NotFound => Err(ConfigErrors::DeserializationError(DError::FileNotFound(format!("OS error code: {:?}", err.raw_os_error())))),
                    io::ErrorKind::PermissionDenied => Err(ConfigErrors::DeserializationError(DError::FilePermissionDenied(format!("OS error code: {:?}", err.raw_os_error())))),
                    _ => Err(ConfigErrors::DeserializationError(DError::Miscellanious(format!("OS error code: {:?}", err.raw_os_error())))),
                }
            },
        }
    }

    fn save(filePath: &Path, conf: &T) -> Result<(), ConfigErrors> {
        let str: Result<String, toml::ser::Error> = toml::to_string(conf);
        match str {
            Ok(res) => {
                match fs::write(filePath, res) {
                    Ok(_) => Ok(()),
                    Err(err) => {
                        match err.kind() {
                            io::ErrorKind::PermissionDenied => Err(ConfigErrors::SerializationError(SError::PermissionDenied(format!("OS error code: {:?}", err.raw_os_error())))),
                            _ => Err(ConfigErrors::SerializationError(SError::Miscellanious(format!("OS error code: {:?}", err.raw_os_error())))),
                        }
                        
                    }
                }
            }
            Err(_) => Err(ConfigErrors::SerializationError(SError::Miscellanious(String::from("Failed to convert struct to toml"))))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub database_url: String,
    pub database_name: Option<String>,
    pub address: Option<String>,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub password: Option<String>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_url: String::new(),
            database_name: Some("vaction_app".to_string()),
            address: None,
            port: None,
            user: None,
            password: None,
        }
    }
}

impl DatabaseConfig {
    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    pub fn db_name(&self) -> &str {
        if let Some(name) = self.database_name.as_deref() {
            return name;
        }

        self.database_url
            .split_whitespace()
            .find_map(|part| part.strip_prefix("dbname="))
            .unwrap_or("vaction_app")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthSettings {
    pub store_credentials: bool,
}

impl Default for AuthSettings {
    fn default() -> Self {
        Self {
            store_credentials: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostgresSettings {
    pub schema: String,
}

impl Default for PostgresSettings {
    fn default() -> Self {
        Self {
            schema: "public".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VacationDbSettings {
    pub auto_setup_auth: bool,
}

impl Default for VacationDbSettings {
    fn default() -> Self {
        Self {
            auto_setup_auth: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
        }
    }
}

impl ServerSettings {
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::from((
            self.host.parse().unwrap_or(std::net::IpAddr::V4(
                std::net::Ipv4Addr::LOCALHOST,
            )),
            self.port,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrototypeSettings {
    pub backend_name: String,
}

impl Default for PrototypeSettings {
    fn default() -> Self {
        Self {
            backend_name: "prototype".to_string(),
        }
    }
}

/// SMTP / outbound email settings (consumed by [`crate::email`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailSettings {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub from_address: String,
    pub from_name: Option<String>,
    #[serde(default)]
    pub tls: EmailTlsMode,
}

impl Default for EmailSettings {
    fn default() -> Self {
        Self {
            smtp_host: "localhost".to_string(),
            smtp_port: 1025,
            username: None,
            password: None,
            from_address: "noreply@localhost".to_string(),
            from_name: Some("Vaction App".to_string()),
            tls: EmailTlsMode::None,
        }
    }
}

/// How the SMTP client negotiates TLS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EmailTlsMode {
    /// Plain SMTP (no TLS). Suitable only for local relays.
    None,
    /// STARTTLS upgrade (typical submission port 587).
    #[default]
    StartTls,
    /// Implicit TLS / SMTPS (typical port 465).
    Tls,
}

/// Top-level configuration composed of subsystem-specific sections.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VactionAppConfigs {
    pub database: DatabaseConfig,
    pub auth: AuthSettings,
    pub postgres: PostgresSettings,
    pub vacation_db: VacationDbSettings,
    pub server: ServerSettings,
    #[serde(default)]
    pub prototype: PrototypeSettings,
    #[serde(default)]
    pub email: EmailSettings,
}

impl Default for VactionAppConfigs {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            auth: AuthSettings::default(),
            postgres: PostgresSettings::default(),
            vacation_db: VacationDbSettings::default(),
            server: ServerSettings::default(),
            prototype: PrototypeSettings::default(),
            email: EmailSettings::default(),
        }
    }
}

impl VactionAppConfigs {
    pub fn database_url(&self) -> &str {
        self.database.database_url()
    }

    pub fn db_name(&self) -> &str {
        self.database.db_name()
    }
}

pub fn load_config() -> Result<VactionAppConfigs, ConfigErrors> {
    load_config_from(default_config_path())
}

pub fn load_config_from(path: impl AsRef<Path>) -> Result<VactionAppConfigs, ConfigErrors> {
    TomlConfigLoader::load(path.as_ref())
}

pub fn load_prototype_config() -> Result<VactionAppConfigs, ConfigErrors> {
    load_config_from(prototype_config_path())
}

pub fn load_prototype_config_from(path: impl AsRef<Path>) -> Result<VactionAppConfigs, ConfigErrors> {
    TomlConfigLoader::load(path.as_ref())
}

fn default_config_path() -> PathBuf {
    PathBuf::from("server_configs.toml")
}

fn prototype_config_path() -> PathBuf {
    PathBuf::from("prototype_configs.toml")
}