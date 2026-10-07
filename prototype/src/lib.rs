use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use app_core::api::{AppError, PostgresVacationApp, VacationAppApi};
use app_core::app_ext::{
    vacation_stats_for, AdminEmployeeDetail, EmployeePermissions, EmployeeProfile, GroupDetail,
    GroupMemberPermissions, MockPaymentDetails, VacationAppExt, VacationGroupApproval,
};
use app_core::auth::{self, Session};
use app_core::db::DBQueryOps;
use app_core::employees::{EmployeeID, Vacation, VacationID, VacationStatus};
use app_core::events::{Event, EventType};
use app_core::groups::GroupMembershipStore;
use app_core::organizations::OrganizationID;
use app_core::postgres::{PostgresConnector, PostgresServer};
use app_core::users::UserID;
use app_core::{
    load_prototype_config, load_prototype_config_from, Date, VactionAppConfigs,
};
use tokio::sync::Mutex;

const MAX_AVATAR_DATA_URL_LEN: usize = 180_000;

fn validate_avatar_data_url(url: &str) -> Result<(), AppError> {
    if url.len() > MAX_AVATAR_DATA_URL_LEN {
        return Err(AppError::InvalidData);
    }
    let lower = url.to_ascii_lowercase();
    if !(lower.starts_with("data:image/jpeg;base64,")
        || lower.starts_with("data:image/jpg;base64,")
        || lower.starts_with("data:image/png;base64,")
        || lower.starts_with("data:image/webp;base64,"))
    {
        return Err(AppError::InvalidData);
    }
    Ok(())
}

/// Prototype-only overlay (group store + billing mock).
#[derive(Default)]
struct ExtState {
    groups: GroupMembershipStore,
    user_payments: HashMap<u32, MockPaymentDetails>,
    seeded: bool,
}

/// Prototype backend: loads `prototype_configs.toml`, connects with
/// [`PostgresConnector`] to a dedicated database on the same server, and
/// delegates core API work to [`PostgresVacationApp`].
pub struct PrototypeVacationApp {
    config: VactionAppConfigs,
    postgres: PostgresVacationApp,
    ext: Arc<Mutex<ExtState>>,
}

impl PrototypeVacationApp {
    pub fn new() -> Result<Self, AppError> {
        Self::from_config(load_prototype_config()?)
    }

    pub fn from_config_file(path: impl AsRef<Path>) -> Result<Self, AppError> {
        Self::from_config(load_prototype_config_from(path)?)
    }

    pub fn from_config(config: VactionAppConfigs) -> Result<Self, AppError> {
        Ok(Self {
            postgres: PostgresVacationApp::new(config.clone()),
            config,
            ext: Arc::new(Mutex::new(ExtState::default())),
        })
    }

    fn default_config_candidates() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        paths.push(PathBuf::from("prototype_configs.toml"));
        if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
            let manifest = PathBuf::from(manifest);
            paths.push(manifest.join("prototype_configs.toml"));
            paths.push(manifest.join("..").join("prototype_configs.toml"));
        }
        paths
    }

    pub fn load() -> Result<Self, AppError> {
        for path in Self::default_config_candidates() {
            if path.exists() {
                return Self::from_config_file(path);
            }
        }
        Self::new()
    }

    pub async fn can_see_vacation_details(
        &self,
        viewer_id: EmployeeID,
        owner_id: u32,
        approval_group_ids: &[u32],
    ) -> Result<bool, AppError> {
        if viewer_id.id() == owner_id {
            return Ok(true);
        }
        let profile = self.build_employee_profile(viewer_id).await?;
        if profile.permissions.approve_vacations || profile.permissions.edit_vacations {
            return Ok(true);
        }
        let ext = self.ext.lock().await;
        Ok(approval_group_ids.iter().any(|group_id| {
            ext.groups
                .actor_can_vote_for_group(&profile, *group_id, owner_id)
        }))
    }

    async fn dispatch_password_changed_email(&self, username: &str, to_email: &str) {
        use app_core::{password_changed_message, LettreSmtpMailer, Mailer};

        let settings = &self.config.email;
        let mailer = match LettreSmtpMailer::from_settings(settings) {
            Ok(mailer) => mailer,
            Err(err) => {
                eprintln!(
                    "[email] skip password_changed: cannot build SMTP client for {}:{} ({err})",
                    settings.smtp_host, settings.smtp_port
                );
                return;
            }
        };
        let message = match password_changed_message(to_email, username, settings) {
            Ok(message) => message,
            Err(err) => {
                eprintln!("[email] skip password_changed: invalid message ({err})");
                return;
            }
        };
        if let Err(err) = mailer.send(&message).await {
            eprintln!("[email] password_changed send failed ({err})");
        }
    }

    /// Ensures the prototype database exists on the shared Postgres server via
    /// [`PostgresConnector`], then opens it with a `db_name` query.
    async fn ensure_prototype_database(&self) -> Result<(), AppError> {
        let db_name = self.config.db_name().to_string();
        let _database = PostgresConnector::connect_url_ensuring_database(
            self.config.database_url(),
            &db_name,
        )
        .await
        .map_err(|err| match err {
            app_core::postgres::PostgresError::CreateDatabaseDenied => AppError::Config(format!(
                "role cannot CREATE DATABASE `{db_name}`. As a Postgres admin run: \
                 CREATE DATABASE {db_name} OWNER <your_user>; \
                 or GRANT CREATEDB TO <your_user>;"
            )),
            other => AppError::Config(format!("postgres connector failed: {other:?}")),
        })?;

        // Re-open through the same connector query API the rest of the app uses.
        let server = PostgresConnector::connect_url(self.config.database_url())
            .await
            .map_err(|err| AppError::Config(format!("postgres connector failed: {err:?}")))?;
        let _ = server
            .open(PostgresServer::query_db_name(DBQueryOps::Equal, &db_name))
            .await
            .map_err(|err| AppError::Config(format!("open prototype db failed: {err:?}")))?;

        Ok(())
    }

    async fn seed_demo_if_needed(&self) -> Result<(), AppError> {
        {
            let ext = self.ext.lock().await;
            if ext.seeded {
                return Ok(());
            }
        }

        let users = self.postgres.list_users().await?;
        if !users.is_empty() {
            let mut ext = self.ext.lock().await;
            ext.seeded = true;
            return Ok(());
        }

        let org = self
            .postgres
            .create_organization(0, "Demo Corp")
            .await?;
        let org_id = org.organization_id().id();

        let user = self
            .postgres
            .create_user("demo@test.com", org_id)
            .await?;
        self.postgres
            .update_organization(org.organization_id(), user.user_id().id(), "Demo Corp")
            .await?;
        auth::write_user_credentials(
            &self.config,
            user.user_id(),
            "demo@test.com",
            "demo",
        )
        .await?;

        let employee = self
            .postgres
            .create_employee("bob", "bob@test.com")
            .await?;
        self.postgres
            .add_employee_to_organization(org.organization_id(), employee.employee_id())
            .await?;
        auth::write_employee_credentials(
            &self.config,
            employee.employee_id(),
            "bob@test.com",
            "demo",
        )
        .await?;

        let group = self.postgres.create_group(org_id, "Managers").await?;

        let demo_permissions = EmployeePermissions {
            create_groups: true,
            approve_vacations: true,
            approve_own_vacations: false,
            request_vacation: true,
            edit_vacations: true,
            delete_vacations: true,
            edit_own_account: true,
        };
        let employee_id = employee.employee_id().id();
        self.persist_employee_permissions(employee_id, &demo_permissions)
            .await?;
        self.persist_grant_group_admin(group.group_id(), employee_id)
            .await?;

        let mut ext = self.ext.lock().await;
        ext.user_payments.insert(
            user.user_id().id(),
            MockPaymentDetails {
                billing_name: "Demo Admin".into(),
                card_last4: "4242".into(),
                plan: "Business".into(),
            },
        );
        ext.seeded = true;
        Ok(())
    }

    async fn with_ext<F, T>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&mut ExtState) -> Result<T, AppError>,
    {
        let mut ext = self.ext.lock().await;
        f(&mut ext)
    }

    async fn with_groups<F, T>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&mut GroupMembershipStore) -> Result<T, AppError>,
    {
        let mut ext = self.ext.lock().await;
        f(&mut ext.groups)
    }

    async fn load_groups_from_db(&self) -> Result<(), AppError> {
        let store = app_core::vacation_app_db::load_group_membership_store(&self.config).await?;
        let mut ext = self.ext.lock().await;
        ext.groups = store;
        Ok(())
    }

    async fn persist_employee_permissions(
        &self,
        employee_id: u32,
        permissions: &EmployeePermissions,
    ) -> Result<(), AppError> {
        app_core::vacation_app_db::save_employee_permissions(&self.config, employee_id, permissions)
            .await?;
        Ok(())
    }

    async fn persist_grant_group_admin(
        &self,
        group_id: u32,
        employee_id: u32,
    ) -> Result<(), AppError> {
        app_core::vacation_app_db::save_group_member(&self.config, group_id, employee_id).await?;
        app_core::vacation_app_db::save_group_member_permissions(
            &self.config,
            group_id,
            employee_id,
            &GroupMemberPermissions::group_admin(),
        )
        .await?;
        Ok(())
    }

    async fn persist_group_member(&self, group_id: u32, employee_id: u32) -> Result<(), AppError> {
        app_core::vacation_app_db::save_group_member(&self.config, group_id, employee_id).await?;
        Ok(())
    }

    async fn persist_remove_group_member(
        &self,
        group_id: u32,
        employee_id: u32,
    ) -> Result<(), AppError> {
        app_core::vacation_app_db::delete_group_member(&self.config, group_id, employee_id).await?;
        Ok(())
    }

    async fn persist_group_member_permissions(
        &self,
        group_id: u32,
        employee_id: u32,
        permissions: &GroupMemberPermissions,
    ) -> Result<(), AppError> {
        app_core::vacation_app_db::save_group_member_permissions(
            &self.config,
            group_id,
            employee_id,
            permissions,
        )
        .await?;
        Ok(())
    }

    async fn persist_replace_employee_groups(
        &self,
        employee_id: u32,
        group_ids: &[u32],
    ) -> Result<(), AppError> {
        app_core::vacation_app_db::replace_employee_group_memberships(
            &self.config,
            employee_id,
            group_ids,
        )
        .await?;
        Ok(())
    }

    async fn employee_organization_id(&self, employee_id: EmployeeID) -> Result<Option<u32>, AppError> {
        let orgs = self.postgres.list_organizations().await?;
        Ok(orgs
            .into_iter()
            .find(|org| org.employee_ids().contains(&employee_id.id()))
            .map(|org| org.organization_id().id()))
    }

    async fn org_employee_ids_for(&self, employee_id: EmployeeID) -> Result<Vec<u32>, AppError> {
        match self.employee_organization_id(employee_id).await? {
            Some(org_id) => {
                let org = self
                    .postgres
                    .get_organization(OrganizationID::new(org_id))
                    .await?;
                Ok(org.employee_ids().to_vec())
            }
            None => Ok(vec![employee_id.id()]),
        }
    }

    async fn build_event_email_context(
        &self,
        event: &Event,
    ) -> Result<app_core::EventEmailContext, AppError> {
        use app_core::{EmployeeContact, EventEmailContext};

        let mut needed_ids: Vec<u32> = Vec::new();
        let mut approver_ids: Vec<u32> = Vec::new();

        match event.event_type() {
            EventType::EmployeeAdded { employee_id, .. } => {
                needed_ids.push(employee_id.id());
            }
            EventType::VacationRequest { employee_id, .. } => {
                needed_ids.push(employee_id.id());
                let org_ids = self.org_employee_ids_for(*employee_id).await?;
                approver_ids = {
                    let ext = self.ext.lock().await;
                    ext.groups
                        .approver_ids_for_vacation(employee_id.id(), &org_ids)
                };
                needed_ids.extend(approver_ids.iter().copied());
            }
            EventType::VacationCancellationRequested {
                employee_id,
                requested_by,
                ..
            } => {
                needed_ids.push(employee_id.id());
                needed_ids.push(requested_by.id());
                let org_ids = self.org_employee_ids_for(*employee_id).await?;
                approver_ids = {
                    let ext = self.ext.lock().await;
                    ext.groups
                        .approver_ids_for_vacation(employee_id.id(), &org_ids)
                };
                needed_ids.extend(approver_ids.iter().copied());
            }
            EventType::VacationApproval {
                employee_id,
                approved_by,
                ..
            } => {
                needed_ids.push(employee_id.id());
                if let Some(id) = approved_by {
                    needed_ids.push(id.id());
                }
            }
            EventType::VacationDenial {
                employee_id,
                denied_by,
                ..
            } => {
                needed_ids.push(employee_id.id());
                if let Some(id) = denied_by {
                    needed_ids.push(id.id());
                }
            }
            EventType::VacationCancellationApproved {
                employee_id,
                cancelled_by,
                ..
            } => {
                needed_ids.push(employee_id.id());
                if let Some(id) = cancelled_by {
                    needed_ids.push(id.id());
                }
            }
            EventType::VacationCancellationDenied {
                employee_id,
                denied_by,
                ..
            } => {
                needed_ids.push(employee_id.id());
                if let Some(id) = denied_by {
                    needed_ids.push(id.id());
                }
            }
            _ => {}
        }

        needed_ids.sort_unstable();
        needed_ids.dedup();

        let mut contacts = Vec::new();
        for id in needed_ids {
            if let Ok(emp) = self.postgres.get_employee(EmployeeID::new(id)).await {
                contacts.push(EmployeeContact::new(
                    id,
                    emp.email().to_string(),
                    emp.username().to_string(),
                ));
            }
        }

        Ok(EventEmailContext::from_settings(self.config.email.clone())
            .with_employees(contacts)
            .with_approver_ids(approver_ids))
    }

    /// Best-effort outbound mail for application events. Failures are logged but
    /// never block the primary action.
    async fn dispatch_event_emails(&self, event: &Event) {
        use app_core::{event_sends_email, messages_for_event, LettreSmtpMailer, Mailer};

        if !event_sends_email(event.event_type()) {
            return;
        }

        let event_name = event.event_type().type_name();
        let settings = &self.config.email;

        let mailer = match LettreSmtpMailer::from_settings(settings) {
            Ok(mailer) => mailer,
            Err(err) => {
                eprintln!(
                    "[email] skip {event_name}: cannot build SMTP client for {}:{} ({err})",
                    settings.smtp_host, settings.smtp_port
                );
                return;
            }
        };

        let ctx = match self.build_event_email_context(event).await {
            Ok(ctx) => ctx,
            Err(err) => {
                eprintln!("[email] skip {event_name}: failed to resolve recipients ({err:?})");
                return;
            }
        };

        let messages = match messages_for_event(event.event_type(), &ctx) {
            Ok(messages) => messages,
            Err(err) => {
                eprintln!("[email] skip {event_name}: failed to compose messages ({err})");
                return;
            }
        };

        if messages.is_empty() {
            eprintln!(
                "[email] skip {event_name}: no recipients (check approvers / employee email on record)"
            );
            return;
        }

        let smtp_host = settings.smtp_host.clone();
        let smtp_port = settings.smtp_port;
        tokio::spawn(async move {
            for message in messages {
                let to = message
                    .to
                    .iter()
                    .map(|a| a.email.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                match mailer.send(&message).await {
                    Ok(()) => {
                        println!(
                            "[email] sent {event_name} → {to} via {smtp_host}:{smtp_port}"
                        );
                    }
                    Err(err) => {
                        eprintln!(
                            "[email] FAILED {event_name} → {to} via {smtp_host}:{smtp_port}: {err}"
                        );
                    }
                }
            }
        });
    }

    async fn build_employee_profile(
        &self,
        employee_id: EmployeeID,
    ) -> Result<EmployeeProfile, AppError> {
        let employee = self.postgres.get_employee(employee_id).await?;
        let organization_id = self.employee_organization_id(employee_id).await?;
        let ext = self.ext.lock().await;
        Ok(EmployeeProfile {
            id: employee.employee_id().id(),
            username: employee.username().to_string(),
            email: employee.email().to_string(),
            avatar_url: employee.avatar_url().map(str::to_string),
            organization_id,
            group_ids: ext.groups.group_ids_for_employee(employee_id.id()),
            permissions: ext.groups.employee_permissions(employee_id.id()),
        })
    }

    fn require_can_decide(
        perms: &EmployeePermissions,
        actor_id: u32,
        vacation_owner_id: u32,
    ) -> Result<(), AppError> {
        let is_own = actor_id == vacation_owner_id;
        if is_own {
            if perms.approve_own_vacations {
                Ok(())
            } else {
                Err(AppError::Forbidden)
            }
        } else if perms.approve_vacations {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        }
    }

    async fn enrich_group_approvals(
        &self,
        records: Vec<app_core::vacation_app_db::VacationGroupApprovalRecord>,
    ) -> Result<Vec<VacationGroupApproval>, AppError> {
        let mut out = Vec::with_capacity(records.len());
        for record in records {
            let name = match self.get_group(record.group_id).await {
                Ok(group) => group.name().to_string(),
                Err(_) => format!("Group #{}", record.group_id),
            };
            out.push(VacationGroupApproval {
                group_id: record.group_id,
                group_name: name,
                status: record.status,
                decided_by: record.decided_by,
            });
        }
        Ok(out)
    }

    async fn assert_admin_manages_employee(
        &self,
        user_id: UserID,
        employee_id: EmployeeID,
    ) -> Result<(), AppError> {
        let user = self.postgres.get_user(user_id).await?;
        let org = self
            .postgres
            .get_organization(OrganizationID::new(user.organization_id()))
            .await?;
        if org.employee_ids().contains(&employee_id.id()) {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        }
    }
}

impl Default for PrototypeVacationApp {
    fn default() -> Self {
        Self::load().expect("prototype_configs.toml is required for the prototype backend")
    }
}

impl VacationAppApi for PrototypeVacationApp {
    fn config(&self) -> &VactionAppConfigs {
        &self.config
    }

    async fn setup(&self) -> Result<(), AppError> {
        self.ensure_prototype_database().await?;
        self.postgres.setup().await?;
        self.seed_demo_if_needed().await?;
        self.load_groups_from_db().await?;
        Ok(())
    }

    async fn authenticate_user(&self, email: &str, password: &str) -> Result<Session, AppError> {
        self.postgres.authenticate_user(email, password).await
    }

    async fn authenticate_employee(
        &self,
        email: &str,
        password: &str,
    ) -> Result<Session, AppError> {
        self.postgres.authenticate_employee(email, password).await
    }

    async fn logout(&self, session_id: u32) -> Result<(), AppError> {
        self.postgres.logout(session_id).await
    }

    async fn validate_session(&self, session_id: u32) -> Result<Session, AppError> {
        self.postgres.validate_session(session_id).await
    }

    async fn create_employee(&self, username: &str, email: &str) -> Result<app_core::vacation_app_db::EmployeeRecord, AppError> {
        let record = self.postgres.create_employee(username, email).await?;
        let mut ext = self.ext.lock().await;
        ext.groups
            .ensure_default_employee_permissions(record.employee_id().id());
        Ok(record)
    }

    async fn get_employee(&self, employee_id: EmployeeID) -> Result<app_core::vacation_app_db::EmployeeRecord, AppError> {
        self.postgres.get_employee(employee_id).await
    }

    async fn list_employees(&self) -> Result<Vec<app_core::vacation_app_db::EmployeeRecord>, AppError> {
        self.postgres.list_employees().await
    }

    async fn update_employee(
        &self,
        employee_id: EmployeeID,
        username: &str,
        email: &str,
    ) -> Result<app_core::vacation_app_db::EmployeeRecord, AppError> {
        self.postgres
            .update_employee(employee_id, username, email)
            .await
    }

    async fn update_employee_avatar(
        &self,
        employee_id: EmployeeID,
        avatar_url: Option<&str>,
    ) -> Result<app_core::vacation_app_db::EmployeeRecord, AppError> {
        self.postgres
            .update_employee_avatar(employee_id, avatar_url)
            .await
    }

    async fn delete_employee(&self, employee_id: EmployeeID) -> Result<(), AppError> {
        self.postgres.delete_employee(employee_id).await?;
        let mut ext = self.ext.lock().await;
        ext.groups.remove_employee(employee_id.id());
        Ok(())
    }

    async fn create_organization(
        &self,
        owner_id: u32,
        name: &str,
    ) -> Result<app_core::organizations::Organization, AppError> {
        self.postgres.create_organization(owner_id, name).await
    }

    async fn get_organization(
        &self,
        organization_id: OrganizationID,
    ) -> Result<app_core::organizations::Organization, AppError> {
        self.postgres.get_organization(organization_id).await
    }

    async fn list_organizations(&self) -> Result<Vec<app_core::organizations::Organization>, AppError> {
        self.postgres.list_organizations().await
    }

    async fn update_organization(
        &self,
        organization_id: OrganizationID,
        owner_id: u32,
        name: &str,
    ) -> Result<app_core::organizations::Organization, AppError> {
        self.postgres
            .update_organization(organization_id, owner_id, name)
            .await
    }

    async fn delete_organization(&self, organization_id: OrganizationID) -> Result<(), AppError> {
        self.postgres.delete_organization(organization_id).await
    }

    async fn add_employee_to_organization(
        &self,
        organization_id: OrganizationID,
        employee_id: EmployeeID,
    ) -> Result<(), AppError> {
        self.postgres
            .add_employee_to_organization(organization_id, employee_id)
            .await
    }

    async fn remove_employee_from_organization(
        &self,
        organization_id: OrganizationID,
        employee_id: EmployeeID,
    ) -> Result<(), AppError> {
        self.postgres
            .remove_employee_from_organization(organization_id, employee_id)
            .await
    }

    async fn create_user(
        &self,
        email: &str,
        organization_id: u32,
    ) -> Result<app_core::vacation_app_db::UserRecord, AppError> {
        self.postgres.create_user(email, organization_id).await
    }

    async fn get_user(&self, user_id: UserID) -> Result<app_core::vacation_app_db::UserRecord, AppError> {
        self.postgres.get_user(user_id).await
    }

    async fn list_users(&self) -> Result<Vec<app_core::vacation_app_db::UserRecord>, AppError> {
        self.postgres.list_users().await
    }

    async fn list_users_for_organization(
        &self,
        organization_id: u32,
    ) -> Result<Vec<app_core::vacation_app_db::UserRecord>, AppError> {
        self.postgres
            .list_users_for_organization(organization_id)
            .await
    }

    async fn update_user(
        &self,
        user_id: UserID,
        email: &str,
        organization_id: u32,
    ) -> Result<app_core::vacation_app_db::UserRecord, AppError> {
        self.postgres
            .update_user(user_id, email, organization_id)
            .await
    }

    async fn delete_user(&self, user_id: UserID) -> Result<(), AppError> {
        self.postgres.delete_user(user_id).await
    }

    async fn create_group(
        &self,
        organization_id: u32,
        name: &str,
    ) -> Result<app_core::vacation_app_db::GroupRecord, AppError> {
        let group = self.postgres.create_group(organization_id, name).await?;
        let mut ext = self.ext.lock().await;
        ext.groups.ensure_group(group.group_id());
        Ok(group)
    }

    async fn get_group(&self, group_id: u32) -> Result<app_core::vacation_app_db::GroupRecord, AppError> {
        self.postgres.get_group(group_id).await
    }

    async fn list_groups_for_organization(
        &self,
        organization_id: u32,
    ) -> Result<Vec<app_core::vacation_app_db::GroupRecord>, AppError> {
        self.postgres
            .list_groups_for_organization(organization_id)
            .await
    }

    async fn update_group(
        &self,
        group_id: u32,
        name: &str,
    ) -> Result<app_core::vacation_app_db::GroupRecord, AppError> {
        self.postgres.update_group(group_id, name).await
    }

    async fn delete_group(&self, group_id: u32) -> Result<(), AppError> {
        self.postgres.delete_group(group_id).await?;
        let mut ext = self.ext.lock().await;
        ext.groups.clear_group(group_id);
        Ok(())
    }

    async fn get_vacations(&self) -> Result<Vec<Vacation>, AppError> {
        self.postgres.get_vacations().await
    }

    async fn get_vacations_for_employee(
        &self,
        employee_id: EmployeeID,
    ) -> Result<Vec<Vacation>, AppError> {
        self.postgres.get_vacations_for_employee(employee_id).await
    }

    async fn get_vacation(&self, vacation_id: VacationID) -> Result<Vacation, AppError> {
        self.postgres.get_vacation(vacation_id).await
    }

    async fn request_vacation(
        &self,
        employee_id: EmployeeID,
        start_date: Date,
        final_date: Date,
    ) -> Result<Vacation, AppError> {
        self.postgres
            .request_vacation(employee_id, start_date, final_date)
            .await
    }

    async fn approve_vacation(
        &self,
        vacation_id: VacationID,
        approved_by: EmployeeID,
        covered_by: EmployeeID,
    ) -> Result<Vacation, AppError> {
        self.postgres
            .approve_vacation(vacation_id, approved_by, covered_by)
            .await
    }

    async fn deny_vacation(
        &self,
        vacation_id: VacationID,
        denied_by: EmployeeID,
        reason: &str,
    ) -> Result<Vacation, AppError> {
        self.postgres
            .deny_vacation(vacation_id, denied_by, reason)
            .await
    }

    async fn request_vacation_cancellation(
        &self,
        vacation_id: VacationID,
        requested_by: EmployeeID,
        approved_by: EmployeeID,
        covered_by: EmployeeID,
    ) -> Result<Vacation, AppError> {
        self.postgres
            .request_vacation_cancellation(vacation_id, requested_by, approved_by, covered_by)
            .await
    }

    async fn approve_vacation_cancellation(
        &self,
        vacation_id: VacationID,
        cancelled_by: EmployeeID,
    ) -> Result<Vacation, AppError> {
        self.postgres
            .approve_vacation_cancellation(vacation_id, cancelled_by)
            .await
    }

    async fn deny_vacation_cancellation(
        &self,
        vacation_id: VacationID,
        approved_by: EmployeeID,
        covered_by: EmployeeID,
    ) -> Result<Vacation, AppError> {
        self.postgres
            .deny_vacation_cancellation(vacation_id, approved_by, covered_by)
            .await
    }

    async fn update_vacation(
        &self,
        vacation_id: VacationID,
        employee_id: EmployeeID,
        start_date: Date,
        final_date: Date,
    ) -> Result<Vacation, AppError> {
        self.postgres
            .update_vacation(vacation_id, employee_id, start_date, final_date)
            .await
    }

    async fn update_vacation_fields(
        &self,
        vacation_id: VacationID,
        employee_id: EmployeeID,
        start_date: Date,
        final_date: Date,
        details: &str,
    ) -> Result<Vacation, AppError> {
        self.postgres
            .update_vacation_fields(vacation_id, employee_id, start_date, final_date, details)
            .await
    }

    async fn delete_vacation(&self, vacation_id: VacationID) -> Result<(), AppError> {
        self.postgres.delete_vacation(vacation_id).await
    }

    async fn list_vacation_events(
        &self,
        vacation_id: VacationID,
    ) -> Result<Vec<app_core::vacation_app_db::EventRecord>, AppError> {
        self.postgres.list_vacation_events(vacation_id).await
    }

    async fn list_all_events(&self) -> Result<Vec<app_core::vacation_app_db::EventRecord>, AppError> {
        self.postgres.list_all_events().await
    }

    async fn log_app_event(&self, event: Event) -> Result<(), AppError> {
        self.postgres.log_app_event(event.clone()).await?;
        self.dispatch_event_emails(&event).await;
        Ok(())
    }
}

impl VacationAppExt for PrototypeVacationApp {
    async fn register_employee_login(
        &self,
        email: &str,
        employee_id: EmployeeID,
        password: &str,
    ) -> Result<(), AppError> {
        Ok(auth::write_employee_credentials(&self.config, employee_id, email, password).await?)
    }

    async fn get_user_payment(&self, user_id: UserID) -> Result<MockPaymentDetails, AppError> {
        self.with_ext(|ext| {
            Ok(ext
                .user_payments
                .get(&user_id.id())
                .cloned()
                .unwrap_or_default())
        })
        .await
    }

    async fn set_user_payment(
        &self,
        user_id: UserID,
        payment: MockPaymentDetails,
    ) -> Result<MockPaymentDetails, AppError> {
        self.postgres.get_user(user_id).await?;
        self.with_ext(|ext| {
            ext.user_payments.insert(user_id.id(), payment.clone());
            Ok(payment)
        })
        .await
    }

    async fn create_employee_for_org(
        &self,
        user_id: UserID,
        organization_id: u32,
        username: &str,
        email: &str,
        password: &str,
    ) -> Result<EmployeeProfile, AppError> {
        let user = self.get_user(user_id).await?;
        if user.organization_id() != organization_id {
            return Err(AppError::Forbidden);
        }
        let employee = self.create_employee(username, email).await?;
        self.add_employee_to_organization(
            OrganizationID::new(organization_id),
            employee.employee_id(),
        )
        .await?;
        self.register_employee_login(email, employee.employee_id(), password)
            .await?;
        let _ = self
            .log_app_event(Event::new(
                Date::new(26, 1, 1),
                EventType::EmployeeAdded {
                    employee_id: employee.employee_id(),
                    added_by: user_id,
                },
            ))
            .await;
        self.get_employee_profile(employee.employee_id()).await
    }

    async fn get_employee_profile(
        &self,
        employee_id: EmployeeID,
    ) -> Result<EmployeeProfile, AppError> {
        self.build_employee_profile(employee_id).await
    }

    async fn update_own_account(
        &self,
        actor_id: EmployeeID,
        username: &str,
        email: &str,
        password: Option<&str>,
        password_change_email: Option<&str>,
    ) -> Result<EmployeeProfile, AppError> {
        let profile = self.build_employee_profile(actor_id).await?;
        if !profile.permissions.edit_own_account {
            return Err(AppError::Forbidden);
        }
        self.update_employee(actor_id, username, email).await?;
        auth::update_employee_credentials(&self.config, actor_id, email, password).await?;
        let _ = self
            .log_app_event(Event::new(
                Date::new(26, 1, 1),
                EventType::EmployeeModified {
                    employee_id: actor_id,
                    modified_by_user: None,
                    modified_by_employee: Some(actor_id),
                },
            ))
            .await;
        if password.is_some() {
            let notify_email = password_change_email
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or(email);
            self.dispatch_password_changed_email(username, notify_email)
                .await;
        }
        self.build_employee_profile(actor_id).await
    }

    async fn update_own_avatar(
        &self,
        actor_id: EmployeeID,
        avatar_url: Option<&str>,
    ) -> Result<EmployeeProfile, AppError> {
        let profile = self.build_employee_profile(actor_id).await?;
        if !profile.permissions.edit_own_account {
            return Err(AppError::Forbidden);
        }
        if let Some(url) = avatar_url {
            validate_avatar_data_url(url)?;
        }
        self.update_employee_avatar(actor_id, avatar_url).await?;
        self.build_employee_profile(actor_id).await
    }

    async fn list_employees_visible_to(
        &self,
        actor_id: EmployeeID,
    ) -> Result<Vec<app_core::vacation_app_db::EmployeeRecord>, AppError> {
        let visible = {
            let ext = self.ext.lock().await;
            ext.groups.visible_colleague_ids(actor_id.id())
        };
        let all = self.list_employees().await?;
        Ok(all
            .into_iter()
            .filter(|employee| visible.contains(&employee.employee_id().id()))
            .collect())
    }

    async fn list_vacations_visible_to(
        &self,
        actor_id: EmployeeID,
    ) -> Result<Vec<Vacation>, AppError> {
        let visible = {
            let ext = self.ext.lock().await;
            ext.groups.visible_colleague_ids(actor_id.id())
        };
        let all = self.get_vacations().await?;
        Ok(all
            .into_iter()
            .filter(|vacation| visible.contains(&vacation.employee_id().id()))
            .collect())
    }

    async fn get_employee_permissions(
        &self,
        actor_id: EmployeeID,
        target_id: EmployeeID,
    ) -> Result<EmployeePermissions, AppError> {
        let actor = self.build_employee_profile(actor_id).await?;
        if !actor.permissions.can_administer_groups() {
            return Err(AppError::Forbidden);
        }
        let target = self.build_employee_profile(target_id).await?;
        if actor.organization_id.is_none() || actor.organization_id != target.organization_id {
            return Err(AppError::Forbidden);
        }
        Ok(target.permissions)
    }

    async fn set_employee_permissions(
        &self,
        actor_id: EmployeeID,
        target_id: EmployeeID,
        permissions: EmployeePermissions,
    ) -> Result<EmployeePermissions, AppError> {
        let actor = self.build_employee_profile(actor_id).await?;
        self.postgres.get_employee(target_id).await?;
        let saved = self
            .with_groups(|groups| {
                groups.set_employee_permissions_as_actor(&actor, target_id.id(), permissions)
            })
            .await?;
        self.persist_employee_permissions(target_id.id(), &saved)
            .await?;
        Ok(saved)
    }

    async fn list_group_details(&self, organization_id: u32) -> Result<Vec<GroupDetail>, AppError> {
        let groups = self
            .postgres
            .list_groups_for_organization(organization_id)
            .await?;
        let ext = self.ext.lock().await;
        Ok(ext.groups.list_details(&groups))
    }

    async fn create_group_as_employee(
        &self,
        actor_id: EmployeeID,
        organization_id: u32,
        name: &str,
    ) -> Result<GroupDetail, AppError> {
        let actor = self.build_employee_profile(actor_id).await?;
        GroupMembershipStore::authorize_create_group(&actor, organization_id)?;
        let group = self.create_group(organization_id, name).await?;
        {
            let mut ext = self.ext.lock().await;
            ext.groups.grant_group_admin(group.group_id(), actor_id.id());
        }
        self.persist_grant_group_admin(group.group_id(), actor_id.id())
            .await?;
        let ext = self.ext.lock().await;
        Ok(ext.groups.to_detail(&group))
    }

    async fn update_group_as_employee(
        &self,
        actor_id: EmployeeID,
        group_id: u32,
        name: &str,
    ) -> Result<GroupDetail, AppError> {
        let actor = self.build_employee_profile(actor_id).await?;
        let group = self.postgres.get_group(group_id).await?;
        {
            let ext = self.ext.lock().await;
            ext.groups.authorize_update_group(&actor, &group)?;
        }
        let updated = self.update_group(group_id, name).await?;
        let ext = self.ext.lock().await;
        Ok(ext.groups.to_detail(&updated))
    }

    async fn delete_group_as_employee(
        &self,
        actor_id: EmployeeID,
        group_id: u32,
    ) -> Result<(), AppError> {
        let actor = self.build_employee_profile(actor_id).await?;
        let group = self.postgres.get_group(group_id).await?;
        {
            let ext = self.ext.lock().await;
            ext.groups.authorize_delete_group(&actor, &group)?;
        }
        self.delete_group(group_id).await
    }

    async fn add_group_member(
        &self,
        actor_id: EmployeeID,
        group_id: u32,
        member_id: EmployeeID,
    ) -> Result<GroupDetail, AppError> {
        let actor = self.build_employee_profile(actor_id).await?;
        let group = self.postgres.get_group(group_id).await?;
        self.postgres.get_employee(member_id).await?;
        let detail = self
            .with_groups(|groups| groups.add_member(&actor, &group, member_id.id()))
            .await?;
        self.persist_group_member(group_id, member_id.id()).await?;
        Ok(detail)
    }

    async fn remove_group_member(
        &self,
        actor_id: EmployeeID,
        group_id: u32,
        member_id: EmployeeID,
    ) -> Result<GroupDetail, AppError> {
        let actor = self.build_employee_profile(actor_id).await?;
        let group = self.postgres.get_group(group_id).await?;
        let detail = self
            .with_groups(|groups| groups.remove_member(&actor, &group, member_id.id()))
            .await?;
        self.persist_remove_group_member(group_id, member_id.id())
            .await?;
        Ok(detail)
    }

    async fn get_group_member_permissions(
        &self,
        actor_id: EmployeeID,
        group_id: u32,
        member_id: EmployeeID,
    ) -> Result<GroupMemberPermissions, AppError> {
        let actor = self.build_employee_profile(actor_id).await?;
        let group = self.postgres.get_group(group_id).await?;
        let ext = self.ext.lock().await;
        ext.groups
            .get_member_permissions_as_actor(&actor, &group, member_id.id())
    }

    async fn set_group_member_permissions(
        &self,
        actor_id: EmployeeID,
        group_id: u32,
        member_id: EmployeeID,
        permissions: GroupMemberPermissions,
    ) -> Result<GroupMemberPermissions, AppError> {
        let actor = self.build_employee_profile(actor_id).await?;
        let group = self.postgres.get_group(group_id).await?;
        let saved = self
            .with_groups(|groups| {
                groups.set_member_permissions_as_actor(&actor, &group, member_id.id(), permissions)
            })
            .await?;
        self.persist_group_member_permissions(group_id, member_id.id(), &saved)
            .await?;
        Ok(saved)
    }

    async fn admin_get_employee_detail(
        &self,
        user_id: UserID,
        employee_id: EmployeeID,
    ) -> Result<AdminEmployeeDetail, AppError> {
        self.assert_admin_manages_employee(user_id, employee_id)
            .await?;
        let profile = self.build_employee_profile(employee_id).await?;
        let vacations = self.get_vacations_for_employee(employee_id).await?;
        Ok(AdminEmployeeDetail {
            profile,
            vacations: vacation_stats_for(&vacations),
        })
    }

    async fn admin_update_employee(
        &self,
        user_id: UserID,
        employee_id: EmployeeID,
        username: &str,
        email: &str,
        password: Option<&str>,
    ) -> Result<EmployeeProfile, AppError> {
        self.assert_admin_manages_employee(user_id, employee_id)
            .await?;
        let existing = self.get_employee(employee_id).await?;
        self.update_employee(employee_id, username, email).await?;
        if let Some(password) = password {
            self.register_employee_login(email, employee_id, password)
                .await?;
        } else if existing.email() != email {
            self.register_employee_login(email, employee_id, "demo")
                .await?;
        }
        let _ = self
            .log_app_event(Event::new(
                Date::new(26, 1, 1),
                EventType::EmployeeModified {
                    employee_id,
                    modified_by_user: Some(user_id),
                    modified_by_employee: None,
                },
            ))
            .await;
        self.build_employee_profile(employee_id).await
    }

    async fn admin_set_employee_permissions(
        &self,
        user_id: UserID,
        employee_id: EmployeeID,
        permissions: EmployeePermissions,
    ) -> Result<EmployeePermissions, AppError> {
        self.assert_admin_manages_employee(user_id, employee_id)
            .await?;
        let saved = self
            .with_groups(|groups| {
                groups.set_employee_permissions(employee_id.id(), permissions.clone());
                Ok(permissions)
            })
            .await?;
        self.persist_employee_permissions(employee_id.id(), &saved)
            .await?;
        Ok(saved)
    }

    async fn admin_set_employee_groups(
        &self,
        user_id: UserID,
        employee_id: EmployeeID,
        group_ids: Vec<u32>,
    ) -> Result<EmployeeProfile, AppError> {
        self.assert_admin_manages_employee(user_id, employee_id)
            .await?;
        let user = self.get_user(user_id).await?;
        for group_id in &group_ids {
            let group = self.get_group(*group_id).await?;
            if group.organization_id() != user.organization_id() {
                return Err(AppError::Forbidden);
            }
        }
        self.with_groups(|groups| {
            groups.replace_employee_groups(employee_id.id(), &group_ids);
            Ok(())
        })
        .await?;
        self.persist_replace_employee_groups(employee_id.id(), &group_ids)
            .await?;
        self.build_employee_profile(employee_id).await
    }

    async fn admin_delete_employee(
        &self,
        user_id: UserID,
        employee_id: EmployeeID,
    ) -> Result<(), AppError> {
        self.assert_admin_manages_employee(user_id, employee_id)
            .await?;
        let _ = self
            .log_app_event(Event::new(
                Date::new(26, 1, 1),
                EventType::EmployeeDeleted {
                    employee_id,
                    deleted_by: user_id,
                },
            ))
            .await;
        self.delete_employee(employee_id).await
    }

    async fn admin_update_organization(
        &self,
        user_id: UserID,
        organization_id: OrganizationID,
        name: &str,
    ) -> Result<app_core::organizations::Organization, AppError> {
        let user = self.get_user(user_id).await?;
        if user.organization_id() != organization_id.id() {
            return Err(AppError::Forbidden);
        }
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(AppError::InvalidData);
        }
        let org = self.get_organization(organization_id).await?;
        self.update_organization(organization_id, org.owner_id(), trimmed)
            .await
    }

    async fn admin_create_group(
        &self,
        user_id: UserID,
        organization_id: u32,
        name: &str,
    ) -> Result<GroupDetail, AppError> {
        let user = self.get_user(user_id).await?;
        if user.organization_id() != organization_id {
            return Err(AppError::Forbidden);
        }
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(AppError::InvalidData);
        }
        let group = self.create_group(organization_id, trimmed).await?;
        let ext = self.ext.lock().await;
        Ok(ext.groups.to_detail(&group))
    }

    async fn admin_update_group(
        &self,
        user_id: UserID,
        group_id: u32,
        name: &str,
    ) -> Result<GroupDetail, AppError> {
        let user = self.get_user(user_id).await?;
        let group = self.get_group(group_id).await?;
        if group.organization_id() != user.organization_id() {
            return Err(AppError::Forbidden);
        }
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(AppError::InvalidData);
        }
        let updated = self.update_group(group_id, trimmed).await?;
        let ext = self.ext.lock().await;
        Ok(ext.groups.to_detail(&updated))
    }

    async fn admin_delete_group(&self, user_id: UserID, group_id: u32) -> Result<(), AppError> {
        let user = self.get_user(user_id).await?;
        let group = self.get_group(group_id).await?;
        if group.organization_id() != user.organization_id() {
            return Err(AppError::Forbidden);
        }
        self.delete_group(group_id).await
    }

    async fn admin_add_group_member(
        &self,
        user_id: UserID,
        group_id: u32,
        member_id: EmployeeID,
    ) -> Result<GroupDetail, AppError> {
        self.assert_admin_manages_employee(user_id, member_id)
            .await?;
        let user = self.get_user(user_id).await?;
        let group = self.get_group(group_id).await?;
        if group.organization_id() != user.organization_id() {
            return Err(AppError::Forbidden);
        }
        self.postgres.get_employee(member_id).await?;
        let detail = self
            .with_groups(|groups| Ok(groups.admin_add_member(&group, member_id.id())))
            .await?;
        self.persist_group_member(group_id, member_id.id()).await?;
        Ok(detail)
    }

    async fn admin_remove_group_member(
        &self,
        user_id: UserID,
        group_id: u32,
        member_id: EmployeeID,
    ) -> Result<GroupDetail, AppError> {
        self.assert_admin_manages_employee(user_id, member_id)
            .await?;
        let user = self.get_user(user_id).await?;
        let group = self.get_group(group_id).await?;
        if group.organization_id() != user.organization_id() {
            return Err(AppError::Forbidden);
        }
        let detail = self
            .with_groups(|groups| groups.admin_remove_member(&group, member_id.id()))
            .await?;
        self.persist_remove_group_member(group_id, member_id.id())
            .await?;
        Ok(detail)
    }

    async fn admin_get_group_member_permissions(
        &self,
        user_id: UserID,
        group_id: u32,
        member_id: EmployeeID,
    ) -> Result<GroupMemberPermissions, AppError> {
        self.assert_admin_manages_employee(user_id, member_id)
            .await?;
        let user = self.get_user(user_id).await?;
        let group = self.get_group(group_id).await?;
        if group.organization_id() != user.organization_id() {
            return Err(AppError::Forbidden);
        }
        let ext = self.ext.lock().await;
        ext.groups.admin_get_member_permissions(&group, member_id.id())
    }

    async fn admin_set_group_member_permissions(
        &self,
        user_id: UserID,
        group_id: u32,
        member_id: EmployeeID,
        permissions: GroupMemberPermissions,
    ) -> Result<GroupMemberPermissions, AppError> {
        self.assert_admin_manages_employee(user_id, member_id)
            .await?;
        let user = self.get_user(user_id).await?;
        let group = self.get_group(group_id).await?;
        if group.organization_id() != user.organization_id() {
            return Err(AppError::Forbidden);
        }
        let saved = self
            .with_groups(|groups| {
                groups.admin_set_member_permissions(&group, member_id.id(), permissions)
            })
            .await?;
        self.persist_group_member_permissions(group_id, member_id.id(), &saved)
            .await?;
        Ok(saved)
    }

    async fn request_vacation_with_permissions(
        &self,
        employee_id: EmployeeID,
        start_date: Date,
        final_date: Date,
        details: &str,
    ) -> Result<Vacation, AppError> {
        let profile = self.build_employee_profile(employee_id).await?;
        if !profile.permissions.request_vacation {
            return Err(AppError::Forbidden);
        }
        let org_employee_ids = match profile.organization_id {
            Some(org_id) => {
                let org = self
                    .postgres
                    .get_organization(OrganizationID::new(org_id))
                    .await?;
                org.employee_ids().to_vec()
            }
            None => vec![employee_id.id()],
        };
        let needs_approval = {
            let ext = self.ext.lock().await;
            ext.groups
                .anyone_can_approve_vacation_for(employee_id.id(), &org_employee_ids)
        };
        let vacation = app_core::vacation_app_db::request_vacation_with_details(
            &self.config,
            employee_id,
            start_date,
            final_date,
            details,
        )
        .await?;
        let today = Date::new(26, 1, 1);
        let _ = self
            .log_app_event(Event::new(
                today,
                EventType::VacationRequest {
                    employee_id,
                    vacation_id: vacation.vacation_id(),
                },
            ))
            .await;

        // No one can approve → auto-approve (requester as approved/covered by).
        if !needs_approval {
            let vacation = self
                .approve_vacation(vacation.vacation_id(), employee_id, employee_id)
                .await?;
            let _ = self
                .log_app_event(Event::new(
                    today,
                    EventType::VacationApproval {
                        employee_id,
                        vacation_id: vacation.vacation_id(),
                        approved_by: Some(employee_id),
                        approved_by_user: None,
                    },
                ))
                .await;
            return Ok(vacation);
        }

        let group_ids = {
            let ext = self.ext.lock().await;
            ext.groups.approval_group_ids_for_employee(employee_id.id())
        };
        app_core::vacation_app_db::seed_vacation_group_approvals(
            &self.config,
            vacation.vacation_id(),
            &group_ids,
        )
        .await?;
        Ok(vacation)
    }

    async fn approve_vacation_with_permissions(
        &self,
        actor_id: EmployeeID,
        vacation_id: VacationID,
        covered_by: EmployeeID,
        group_id: Option<u32>,
    ) -> Result<Vacation, AppError> {
        let vacation = self.get_vacation(vacation_id).await?;
        if !matches!(vacation.status(), VacationStatus::AwaitingConfirmation) {
            return Err(AppError::InvalidData);
        }
        let profile = self.build_employee_profile(actor_id).await?;
        let owner_id = vacation.employee_id().id();
        let votes = app_core::vacation_app_db::list_vacation_group_approvals(
            &self.config,
            vacation_id,
        )
        .await?;

        // No group votes → legacy single global decision.
        if votes.is_empty() {
            Self::require_can_decide(&profile.permissions, actor_id.id(), owner_id)?;
            let vacation = self
                .approve_vacation(vacation_id, actor_id, covered_by)
                .await?;
            let today = Date::new(26, 1, 1);
            let _ = self
                .log_app_event(Event::new(
                    today,
                    EventType::VacationApproval {
                        employee_id: vacation.employee_id(),
                        vacation_id,
                        approved_by: Some(actor_id),
                        approved_by_user: None,
                    },
                ))
                .await;
            return Ok(vacation);
        }

        let pending: Vec<u32> = votes
            .iter()
            .filter(|v| v.status == "pending")
            .map(|v| v.group_id)
            .collect();
        if pending.is_empty() {
            return Err(AppError::InvalidData);
        }

        let votable = {
            let ext = self.ext.lock().await;
            ext.groups
                .actor_votable_pending_groups(&profile, &pending, owner_id)
        };
        if votable.is_empty() {
            return Err(AppError::Forbidden);
        }

        let targets: Vec<u32> = if let Some(gid) = group_id {
            if !votable.contains(&gid) {
                return Err(AppError::Forbidden);
            }
            vec![gid]
        } else {
            votable
        };

        let today = Date::new(26, 1, 1);
        for gid in &targets {
            app_core::vacation_app_db::record_vacation_group_decision(
                &self.config,
                vacation_id,
                *gid,
                "approved",
                actor_id,
            )
            .await?;
            let _ = self
                .log_app_event(Event::new(
                    today,
                    EventType::VacationGroupVote {
                        employee_id: vacation.employee_id(),
                        vacation_id,
                        group_id: *gid,
                        decided_by: actor_id,
                        approved: true,
                    },
                ))
                .await;
        }

        let updated_votes = app_core::vacation_app_db::list_vacation_group_approvals(
            &self.config,
            vacation_id,
        )
        .await?;
        let all_approved = updated_votes.iter().all(|v| v.status == "approved");
        if !all_approved {
            return self.get_vacation(vacation_id).await;
        }

        let vacation = self
            .approve_vacation(vacation_id, actor_id, covered_by)
            .await?;
        let _ = self
            .log_app_event(Event::new(
                today,
                EventType::VacationApproval {
                    employee_id: vacation.employee_id(),
                    vacation_id,
                    approved_by: Some(actor_id),
                    approved_by_user: None,
                },
            ))
            .await;
        Ok(vacation)
    }

    async fn deny_vacation_with_permissions(
        &self,
        actor_id: EmployeeID,
        vacation_id: VacationID,
        group_id: Option<u32>,
        reason: Option<&str>,
    ) -> Result<Vacation, AppError> {
        let reason = reason.unwrap_or("").trim();
        let vacation = self.get_vacation(vacation_id).await?;
        if !matches!(vacation.status(), VacationStatus::AwaitingConfirmation) {
            return Err(AppError::InvalidData);
        }
        let profile = self.build_employee_profile(actor_id).await?;
        let owner_id = vacation.employee_id().id();
        let votes = app_core::vacation_app_db::list_vacation_group_approvals(
            &self.config,
            vacation_id,
        )
        .await?;

        if votes.is_empty() {
            Self::require_can_decide(&profile.permissions, actor_id.id(), owner_id)?;
            let vacation = self.deny_vacation(vacation_id, actor_id, reason).await?;
            let today = Date::new(26, 1, 1);
            let _ = self
                .log_app_event(Event::new(
                    today,
                    EventType::VacationDenial {
                        employee_id: vacation.employee_id(),
                        vacation_id,
                        denied_by: Some(actor_id),
                        denied_by_user: None,
                        reason: if reason.is_empty() {
                            None
                        } else {
                            Some(reason.to_string())
                        },
                    },
                ))
                .await;
            return Ok(vacation);
        }

        let pending: Vec<u32> = votes
            .iter()
            .filter(|v| v.status == "pending")
            .map(|v| v.group_id)
            .collect();
        let votable = {
            let ext = self.ext.lock().await;
            ext.groups
                .actor_votable_pending_groups(&profile, &pending, owner_id)
        };
        if votable.is_empty() {
            return Err(AppError::Forbidden);
        }

        let target = if let Some(gid) = group_id {
            if !votable.contains(&gid) {
                return Err(AppError::Forbidden);
            }
            gid
        } else {
            votable[0]
        };

        app_core::vacation_app_db::record_vacation_group_decision(
            &self.config,
            vacation_id,
            target,
            "denied",
            actor_id,
        )
        .await?;

        let today = Date::new(26, 1, 1);
        let _ = self
            .log_app_event(Event::new(
                today,
                EventType::VacationGroupVote {
                    employee_id: vacation.employee_id(),
                    vacation_id,
                    group_id: target,
                    decided_by: actor_id,
                    approved: false,
                },
            ))
            .await;

        let vacation = self.deny_vacation(vacation_id, actor_id, reason).await?;
        let _ = self
            .log_app_event(Event::new(
                today,
                EventType::VacationDenial {
                    employee_id: vacation.employee_id(),
                    vacation_id,
                    denied_by: Some(actor_id),
                    denied_by_user: None,
                    reason: if reason.is_empty() {
                        None
                    } else {
                        Some(reason.to_string())
                    },
                },
            ))
            .await;
        Ok(vacation)
    }

    async fn request_vacation_cancellation_with_permissions(
        &self,
        actor_id: EmployeeID,
        vacation_id: VacationID,
    ) -> Result<Vacation, AppError> {
        let vacation = self.get_vacation(vacation_id).await?;
        let VacationStatus::Approved {
            approvedBy,
            coveredBy,
        } = *vacation.status()
        else {
            return Err(AppError::InvalidData);
        };
        let profile = self.build_employee_profile(actor_id).await?;
        let owner_id = vacation.employee_id().id();
        let can_request = {
            let ext = self.ext.lock().await;
            ext.groups
                .actor_can_request_cancellation(&profile, owner_id)
        };
        if !can_request {
            return Err(AppError::Forbidden);
        }

        let owner_org = self.employee_organization_id(vacation.employee_id()).await?;
        let org_employee_ids = match owner_org.or(profile.organization_id) {
            Some(org_id) => {
                let org = self
                    .postgres
                    .get_organization(OrganizationID::new(org_id))
                    .await?;
                org.employee_ids().to_vec()
            }
            None => vec![owner_id],
        };
        let needs_approval = {
            let ext = self.ext.lock().await;
            ext.groups
                .anyone_can_approve_vacation_for(owner_id, &org_employee_ids)
        };

        let vacation = self
            .request_vacation_cancellation(vacation_id, actor_id, approvedBy, coveredBy)
            .await?;
        let today = Date::new(26, 1, 1);
        let _ = self
            .log_app_event(Event::new(
                today,
                EventType::VacationCancellationRequested {
                    employee_id: vacation.employee_id(),
                    vacation_id,
                    requested_by: actor_id,
                },
            ))
            .await;

        // Clear leftover request-approval votes, then auto-cancel or seed a fresh ballot.
        app_core::vacation_app_db::delete_vacation_group_approvals(&self.config, vacation_id)
            .await?;

        if !needs_approval {
            let vacation = self
                .approve_vacation_cancellation(vacation_id, actor_id)
                .await?;
            let _ = self
                .log_app_event(Event::new(
                    today,
                    EventType::VacationCancellationApproved {
                        employee_id: vacation.employee_id(),
                        vacation_id,
                        cancelled_by: Some(actor_id),
                        cancelled_by_user: None,
                    },
                ))
                .await;
            return Ok(vacation);
        }

        let group_ids = {
            let ext = self.ext.lock().await;
            ext.groups.approval_group_ids_for_employee(owner_id)
        };
        app_core::vacation_app_db::seed_vacation_group_approvals(
            &self.config,
            vacation_id,
            &group_ids,
        )
        .await?;
        Ok(vacation)
    }

    async fn approve_vacation_cancellation_with_permissions(
        &self,
        actor_id: EmployeeID,
        vacation_id: VacationID,
        group_id: Option<u32>,
    ) -> Result<Vacation, AppError> {
        let vacation = self.get_vacation(vacation_id).await?;
        if !matches!(
            vacation.status(),
            VacationStatus::CancellationRequested { .. }
        ) {
            return Err(AppError::InvalidData);
        }
        let profile = self.build_employee_profile(actor_id).await?;
        let owner_id = vacation.employee_id().id();
        let votes = app_core::vacation_app_db::list_vacation_group_approvals(
            &self.config,
            vacation_id,
        )
        .await?;

        // No group votes → legacy single global decision (same as request approve).
        if votes.is_empty() {
            Self::require_can_decide(&profile.permissions, actor_id.id(), owner_id)?;
            let vacation = self
                .approve_vacation_cancellation(vacation_id, actor_id)
                .await?;
            let today = Date::new(26, 1, 1);
            let _ = self
                .log_app_event(Event::new(
                    today,
                    EventType::VacationCancellationApproved {
                        employee_id: vacation.employee_id(),
                        vacation_id,
                        cancelled_by: Some(actor_id),
                        cancelled_by_user: None,
                    },
                ))
                .await;
            return Ok(vacation);
        }

        let pending: Vec<u32> = votes
            .iter()
            .filter(|v| v.status == "pending")
            .map(|v| v.group_id)
            .collect();
        if pending.is_empty() {
            return Err(AppError::InvalidData);
        }

        let votable = {
            let ext = self.ext.lock().await;
            ext.groups
                .actor_votable_pending_groups(&profile, &pending, owner_id)
        };
        if votable.is_empty() {
            return Err(AppError::Forbidden);
        }

        let targets: Vec<u32> = if let Some(gid) = group_id {
            if !votable.contains(&gid) {
                return Err(AppError::Forbidden);
            }
            vec![gid]
        } else {
            votable
        };

        let today = Date::new(26, 1, 1);
        for gid in &targets {
            app_core::vacation_app_db::record_vacation_group_decision(
                &self.config,
                vacation_id,
                *gid,
                "approved",
                actor_id,
            )
            .await?;
            let _ = self
                .log_app_event(Event::new(
                    today,
                    EventType::VacationGroupVote {
                        employee_id: vacation.employee_id(),
                        vacation_id,
                        group_id: *gid,
                        decided_by: actor_id,
                        approved: true,
                    },
                ))
                .await;
        }

        let updated_votes = app_core::vacation_app_db::list_vacation_group_approvals(
            &self.config,
            vacation_id,
        )
        .await?;
        let all_approved = updated_votes.iter().all(|v| v.status == "approved");
        if !all_approved {
            return self.get_vacation(vacation_id).await;
        }

        let vacation = self
            .approve_vacation_cancellation(vacation_id, actor_id)
            .await?;
        let _ = self
            .log_app_event(Event::new(
                today,
                EventType::VacationCancellationApproved {
                    employee_id: vacation.employee_id(),
                    vacation_id,
                    cancelled_by: Some(actor_id),
                    cancelled_by_user: None,
                },
            ))
            .await;
        Ok(vacation)
    }

    async fn deny_vacation_cancellation_with_permissions(
        &self,
        actor_id: EmployeeID,
        vacation_id: VacationID,
        group_id: Option<u32>,
    ) -> Result<Vacation, AppError> {
        let vacation = self.get_vacation(vacation_id).await?;
        let VacationStatus::CancellationRequested {
            approvedBy,
            coveredBy,
            ..
        } = *vacation.status()
        else {
            return Err(AppError::InvalidData);
        };
        let profile = self.build_employee_profile(actor_id).await?;
        let owner_id = vacation.employee_id().id();
        let votes = app_core::vacation_app_db::list_vacation_group_approvals(
            &self.config,
            vacation_id,
        )
        .await?;

        if votes.is_empty() {
            Self::require_can_decide(&profile.permissions, actor_id.id(), owner_id)?;
            let vacation = self
                .deny_vacation_cancellation(vacation_id, approvedBy, coveredBy)
                .await?;
            let today = Date::new(26, 1, 1);
            let _ = self
                .log_app_event(Event::new(
                    today,
                    EventType::VacationCancellationDenied {
                        employee_id: vacation.employee_id(),
                        vacation_id,
                        denied_by: Some(actor_id),
                        denied_by_user: None,
                    },
                ))
                .await;
            return Ok(vacation);
        }

        let pending: Vec<u32> = votes
            .iter()
            .filter(|v| v.status == "pending")
            .map(|v| v.group_id)
            .collect();
        let votable = {
            let ext = self.ext.lock().await;
            ext.groups
                .actor_votable_pending_groups(&profile, &pending, owner_id)
        };
        if votable.is_empty() {
            return Err(AppError::Forbidden);
        }

        let target = if let Some(gid) = group_id {
            if !votable.contains(&gid) {
                return Err(AppError::Forbidden);
            }
            gid
        } else {
            votable[0]
        };

        app_core::vacation_app_db::record_vacation_group_decision(
            &self.config,
            vacation_id,
            target,
            "denied",
            actor_id,
        )
        .await?;

        let today = Date::new(26, 1, 1);
        let _ = self
            .log_app_event(Event::new(
                today,
                EventType::VacationGroupVote {
                    employee_id: vacation.employee_id(),
                    vacation_id,
                    group_id: target,
                    decided_by: actor_id,
                    approved: false,
                },
            ))
            .await;

        let vacation = self
            .deny_vacation_cancellation(vacation_id, approvedBy, coveredBy)
            .await?;
        // Clear the cancellation ballot so a later cancel request can reseed cleanly.
        let _ = app_core::vacation_app_db::delete_vacation_group_approvals(
            &self.config,
            vacation_id,
        )
        .await;
        let _ = self
            .log_app_event(Event::new(
                today,
                EventType::VacationCancellationDenied {
                    employee_id: vacation.employee_id(),
                    vacation_id,
                    denied_by: Some(actor_id),
                    denied_by_user: None,
                },
            ))
            .await;
        Ok(vacation)
    }

    async fn vacation_group_approvals(
        &self,
        vacation_id: VacationID,
    ) -> Result<Vec<VacationGroupApproval>, AppError> {
        let records =
            app_core::vacation_app_db::list_vacation_group_approvals(&self.config, vacation_id)
                .await?;
        self.enrich_group_approvals(records).await
    }

    async fn vacation_group_approvals_for_many(
        &self,
        vacation_ids: &[u32],
    ) -> Result<Vec<(u32, Vec<VacationGroupApproval>)>, AppError> {
        let records = app_core::vacation_app_db::list_vacation_group_approvals_for_many(
            &self.config,
            vacation_ids,
        )
        .await?;
        let mut by_vacation: HashMap<u32, Vec<app_core::vacation_app_db::VacationGroupApprovalRecord>> =
            HashMap::new();
        for record in records {
            by_vacation
                .entry(record.vacation_id)
                .or_default()
                .push(record);
        }
        let mut out = Vec::new();
        for id in vacation_ids {
            let enriched = self
                .enrich_group_approvals(by_vacation.remove(id).unwrap_or_default())
                .await?;
            out.push((*id, enriched));
        }
        Ok(out)
    }

    async fn update_vacation_with_permissions(
        &self,
        actor_id: EmployeeID,
        vacation_id: VacationID,
        start_date: Date,
        final_date: Date,
        details: &str,
    ) -> Result<Vacation, AppError> {
        let vacation = self.get_vacation(vacation_id).await?;
        let profile = self.build_employee_profile(actor_id).await?;
        // Only global editors may change vacation details before (or after) approval.
        // Owners cannot edit while the multi-group vote is outstanding.
        if !profile.permissions.edit_vacations {
            return Err(AppError::Forbidden);
        }
        let vacation = self
            .update_vacation_fields(
                vacation_id,
                vacation.employee_id(),
                start_date,
                final_date,
                details,
            )
            .await?;
        let today = Date::new(26, 1, 1);
        let _ = self
            .log_app_event(Event::new(
                today,
                EventType::VacationModified {
                    employee_id: vacation.employee_id(),
                    vacation_id,
                    modified_by_employee: Some(actor_id),
                    modified_by_user: None,
                },
            ))
            .await;
        Ok(vacation)
    }

    async fn delete_vacation_with_permissions(
        &self,
        actor_id: EmployeeID,
        vacation_id: VacationID,
    ) -> Result<(), AppError> {
        let vacation = self.get_vacation(vacation_id).await?;
        let profile = self.build_employee_profile(actor_id).await?;
        let is_owner = actor_id.id() == vacation.employee_id().id();
        let awaiting = matches!(vacation.status(), VacationStatus::AwaitingConfirmation);
        // Global deleters can always delete. Owners may still withdraw while awaiting.
        let can_delete = profile.permissions.delete_vacations || (is_owner && awaiting);
        if !can_delete {
            return Err(AppError::Forbidden);
        }
        let employee_id = vacation.employee_id();
        self.delete_vacation(vacation_id).await?;
        let today = Date::new(26, 1, 1);
        let _ = self
            .log_app_event(Event::new(
                today,
                EventType::VacationDeleted {
                    employee_id,
                    vacation_id,
                    deleted_by_employee: Some(actor_id),
                    deleted_by_user: None,
                },
            ))
            .await;
        Ok(())
    }

    async fn admin_list_org_vacations(
        &self,
        user_id: UserID,
    ) -> Result<Vec<Vacation>, AppError> {
        let user = self.get_user(user_id).await?;
        let org = self
            .get_organization(OrganizationID::new(user.organization_id()))
            .await?;
        let employee_ids: std::collections::HashSet<u32> =
            org.employee_ids().iter().copied().collect();
        let vacations = self.get_vacations().await?;
        Ok(vacations
            .into_iter()
            .filter(|v| employee_ids.contains(&v.employee_id().id()))
            .collect())
    }

    async fn admin_update_vacation(
        &self,
        user_id: UserID,
        vacation_id: VacationID,
        start_date: Date,
        final_date: Date,
        details: &str,
    ) -> Result<Vacation, AppError> {
        let vacation = self.get_vacation(vacation_id).await?;
        self.assert_admin_manages_employee(user_id, vacation.employee_id())
            .await?;
        let vacation = self
            .update_vacation_fields(
                vacation_id,
                vacation.employee_id(),
                start_date,
                final_date,
                details,
            )
            .await?;
        let today = Date::new(26, 1, 1);
        let _ = self
            .log_app_event(Event::new(
                today,
                EventType::VacationModified {
                    employee_id: vacation.employee_id(),
                    vacation_id,
                    modified_by_employee: None,
                    modified_by_user: Some(user_id),
                },
            ))
            .await;
        Ok(vacation)
    }

    async fn admin_approve_vacation(
        &self,
        user_id: UserID,
        vacation_id: VacationID,
    ) -> Result<Vacation, AppError> {
        let vacation = self.get_vacation(vacation_id).await?;
        self.assert_admin_manages_employee(user_id, vacation.employee_id())
            .await?;
        let owner = vacation.employee_id();
        let today = Date::new(26, 1, 1);

        match *vacation.status() {
            VacationStatus::AwaitingConfirmation => {
                // Force-approve: clear any pending group ballot first.
                app_core::vacation_app_db::delete_vacation_group_approvals(
                    &self.config,
                    vacation_id,
                )
                .await?;
                let vacation = self.approve_vacation(vacation_id, owner, owner).await?;
                let _ = self
                    .log_app_event(Event::new(
                        today,
                        EventType::VacationApproval {
                            employee_id: owner,
                            vacation_id,
                            approved_by: None,
                            approved_by_user: Some(user_id),
                        },
                    ))
                    .await;
                Ok(vacation)
            }
            VacationStatus::CancellationRequested { .. } => {
                app_core::vacation_app_db::delete_vacation_group_approvals(
                    &self.config,
                    vacation_id,
                )
                .await?;
                let vacation = self
                    .approve_vacation_cancellation(vacation_id, owner)
                    .await?;
                let _ = self
                    .log_app_event(Event::new(
                        today,
                        EventType::VacationCancellationApproved {
                            employee_id: owner,
                            vacation_id,
                            cancelled_by: None,
                            cancelled_by_user: Some(user_id),
                        },
                    ))
                    .await;
                Ok(vacation)
            }
            _ => Err(AppError::InvalidData),
        }
    }

    async fn admin_deny_vacation(
        &self,
        user_id: UserID,
        vacation_id: VacationID,
        reason: Option<&str>,
    ) -> Result<Vacation, AppError> {
        let reason = reason.unwrap_or("").trim();
        let vacation = self.get_vacation(vacation_id).await?;
        self.assert_admin_manages_employee(user_id, vacation.employee_id())
            .await?;
        let owner = vacation.employee_id();
        let today = Date::new(26, 1, 1);

        match *vacation.status() {
            VacationStatus::AwaitingConfirmation => {
                app_core::vacation_app_db::delete_vacation_group_approvals(
                    &self.config,
                    vacation_id,
                )
                .await?;
                let vacation = self.deny_vacation(vacation_id, owner, reason).await?;
                let _ = self
                    .log_app_event(Event::new(
                        today,
                        EventType::VacationDenial {
                            employee_id: owner,
                            vacation_id,
                            denied_by: None,
                            denied_by_user: Some(user_id),
                            reason: if reason.is_empty() {
                                None
                            } else {
                                Some(reason.to_string())
                            },
                        },
                    ))
                    .await;
                Ok(vacation)
            }
            VacationStatus::CancellationRequested {
                approvedBy,
                coveredBy,
                ..
            } => {
                app_core::vacation_app_db::delete_vacation_group_approvals(
                    &self.config,
                    vacation_id,
                )
                .await?;
                let vacation = self
                    .deny_vacation_cancellation(vacation_id, approvedBy, coveredBy)
                    .await?;
                let _ = self
                    .log_app_event(Event::new(
                        today,
                        EventType::VacationCancellationDenied {
                            employee_id: owner,
                            vacation_id,
                            denied_by: None,
                            denied_by_user: Some(user_id),
                        },
                    ))
                    .await;
                Ok(vacation)
            }
            _ => Err(AppError::InvalidData),
        }
    }

    async fn admin_delete_vacation(
        &self,
        user_id: UserID,
        vacation_id: VacationID,
    ) -> Result<(), AppError> {
        let vacation = self.get_vacation(vacation_id).await?;
        self.assert_admin_manages_employee(user_id, vacation.employee_id())
            .await?;
        let employee_id = vacation.employee_id();
        self.delete_vacation(vacation_id).await?;
        let today = Date::new(26, 1, 1);
        let _ = self
            .log_app_event(Event::new(
                today,
                EventType::VacationDeleted {
                    employee_id,
                    vacation_id,
                    deleted_by_employee: None,
                    deleted_by_user: Some(user_id),
                },
            ))
            .await;
        Ok(())
    }

    async fn admin_list_vacation_events(
        &self,
        user_id: UserID,
        vacation_id: VacationID,
    ) -> Result<Vec<app_core::vacation_app_db::EventRecord>, AppError> {
        let vacation = self.get_vacation(vacation_id).await?;
        self.assert_admin_manages_employee(user_id, vacation.employee_id())
            .await?;
        self.list_vacation_events(vacation_id).await
    }
}

pub mod server;

pub use server::{run, run_localhost};
