use std::path::Path;

use crate::auth::{self, AuthConfig, AuthError, Session};
use crate::config::{load_config, load_config_from, ConfigErrors, VactionAppConfigs};
use crate::employees::{EmployeeID, Vacation, VacationID};
use crate::organizations::{Organization, OrganizationID};
use crate::users::UserID;
use crate::Date;
use crate::vacation_app_db::{
    self, EmployeeRecord, GroupRecord, UserRecord, VacationAppDbError,
};

#[derive(Debug, PartialEq, Eq)]
pub enum AppError {
    Config(String),
    ConnectionFailed,
    BindFailed { addr: String, reason: String },
    QueryFailed,
    NotFound,
    InvalidData,
    Forbidden,
    Auth(AuthError),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(msg) => write!(f, "config error: {msg}"),
            Self::ConnectionFailed => write!(f, "connection failed"),
            Self::BindFailed { addr, reason } => write!(
                f,
                "could not listen on {addr}: {reason}. \
                 Stop the other process using that port, or change [server].port in server_configs.toml"
            ),
            Self::QueryFailed => write!(f, "query failed"),
            Self::NotFound => write!(f, "not found"),
            Self::InvalidData => write!(f, "invalid data"),
            Self::Forbidden => write!(f, "forbidden"),
            Self::Auth(err) => write!(f, "auth error: {err:?}"),
        }
    }
}

impl From<VacationAppDbError> for AppError {
    fn from(error: VacationAppDbError) -> Self {
        match error {
            VacationAppDbError::Config(msg) => Self::Config(msg),
            VacationAppDbError::ConnectionFailed => Self::ConnectionFailed,
            VacationAppDbError::QueryFailed => Self::QueryFailed,
            VacationAppDbError::NotFound => Self::NotFound,
            VacationAppDbError::InvalidData => Self::InvalidData,
        }
    }
}

impl From<AuthError> for AppError {
    fn from(error: AuthError) -> Self {
        Self::Auth(error)
    }
}

impl From<ConfigErrors> for AppError {
    fn from(error: ConfigErrors) -> Self {
        Self::Config(error.to_string())
    }
}

/// Application-facing interface implemented by production and demo backends.
pub trait VacationAppApi: Send + Sync {
    fn config(&self) -> &VactionAppConfigs;

    async fn setup(&self) -> Result<(), AppError>;

    async fn authenticate_user(&self, email: &str, password: &str) -> Result<Session, AppError>;
    async fn authenticate_employee(
        &self,
        email: &str,
        password: &str,
    ) -> Result<Session, AppError>;
    async fn logout(&self, session_id: u32) -> Result<(), AppError>;
    async fn validate_session(&self, session_id: u32) -> Result<Session, AppError>;

    async fn create_employee(&self, username: &str, email: &str) -> Result<EmployeeRecord, AppError>;
    async fn get_employee(&self, employee_id: EmployeeID) -> Result<EmployeeRecord, AppError>;
    async fn list_employees(&self) -> Result<Vec<EmployeeRecord>, AppError>;
    async fn update_employee(
        &self,
        employee_id: EmployeeID,
        username: &str,
        email: &str,
    ) -> Result<EmployeeRecord, AppError>;
    async fn update_employee_avatar(
        &self,
        employee_id: EmployeeID,
        avatar_url: Option<&str>,
    ) -> Result<EmployeeRecord, AppError>;
    async fn delete_employee(&self, employee_id: EmployeeID) -> Result<(), AppError>;

    async fn create_organization(&self, owner_id: u32, name: &str) -> Result<Organization, AppError>;
    async fn get_organization(
        &self,
        organization_id: OrganizationID,
    ) -> Result<Organization, AppError>;
    async fn list_organizations(&self) -> Result<Vec<Organization>, AppError>;
    async fn update_organization(
        &self,
        organization_id: OrganizationID,
        owner_id: u32,
        name: &str,
    ) -> Result<Organization, AppError>;
    async fn delete_organization(&self, organization_id: OrganizationID) -> Result<(), AppError>;
    async fn add_employee_to_organization(
        &self,
        organization_id: OrganizationID,
        employee_id: EmployeeID,
    ) -> Result<(), AppError>;
    async fn remove_employee_from_organization(
        &self,
        organization_id: OrganizationID,
        employee_id: EmployeeID,
    ) -> Result<(), AppError>;

    async fn create_user(&self, email: &str, organization_id: u32) -> Result<UserRecord, AppError>;
    async fn get_user(&self, user_id: UserID) -> Result<UserRecord, AppError>;
    async fn list_users(&self) -> Result<Vec<UserRecord>, AppError>;
    async fn list_users_for_organization(
        &self,
        organization_id: u32,
    ) -> Result<Vec<UserRecord>, AppError>;
    async fn update_user(
        &self,
        user_id: UserID,
        email: &str,
        organization_id: u32,
    ) -> Result<UserRecord, AppError>;
    async fn delete_user(&self, user_id: UserID) -> Result<(), AppError>;

    async fn create_group(&self, organization_id: u32, name: &str) -> Result<GroupRecord, AppError>;
    async fn get_group(&self, group_id: u32) -> Result<GroupRecord, AppError>;
    async fn list_groups_for_organization(
        &self,
        organization_id: u32,
    ) -> Result<Vec<GroupRecord>, AppError>;
    async fn update_group(&self, group_id: u32, name: &str) -> Result<GroupRecord, AppError>;
    async fn delete_group(&self, group_id: u32) -> Result<(), AppError>;

    async fn get_vacations(&self) -> Result<Vec<Vacation>, AppError>;
    async fn get_vacations_for_employee(
        &self,
        employee_id: EmployeeID,
    ) -> Result<Vec<Vacation>, AppError>;
    async fn get_vacation(&self, vacation_id: VacationID) -> Result<Vacation, AppError>;
    async fn request_vacation(
        &self,
        employee_id: EmployeeID,
        start_date: Date,
        final_date: Date,
    ) -> Result<Vacation, AppError>;
    async fn approve_vacation(
        &self,
        vacation_id: VacationID,
        approved_by: EmployeeID,
        covered_by: EmployeeID,
    ) -> Result<Vacation, AppError>;
    async fn deny_vacation(
        &self,
        vacation_id: VacationID,
        denied_by: EmployeeID,
        reason: &str,
    ) -> Result<Vacation, AppError>;
    async fn request_vacation_cancellation(
        &self,
        vacation_id: VacationID,
        requested_by: EmployeeID,
        approved_by: EmployeeID,
        covered_by: EmployeeID,
    ) -> Result<Vacation, AppError>;
    async fn approve_vacation_cancellation(
        &self,
        vacation_id: VacationID,
        cancelled_by: EmployeeID,
    ) -> Result<Vacation, AppError>;
    async fn deny_vacation_cancellation(
        &self,
        vacation_id: VacationID,
        approved_by: EmployeeID,
        covered_by: EmployeeID,
    ) -> Result<Vacation, AppError>;
    async fn update_vacation(
        &self,
        vacation_id: VacationID,
        employee_id: EmployeeID,
        start_date: Date,
        final_date: Date,
    ) -> Result<Vacation, AppError>;
    async fn update_vacation_fields(
        &self,
        vacation_id: VacationID,
        employee_id: EmployeeID,
        start_date: Date,
        final_date: Date,
        details: &str,
    ) -> Result<Vacation, AppError>;
    async fn delete_vacation(&self, vacation_id: VacationID) -> Result<(), AppError>;
    async fn list_vacation_events(
        &self,
        vacation_id: VacationID,
    ) -> Result<Vec<crate::vacation_app_db::EventRecord>, AppError>;
    async fn list_all_events(&self) -> Result<Vec<crate::vacation_app_db::EventRecord>, AppError>;
    async fn log_app_event(&self, event: crate::events::Event) -> Result<(), AppError>;
}

/// Production backend backed by PostgreSQL.
pub struct PostgresVacationApp {
    config: VactionAppConfigs,
}

impl PostgresVacationApp {
    pub fn new(config: VactionAppConfigs) -> Self {
        Self { config }
    }

    pub fn from_config_file(path: impl AsRef<Path>) -> Result<Self, AppError> {
        Ok(Self::new(load_config_from(path)?))
    }

    pub fn from_default_config() -> Result<Self, AppError> {
        Ok(Self::new(load_config()?))
    }
}

impl AuthConfig for PostgresVacationApp {
    fn database_url(&self) -> &str {
        self.config.database_url()
    }
}

impl VacationAppApi for PostgresVacationApp {
    fn config(&self) -> &VactionAppConfigs {
        &self.config
    }

    async fn setup(&self) -> Result<(), AppError> {
        vacation_app_db::setup_db(&self.config).await?;
        Ok(())
    }

    async fn authenticate_user(&self, email: &str, password: &str) -> Result<Session, AppError> {
        Ok(auth::authenticate_user(&self.config, email, password).await?)
    }

    async fn authenticate_employee(
        &self,
        email: &str,
        password: &str,
    ) -> Result<Session, AppError> {
        Ok(auth::authenticate_employee(&self.config, email, password).await?)
    }

    async fn logout(&self, session_id: u32) -> Result<(), AppError> {
        Ok(auth::logout(&self.config, session_id).await?)
    }

    async fn validate_session(&self, session_id: u32) -> Result<Session, AppError> {
        Ok(auth::validate_session(&self.config, session_id).await?)
    }

    async fn create_employee(&self, username: &str, email: &str) -> Result<EmployeeRecord, AppError> {
        Ok(vacation_app_db::create_employee(&self.config, username, email).await?)
    }

    async fn get_employee(&self, employee_id: EmployeeID) -> Result<EmployeeRecord, AppError> {
        Ok(vacation_app_db::get_employee(&self.config, employee_id).await?)
    }

    async fn list_employees(&self) -> Result<Vec<EmployeeRecord>, AppError> {
        Ok(vacation_app_db::list_employees(&self.config).await?)
    }

    async fn update_employee(
        &self,
        employee_id: EmployeeID,
        username: &str,
        email: &str,
    ) -> Result<EmployeeRecord, AppError> {
        Ok(vacation_app_db::update_employee(&self.config, employee_id, username, email).await?)
    }

    async fn update_employee_avatar(
        &self,
        employee_id: EmployeeID,
        avatar_url: Option<&str>,
    ) -> Result<EmployeeRecord, AppError> {
        Ok(
            vacation_app_db::update_employee_avatar(&self.config, employee_id, avatar_url).await?,
        )
    }

    async fn delete_employee(&self, employee_id: EmployeeID) -> Result<(), AppError> {
        Ok(vacation_app_db::delete_employee(&self.config, employee_id).await?)
    }

    async fn create_organization(&self, owner_id: u32, name: &str) -> Result<Organization, AppError> {
        Ok(vacation_app_db::create_organization(&self.config, owner_id, name).await?)
    }

    async fn get_organization(
        &self,
        organization_id: OrganizationID,
    ) -> Result<Organization, AppError> {
        Ok(vacation_app_db::get_organization(&self.config, organization_id).await?)
    }

    async fn list_organizations(&self) -> Result<Vec<Organization>, AppError> {
        Ok(vacation_app_db::list_organizations(&self.config).await?)
    }

    async fn update_organization(
        &self,
        organization_id: OrganizationID,
        owner_id: u32,
        name: &str,
    ) -> Result<Organization, AppError> {
        Ok(vacation_app_db::update_organization(&self.config, organization_id, owner_id, name).await?)
    }

    async fn delete_organization(&self, organization_id: OrganizationID) -> Result<(), AppError> {
        Ok(vacation_app_db::delete_organization(&self.config, organization_id).await?)
    }

    async fn add_employee_to_organization(
        &self,
        organization_id: OrganizationID,
        employee_id: EmployeeID,
    ) -> Result<(), AppError> {
        Ok(vacation_app_db::add_employee_to_organization(&self.config, organization_id, employee_id).await?)
    }

    async fn remove_employee_from_organization(
        &self,
        organization_id: OrganizationID,
        employee_id: EmployeeID,
    ) -> Result<(), AppError> {
        Ok(vacation_app_db::remove_employee_from_organization(&self.config, organization_id, employee_id).await?)
    }

    async fn create_user(&self, email: &str, organization_id: u32) -> Result<UserRecord, AppError> {
        Ok(vacation_app_db::create_user(&self.config, email, organization_id).await?)
    }

    async fn get_user(&self, user_id: UserID) -> Result<UserRecord, AppError> {
        Ok(vacation_app_db::get_user(&self.config, user_id).await?)
    }

    async fn list_users(&self) -> Result<Vec<UserRecord>, AppError> {
        Ok(vacation_app_db::list_users(&self.config).await?)
    }

    async fn list_users_for_organization(
        &self,
        organization_id: u32,
    ) -> Result<Vec<UserRecord>, AppError> {
        Ok(vacation_app_db::list_users_for_organization(&self.config, organization_id).await?)
    }

    async fn update_user(
        &self,
        user_id: UserID,
        email: &str,
        organization_id: u32,
    ) -> Result<UserRecord, AppError> {
        Ok(vacation_app_db::update_user(&self.config, user_id, email, organization_id).await?)
    }

    async fn delete_user(&self, user_id: UserID) -> Result<(), AppError> {
        Ok(vacation_app_db::delete_user(&self.config, user_id).await?)
    }

    async fn create_group(&self, organization_id: u32, name: &str) -> Result<GroupRecord, AppError> {
        Ok(vacation_app_db::create_group(&self.config, organization_id, name).await?)
    }

    async fn get_group(&self, group_id: u32) -> Result<GroupRecord, AppError> {
        Ok(vacation_app_db::get_group(&self.config, group_id).await?)
    }

    async fn list_groups_for_organization(
        &self,
        organization_id: u32,
    ) -> Result<Vec<GroupRecord>, AppError> {
        Ok(vacation_app_db::list_groups_for_organization(&self.config, organization_id).await?)
    }

    async fn update_group(&self, group_id: u32, name: &str) -> Result<GroupRecord, AppError> {
        Ok(vacation_app_db::update_group(&self.config, group_id, name).await?)
    }

    async fn delete_group(&self, group_id: u32) -> Result<(), AppError> {
        Ok(vacation_app_db::delete_group(&self.config, group_id).await?)
    }

    async fn get_vacations(&self) -> Result<Vec<Vacation>, AppError> {
        Ok(vacation_app_db::get_vacations(&self.config).await?)
    }

    async fn get_vacations_for_employee(
        &self,
        employee_id: EmployeeID,
    ) -> Result<Vec<Vacation>, AppError> {
        Ok(vacation_app_db::get_vacations_for_employee(&self.config, employee_id).await?)
    }

    async fn get_vacation(&self, vacation_id: VacationID) -> Result<Vacation, AppError> {
        Ok(vacation_app_db::get_vacation(&self.config, vacation_id).await?)
    }

    async fn request_vacation(
        &self,
        employee_id: EmployeeID,
        start_date: Date,
        final_date: Date,
    ) -> Result<Vacation, AppError> {
        Ok(vacation_app_db::request_vacation(&self.config, employee_id, start_date, final_date).await?)
    }

    async fn approve_vacation(
        &self,
        vacation_id: VacationID,
        approved_by: EmployeeID,
        covered_by: EmployeeID,
    ) -> Result<Vacation, AppError> {
        Ok(vacation_app_db::approve_vacation(&self.config, vacation_id, approved_by, covered_by).await?)
    }

    async fn deny_vacation(
        &self,
        vacation_id: VacationID,
        denied_by: EmployeeID,
        reason: &str,
    ) -> Result<Vacation, AppError> {
        Ok(vacation_app_db::deny_vacation_with_reason(
            &self.config,
            vacation_id,
            denied_by,
            reason,
        )
        .await?)
    }

    async fn request_vacation_cancellation(
        &self,
        vacation_id: VacationID,
        requested_by: EmployeeID,
        approved_by: EmployeeID,
        covered_by: EmployeeID,
    ) -> Result<Vacation, AppError> {
        Ok(vacation_app_db::request_vacation_cancellation(
            &self.config,
            vacation_id,
            requested_by,
            approved_by,
            covered_by,
        )
        .await?)
    }

    async fn approve_vacation_cancellation(
        &self,
        vacation_id: VacationID,
        cancelled_by: EmployeeID,
    ) -> Result<Vacation, AppError> {
        Ok(vacation_app_db::approve_vacation_cancellation(
            &self.config,
            vacation_id,
            cancelled_by,
        )
        .await?)
    }

    async fn deny_vacation_cancellation(
        &self,
        vacation_id: VacationID,
        approved_by: EmployeeID,
        covered_by: EmployeeID,
    ) -> Result<Vacation, AppError> {
        Ok(vacation_app_db::deny_vacation_cancellation(
            &self.config,
            vacation_id,
            approved_by,
            covered_by,
        )
        .await?)
    }

    async fn update_vacation(
        &self,
        vacation_id: VacationID,
        employee_id: EmployeeID,
        start_date: Date,
        final_date: Date,
    ) -> Result<Vacation, AppError> {
        Ok(vacation_app_db::update_vacation(
            &self.config,
            vacation_id,
            employee_id,
            start_date,
            final_date,
        )
        .await?)
    }

    async fn update_vacation_fields(
        &self,
        vacation_id: VacationID,
        employee_id: EmployeeID,
        start_date: Date,
        final_date: Date,
        details: &str,
    ) -> Result<Vacation, AppError> {
        Ok(vacation_app_db::update_vacation_details(
            &self.config,
            vacation_id,
            employee_id,
            start_date,
            final_date,
            details,
        )
        .await?)
    }

    async fn delete_vacation(&self, vacation_id: VacationID) -> Result<(), AppError> {
        Ok(vacation_app_db::delete_vacation(&self.config, vacation_id).await?)
    }

    async fn list_vacation_events(
        &self,
        vacation_id: VacationID,
    ) -> Result<Vec<vacation_app_db::EventRecord>, AppError> {
        Ok(vacation_app_db::list_events_for_vacation(&self.config, vacation_id).await?)
    }

    async fn list_all_events(&self) -> Result<Vec<vacation_app_db::EventRecord>, AppError> {
        Ok(vacation_app_db::list_all_events(&self.config).await?)
    }

    async fn log_app_event(&self, event: crate::events::Event) -> Result<(), AppError> {
        vacation_app_db::log_event(&self.config, &event).await?;
        Ok(())
    }
}
