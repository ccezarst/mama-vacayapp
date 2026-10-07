use crate::config::VactionAppConfigs;
pub use crate::config::VacationDbSettings;
use crate::app_ext::{EmployeePermissions, GroupMemberPermissions};
use crate::db::DBQueryOps;
use crate::employees::{EmployeeID, Vacation, VacationID, VacationStatus};
use crate::events::{Event, EventType};
use crate::groups::GroupMembershipStore;
use crate::organizations::{Organization, OrganizationID};
use crate::users::UserID;
use crate::postgres::{PostgresConnector, PostgresDatabase, PostgresError, PostgresServer};
use crate::Date;

const VACATIONS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS vacations (
    "vacationID" SERIAL PRIMARY KEY,
    "employeeID" INTEGER NOT NULL,
    "startYear" SMALLINT NOT NULL,
    "startMonth" SMALLINT NOT NULL,
    "startDay" SMALLINT NOT NULL,
    "finalYear" SMALLINT NOT NULL,
    "finalMonth" SMALLINT NOT NULL,
    "finalDay" SMALLINT NOT NULL,
    "status" TEXT NOT NULL,
    "approvedBy" INTEGER,
    "coveredBy" INTEGER,
    "deniedBy" INTEGER,
    "details" TEXT NOT NULL DEFAULT '',
    "denialReason" TEXT NOT NULL DEFAULT ''
)
"#;

const EVENTS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS events (
    "eventID" SERIAL PRIMARY KEY,
    "eventType" TEXT NOT NULL,
    "actorID" INTEGER,
    "employeeID" INTEGER,
    "vacationID" INTEGER,
    "payload" TEXT NOT NULL DEFAULT '',
    "createdAt" TIMESTAMPTZ NOT NULL DEFAULT NOW()
)
"#;

const EMPLOYEES_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS employees (
    "employeeID" SERIAL PRIMARY KEY,
    "username" TEXT NOT NULL,
    "email" TEXT NOT NULL,
    "avatarUrl" TEXT
)
"#;

const ORGANIZATIONS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS organizations (
    "organizationID" SERIAL PRIMARY KEY,
    "ownerID" INTEGER NOT NULL,
    "name" TEXT NOT NULL
)
"#;

const USERS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    "userID" SERIAL PRIMARY KEY,
    "email" TEXT NOT NULL,
    "organizationID" INTEGER NOT NULL
)
"#;

const ORGANIZATION_EMPLOYEES_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS organization_employees (
    "organizationID" INTEGER NOT NULL,
    "employeeID" INTEGER NOT NULL,
    PRIMARY KEY ("organizationID", "employeeID")
)
"#;

const GROUPS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS groups (
    "groupID" SERIAL PRIMARY KEY,
    "organizationID" INTEGER NOT NULL,
    "name" TEXT NOT NULL
)
"#;

const EMPLOYEE_PERMISSIONS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS employee_permissions (
    "employeeID" INTEGER PRIMARY KEY,
    "createGroups" BOOLEAN NOT NULL DEFAULT false,
    "approveVacations" BOOLEAN NOT NULL DEFAULT false,
    "approveOwnVacations" BOOLEAN NOT NULL DEFAULT false,
    "requestVacation" BOOLEAN NOT NULL DEFAULT true,
    "editVacations" BOOLEAN NOT NULL DEFAULT false,
    "deleteVacations" BOOLEAN NOT NULL DEFAULT false,
    "editOwnAccount" BOOLEAN NOT NULL DEFAULT true
)
"#;

const GROUP_MEMBERS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS group_members (
    "groupID" INTEGER NOT NULL,
    "employeeID" INTEGER NOT NULL,
    PRIMARY KEY ("groupID", "employeeID")
)
"#;

const GROUP_MEMBER_PERMISSIONS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS group_member_permissions (
    "groupID" INTEGER NOT NULL,
    "employeeID" INTEGER NOT NULL,
    "canApproveVacations" BOOLEAN NOT NULL DEFAULT false,
    "canApproveOwnVacations" BOOLEAN NOT NULL DEFAULT false,
    "canEditGroup" BOOLEAN NOT NULL DEFAULT false,
    "canAddMembers" BOOLEAN NOT NULL DEFAULT false,
    "canRemoveMembers" BOOLEAN NOT NULL DEFAULT false,
    "canDeleteGroup" BOOLEAN NOT NULL DEFAULT false,
    PRIMARY KEY ("groupID", "employeeID")
)
"#;

const VACATION_GROUP_APPROVALS_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS vacation_group_approvals (
    "vacationID" INTEGER NOT NULL,
    "groupID" INTEGER NOT NULL,
    "status" TEXT NOT NULL DEFAULT 'pending',
    "decidedBy" INTEGER,
    PRIMARY KEY ("vacationID", "groupID")
)
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VacationGroupApprovalRecord {
    pub vacation_id: u32,
    pub group_id: u32,
    pub status: String,
    pub decided_by: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmployeeRecord {
    employee_id: EmployeeID,
    username: String,
    email: String,
    avatar_url: Option<String>,
}

impl EmployeeRecord {
    pub fn new(employee_id: EmployeeID, username: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            employee_id,
            username: username.into(),
            email: email.into(),
            avatar_url: None,
        }
    }

    pub fn employee_id(&self) -> EmployeeID {
        self.employee_id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn avatar_url(&self) -> Option<&str> {
        self.avatar_url.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserRecord {
    user_id: UserID,
    email: String,
    organization_id: u32,
}

impl UserRecord {
    pub fn new(user_id: UserID, email: impl Into<String>, organization_id: u32) -> Self {
        Self {
            user_id,
            email: email.into(),
            organization_id,
        }
    }

    pub fn user_id(&self) -> UserID {
        self.user_id
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn organization_id(&self) -> u32 {
        self.organization_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupRecord {
    group_id: u32,
    organization_id: u32,
    name: String,
}

impl GroupRecord {
    pub fn new(group_id: u32, organization_id: u32, name: impl Into<String>) -> Self {
        Self {
            group_id,
            organization_id,
            name: name.into(),
        }
    }

    pub fn group_id(&self) -> u32 {
        self.group_id
    }

    pub fn organization_id(&self) -> u32 {
        self.organization_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventRecord {
    pub id: u32,
    pub event_type: String,
    pub actor_id: Option<u32>,
    pub employee_id: Option<u32>,
    pub vacation_id: Option<u32>,
    pub payload: String,
    pub created_at: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum VacationAppDbError {
    Config(String),
    ConnectionFailed,
    QueryFailed,
    NotFound,
    InvalidData,
}

pub async fn setup_db(config: &VactionAppConfigs) -> Result<(), VacationAppDbError> {
    let db = connect_database(config).await?;
    let client = db.client();

    client
        .batch_execute(EMPLOYEES_TABLE_SQL)
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    client
        .batch_execute(
            r#"ALTER TABLE employees ADD COLUMN IF NOT EXISTS "avatarUrl" TEXT"#,
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    client
        .batch_execute(ORGANIZATIONS_TABLE_SQL)
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    client
        .batch_execute(USERS_TABLE_SQL)
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    client
        .batch_execute(VACATIONS_TABLE_SQL)
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    client
        .batch_execute(
            r#"ALTER TABLE vacations ADD COLUMN IF NOT EXISTS "details" TEXT NOT NULL DEFAULT ''"#,
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    client
        .batch_execute(
            r#"ALTER TABLE vacations ADD COLUMN IF NOT EXISTS "denialReason" TEXT NOT NULL DEFAULT ''"#,
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    client
        .batch_execute(ORGANIZATION_EMPLOYEES_TABLE_SQL)
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    client
        .batch_execute(GROUPS_TABLE_SQL)
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    client
        .batch_execute(EMPLOYEE_PERMISSIONS_TABLE_SQL)
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    client
        .batch_execute(GROUP_MEMBERS_TABLE_SQL)
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    client
        .batch_execute(GROUP_MEMBER_PERMISSIONS_TABLE_SQL)
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    client
        .batch_execute(
            r#"ALTER TABLE group_member_permissions ADD COLUMN IF NOT EXISTS "canApproveOwnVacations" BOOLEAN NOT NULL DEFAULT false"#,
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    client
        .batch_execute(VACATION_GROUP_APPROVALS_TABLE_SQL)
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    client
        .batch_execute(EVENTS_TABLE_SQL)
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    if config.vacation_db.auto_setup_auth {
        crate::auth::setup_auth(config).await.map_err(map_auth_error)?;
    }

    Ok(())
}

pub async fn get_vacations(config: &VactionAppConfigs) -> Result<Vec<Vacation>, VacationAppDbError> {
    let db = connect_database(config).await?;
    load_vacations(db.client(), None).await
}

pub async fn get_vacations_for_employee(
    config: &VactionAppConfigs,
    employee_id: EmployeeID,
) -> Result<Vec<Vacation>, VacationAppDbError> {
    let db = connect_database(config).await?;
    load_vacations(db.client(), Some(employee_id)).await
}

pub async fn get_vacation(
    config: &VactionAppConfigs,
    vacation_id: VacationID,
) -> Result<Vacation, VacationAppDbError> {
    let db = connect_database(config).await?;
    let rows = db
        .client()
        .query(
            r#"
            SELECT "vacationID", "employeeID",
                   "startYear", "startMonth", "startDay",
                   "finalYear", "finalMonth", "finalDay",
                   "status", "approvedBy", "coveredBy", "deniedBy",
                   "details", "denialReason"
            FROM vacations
            WHERE "vacationID" = $1
            "#,
            &[&(vacation_id.id() as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    rows.first()
        .ok_or(VacationAppDbError::NotFound)
        .and_then(row_to_vacation)
}

pub async fn request_vacation(
    config: &VactionAppConfigs,
    employee_id: EmployeeID,
    start_date: Date,
    final_date: Date,
) -> Result<Vacation, VacationAppDbError> {
    request_vacation_with_details(config, employee_id, start_date, final_date, "").await
}

pub async fn request_vacation_with_details(
    config: &VactionAppConfigs,
    employee_id: EmployeeID,
    start_date: Date,
    final_date: Date,
    details: &str,
) -> Result<Vacation, VacationAppDbError> {
    let db = connect_database(config).await?;
    let row = db
        .client()
        .query_one(
            r#"
            INSERT INTO vacations (
                "employeeID",
                "startYear", "startMonth", "startDay",
                "finalYear", "finalMonth", "finalDay",
                "status", "details"
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING "vacationID", "employeeID",
                      "startYear", "startMonth", "startDay",
                      "finalYear", "finalMonth", "finalDay",
                      "status", "approvedBy", "coveredBy", "deniedBy",
                      "details", "denialReason"
            "#,
            &[
                &(employee_id.id() as i32),
                &(start_date.year() as i16),
                &(start_date.month() as i16),
                &(start_date.day() as i16),
                &(final_date.year() as i16),
                &(final_date.month() as i16),
                &(final_date.day() as i16),
                &"awaiting",
                &details,
            ],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    row_to_vacation(&row)
}

pub async fn approve_vacation(
    config: &VactionAppConfigs,
    vacation_id: VacationID,
    approved_by: EmployeeID,
    covered_by: EmployeeID,
) -> Result<Vacation, VacationAppDbError> {
    update_vacation_status(
        config,
        vacation_id,
        "approved",
        Some(approved_by),
        Some(covered_by),
        None,
    )
    .await
}

pub async fn deny_vacation(
    config: &VactionAppConfigs,
    vacation_id: VacationID,
    denied_by: EmployeeID,
) -> Result<Vacation, VacationAppDbError> {
    deny_vacation_with_reason(config, vacation_id, denied_by, "").await
}

pub async fn deny_vacation_with_reason(
    config: &VactionAppConfigs,
    vacation_id: VacationID,
    denied_by: EmployeeID,
    reason: &str,
) -> Result<Vacation, VacationAppDbError> {
    let db = connect_database(config).await?;
    let row = db
        .client()
        .query_one(
            r#"
            UPDATE vacations
            SET "status" = $2,
                "approvedBy" = NULL,
                "coveredBy" = NULL,
                "deniedBy" = $3,
                "denialReason" = $4
            WHERE "vacationID" = $1
            RETURNING "vacationID", "employeeID",
                      "startYear", "startMonth", "startDay",
                      "finalYear", "finalMonth", "finalDay",
                      "status", "approvedBy", "coveredBy", "deniedBy",
                      "details", "denialReason"
            "#,
            &[
                &(vacation_id.id() as i32),
                &"denied",
                &(denied_by.id() as i32),
                &reason,
            ],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    row_to_vacation(&row)
}

pub async fn request_vacation_cancellation(
    config: &VactionAppConfigs,
    vacation_id: VacationID,
    requested_by: EmployeeID,
    approved_by: EmployeeID,
    covered_by: EmployeeID,
) -> Result<Vacation, VacationAppDbError> {
    // Reuse deniedBy column for the cancellation requester while preserving approval fields.
    update_vacation_status(
        config,
        vacation_id,
        "cancellation_requested",
        Some(approved_by),
        Some(covered_by),
        Some(requested_by),
    )
    .await
}

pub async fn approve_vacation_cancellation(
    config: &VactionAppConfigs,
    vacation_id: VacationID,
    cancelled_by: EmployeeID,
) -> Result<Vacation, VacationAppDbError> {
    update_vacation_status(
        config,
        vacation_id,
        "cancelled",
        None,
        None,
        Some(cancelled_by),
    )
    .await
}

pub async fn deny_vacation_cancellation(
    config: &VactionAppConfigs,
    vacation_id: VacationID,
    approved_by: EmployeeID,
    covered_by: EmployeeID,
) -> Result<Vacation, VacationAppDbError> {
    update_vacation_status(
        config,
        vacation_id,
        "approved",
        Some(approved_by),
        Some(covered_by),
        None,
    )
    .await
}

pub async fn seed_vacation_group_approvals(
    config: &VactionAppConfigs,
    vacation_id: VacationID,
    group_ids: &[u32],
) -> Result<(), VacationAppDbError> {
    if group_ids.is_empty() {
        return Ok(());
    }
    let db = connect_database(config).await?;
    let client = db.client();
    for group_id in group_ids {
        client
            .execute(
                r#"
                INSERT INTO vacation_group_approvals ("vacationID", "groupID", "status")
                VALUES ($1, $2, 'pending')
                ON CONFLICT ("vacationID", "groupID") DO NOTHING
                "#,
                &[&(vacation_id.id() as i32), &(*group_id as i32)],
            )
            .await
            .map_err(|_| VacationAppDbError::QueryFailed)?;
    }
    Ok(())
}

pub async fn list_vacation_group_approvals(
    config: &VactionAppConfigs,
    vacation_id: VacationID,
) -> Result<Vec<VacationGroupApprovalRecord>, VacationAppDbError> {
    let db = connect_database(config).await?;
    let rows = db
        .client()
        .query(
            r#"
            SELECT "vacationID", "groupID", "status", "decidedBy"
            FROM vacation_group_approvals
            WHERE "vacationID" = $1
            ORDER BY "groupID"
            "#,
            &[&(vacation_id.id() as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    Ok(rows
        .iter()
        .map(|row| {
            let decided: Option<i32> = row.get(3);
            VacationGroupApprovalRecord {
                vacation_id: row.get::<_, i32>(0) as u32,
                group_id: row.get::<_, i32>(1) as u32,
                status: row.get(2),
                decided_by: decided.map(|id| id as u32),
            }
        })
        .collect())
}

pub async fn list_vacation_group_approvals_for_many(
    config: &VactionAppConfigs,
    vacation_ids: &[u32],
) -> Result<Vec<VacationGroupApprovalRecord>, VacationAppDbError> {
    if vacation_ids.is_empty() {
        return Ok(Vec::new());
    }
    let db = connect_database(config).await?;
    let ids: Vec<i32> = vacation_ids.iter().map(|id| *id as i32).collect();
    let rows = db
        .client()
        .query(
            r#"
            SELECT "vacationID", "groupID", "status", "decidedBy"
            FROM vacation_group_approvals
            WHERE "vacationID" = ANY($1)
            ORDER BY "vacationID", "groupID"
            "#,
            &[&ids],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    Ok(rows
        .iter()
        .map(|row| {
            let decided: Option<i32> = row.get(3);
            VacationGroupApprovalRecord {
                vacation_id: row.get::<_, i32>(0) as u32,
                group_id: row.get::<_, i32>(1) as u32,
                status: row.get(2),
                decided_by: decided.map(|id| id as u32),
            }
        })
        .collect())
}

pub async fn record_vacation_group_decision(
    config: &VactionAppConfigs,
    vacation_id: VacationID,
    group_id: u32,
    status: &str,
    decided_by: EmployeeID,
) -> Result<VacationGroupApprovalRecord, VacationAppDbError> {
    let db = connect_database(config).await?;
    let updated = db
        .client()
        .execute(
            r#"
            UPDATE vacation_group_approvals
            SET "status" = $3, "decidedBy" = $4
            WHERE "vacationID" = $1 AND "groupID" = $2 AND "status" = 'pending'
            "#,
            &[
                &(vacation_id.id() as i32),
                &(group_id as i32),
                &status,
                &(decided_by.id() as i32),
            ],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    if updated == 0 {
        return Err(VacationAppDbError::NotFound);
    }

    let rows = list_vacation_group_approvals(config, vacation_id).await?;
    rows.into_iter()
        .find(|row| row.group_id == group_id)
        .ok_or(VacationAppDbError::NotFound)
}

pub async fn delete_vacation_group_approvals(
    config: &VactionAppConfigs,
    vacation_id: VacationID,
) -> Result<(), VacationAppDbError> {
    let db = connect_database(config).await?;
    db.client()
        .execute(
            r#"DELETE FROM vacation_group_approvals WHERE "vacationID" = $1"#,
            &[&(vacation_id.id() as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    Ok(())
}

pub async fn update_vacation(
    config: &VactionAppConfigs,
    vacation_id: VacationID,
    employee_id: EmployeeID,
    start_date: Date,
    final_date: Date,
) -> Result<Vacation, VacationAppDbError> {
    let existing = get_vacation(config, vacation_id).await?;
    update_vacation_details(
        config,
        vacation_id,
        employee_id,
        start_date,
        final_date,
        existing.details(),
    )
    .await
}

pub async fn update_vacation_details(
    config: &VactionAppConfigs,
    vacation_id: VacationID,
    employee_id: EmployeeID,
    start_date: Date,
    final_date: Date,
    details: &str,
) -> Result<Vacation, VacationAppDbError> {
    let db = connect_database(config).await?;
    let row = db
        .client()
        .query_one(
            r#"
            UPDATE vacations
            SET "employeeID" = $2,
                "startYear" = $3,
                "startMonth" = $4,
                "startDay" = $5,
                "finalYear" = $6,
                "finalMonth" = $7,
                "finalDay" = $8,
                "details" = $9
            WHERE "vacationID" = $1
            RETURNING "vacationID", "employeeID",
                      "startYear", "startMonth", "startDay",
                      "finalYear", "finalMonth", "finalDay",
                      "status", "approvedBy", "coveredBy", "deniedBy",
                      "details", "denialReason"
            "#,
            &[
                &(vacation_id.id() as i32),
                &(employee_id.id() as i32),
                &(start_date.year() as i16),
                &(start_date.month() as i16),
                &(start_date.day() as i16),
                &(final_date.year() as i16),
                &(final_date.month() as i16),
                &(final_date.day() as i16),
                &details,
            ],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    row_to_vacation(&row)
}

pub async fn log_event(
    config: &VactionAppConfigs,
    event: &Event,
) -> Result<EventRecord, VacationAppDbError> {
    let db = connect_database(config).await?;
    let (actor_id, employee_id, vacation_id, payload) = match event.event_type() {
        EventType::VacationRequest {
            employee_id,
            vacation_id,
        } => (
            Some(employee_id.id() as i32),
            Some(employee_id.id() as i32),
            Some(vacation_id.id() as i32),
            String::new(),
        ),
        EventType::VacationApproval {
            employee_id,
            vacation_id,
            approved_by,
            approved_by_user,
        } => {
            if let Some(actor_id) = approved_by {
                (
                    Some(actor_id.id() as i32),
                    Some(employee_id.id() as i32),
                    Some(vacation_id.id() as i32),
                    format!("approved_by={}", actor_id.id()),
                )
            } else if let Some(user_id) = approved_by_user {
                (
                    None,
                    Some(employee_id.id() as i32),
                    Some(vacation_id.id() as i32),
                    format!("admin_user_id={}", user_id.id()),
                )
            } else {
                (
                    None,
                    Some(employee_id.id() as i32),
                    Some(vacation_id.id() as i32),
                    String::new(),
                )
            }
        }
        EventType::VacationDenial {
            employee_id,
            vacation_id,
            denied_by,
            denied_by_user,
            reason,
        } => {
            let mut payload = if let Some(actor_id) = denied_by {
                format!("denied_by={}", actor_id.id())
            } else if let Some(user_id) = denied_by_user {
                format!("admin_user_id={}", user_id.id())
            } else {
                String::new()
            };
            if let Some(reason) = reason {
                if !reason.is_empty() {
                    if !payload.is_empty() {
                        payload.push(';');
                    }
                    payload.push_str("reason=");
                    payload.push_str(&reason.replace(';', ","));
                }
            }
            if let Some(actor_id) = denied_by {
                (
                    Some(actor_id.id() as i32),
                    Some(employee_id.id() as i32),
                    Some(vacation_id.id() as i32),
                    payload,
                )
            } else {
                (
                    None,
                    Some(employee_id.id() as i32),
                    Some(vacation_id.id() as i32),
                    payload,
                )
            }
        }
        EventType::VacationGroupVote {
            employee_id,
            vacation_id,
            group_id,
            decided_by,
            approved,
        } => (
            Some(decided_by.id() as i32),
            Some(employee_id.id() as i32),
            Some(vacation_id.id() as i32),
            format!(
                "group_id={};decided_by={};vote={}",
                group_id,
                decided_by.id(),
                if *approved { "approved" } else { "denied" }
            ),
        ),
        EventType::VacationModified {
            employee_id,
            vacation_id,
            modified_by_employee,
            modified_by_user,
        } => {
            if let Some(actor_id) = modified_by_employee {
                (
                    Some(actor_id.id() as i32),
                    Some(employee_id.id() as i32),
                    Some(vacation_id.id() as i32),
                    format!("modified_by={}", actor_id.id()),
                )
            } else if let Some(user_id) = modified_by_user {
                (
                    None,
                    Some(employee_id.id() as i32),
                    Some(vacation_id.id() as i32),
                    format!("admin_user_id={}", user_id.id()),
                )
            } else {
                (
                    None,
                    Some(employee_id.id() as i32),
                    Some(vacation_id.id() as i32),
                    String::new(),
                )
            }
        }
        EventType::VacationDeleted {
            employee_id,
            vacation_id,
            deleted_by_employee,
            deleted_by_user,
        } => {
            if let Some(actor_id) = deleted_by_employee {
                (
                    Some(actor_id.id() as i32),
                    Some(employee_id.id() as i32),
                    Some(vacation_id.id() as i32),
                    format!("deleted_by={}", actor_id.id()),
                )
            } else if let Some(user_id) = deleted_by_user {
                (
                    None,
                    Some(employee_id.id() as i32),
                    Some(vacation_id.id() as i32),
                    format!("admin_user_id={}", user_id.id()),
                )
            } else {
                (
                    None,
                    Some(employee_id.id() as i32),
                    Some(vacation_id.id() as i32),
                    String::new(),
                )
            }
        }
        EventType::VacationCancellationRequested {
            employee_id,
            vacation_id,
            requested_by,
        } => (
            Some(requested_by.id() as i32),
            Some(employee_id.id() as i32),
            Some(vacation_id.id() as i32),
            format!("requested_by={}", requested_by.id()),
        ),
        EventType::VacationCancellationApproved {
            employee_id,
            vacation_id,
            cancelled_by,
            cancelled_by_user,
        } => {
            if let Some(actor_id) = cancelled_by {
                (
                    Some(actor_id.id() as i32),
                    Some(employee_id.id() as i32),
                    Some(vacation_id.id() as i32),
                    format!("cancelled_by={}", actor_id.id()),
                )
            } else if let Some(user_id) = cancelled_by_user {
                (
                    None,
                    Some(employee_id.id() as i32),
                    Some(vacation_id.id() as i32),
                    format!("admin_user_id={}", user_id.id()),
                )
            } else {
                (
                    None,
                    Some(employee_id.id() as i32),
                    Some(vacation_id.id() as i32),
                    String::new(),
                )
            }
        }
        EventType::VacationCancellationDenied {
            employee_id,
            vacation_id,
            denied_by,
            denied_by_user,
        } => {
            if let Some(actor_id) = denied_by {
                (
                    Some(actor_id.id() as i32),
                    Some(employee_id.id() as i32),
                    Some(vacation_id.id() as i32),
                    format!("denied_by={}", actor_id.id()),
                )
            } else if let Some(user_id) = denied_by_user {
                (
                    None,
                    Some(employee_id.id() as i32),
                    Some(vacation_id.id() as i32),
                    format!("admin_user_id={}", user_id.id()),
                )
            } else {
                (
                    None,
                    Some(employee_id.id() as i32),
                    Some(vacation_id.id() as i32),
                    String::new(),
                )
            }
        }
        EventType::EmployeeAdded {
            employee_id,
            added_by,
        } => (
            None,
            Some(employee_id.id() as i32),
            None,
            format!("admin_user_id={}", added_by.id()),
        ),
        EventType::EmployeeModified {
            employee_id,
            modified_by_user,
            modified_by_employee,
        } => {
            if let Some(user_id) = modified_by_user {
                (
                    None,
                    Some(employee_id.id() as i32),
                    None,
                    format!("admin_user_id={}", user_id.id()),
                )
            } else if let Some(actor_id) = modified_by_employee {
                (
                    Some(actor_id.id() as i32),
                    Some(employee_id.id() as i32),
                    None,
                    String::new(),
                )
            } else {
                (None, Some(employee_id.id() as i32), None, String::new())
            }
        }
        EventType::EmployeeDeleted {
            employee_id,
            deleted_by,
        } => (
            None,
            Some(employee_id.id() as i32),
            None,
            format!("admin_user_id={}", deleted_by.id()),
        ),
        other => (None, None, None, other.type_name().to_string()),
    };

    let row = db
        .client()
        .query_one(
            r#"
            INSERT INTO events ("eventType", "actorID", "employeeID", "vacationID", "payload")
            VALUES ($1, $2, $3, $4, $5)
            RETURNING "eventID", "eventType", "actorID", "employeeID", "vacationID", "payload",
                      to_char("createdAt" AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"')
            "#,
            &[
                &event.event_type().type_name(),
                &actor_id,
                &employee_id,
                &vacation_id,
                &payload,
            ],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    Ok(EventRecord {
        id: row.get::<_, i32>(0) as u32,
        event_type: row.get(1),
        actor_id: row.get::<_, Option<i32>>(2).map(|v| v as u32),
        employee_id: row.get::<_, Option<i32>>(3).map(|v| v as u32),
        vacation_id: row.get::<_, Option<i32>>(4).map(|v| v as u32),
        payload: row.get(5),
        created_at: row.get(6),
    })
}

pub async fn list_events_for_vacation(
    config: &VactionAppConfigs,
    vacation_id: VacationID,
) -> Result<Vec<EventRecord>, VacationAppDbError> {
    let db = connect_database(config).await?;
    let rows = db
        .client()
        .query(
            r#"
            SELECT "eventID", "eventType", "actorID", "employeeID", "vacationID", "payload",
                   to_char("createdAt" AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"')
            FROM events
            WHERE "vacationID" = $1
            ORDER BY "eventID" DESC
            "#,
            &[&(vacation_id.id() as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    Ok(rows
        .into_iter()
        .map(|row| EventRecord {
            id: row.get::<_, i32>(0) as u32,
            event_type: row.get(1),
            actor_id: row.get::<_, Option<i32>>(2).map(|v| v as u32),
            employee_id: row.get::<_, Option<i32>>(3).map(|v| v as u32),
            vacation_id: row.get::<_, Option<i32>>(4).map(|v| v as u32),
            payload: row.get(5),
            created_at: row.get(6),
        })
        .collect())
}

pub async fn list_all_events(
    config: &VactionAppConfigs,
) -> Result<Vec<EventRecord>, VacationAppDbError> {
    let db = connect_database(config).await?;
    let rows = db
        .client()
        .query(
            r#"
            SELECT "eventID", "eventType", "actorID", "employeeID", "vacationID", "payload",
                   to_char("createdAt" AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"')
            FROM events
            ORDER BY "eventID" DESC
            "#,
            &[],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    Ok(rows
        .into_iter()
        .map(|row| EventRecord {
            id: row.get::<_, i32>(0) as u32,
            event_type: row.get(1),
            actor_id: row.get::<_, Option<i32>>(2).map(|v| v as u32),
            employee_id: row.get::<_, Option<i32>>(3).map(|v| v as u32),
            vacation_id: row.get::<_, Option<i32>>(4).map(|v| v as u32),
            payload: row.get(5),
            created_at: row.get(6),
        })
        .collect())
}

pub async fn delete_vacation(
    config: &VactionAppConfigs,
    vacation_id: VacationID,
) -> Result<(), VacationAppDbError> {
    let db = connect_database(config).await?;
    let client = db.client();
    client
        .execute(
            r#"DELETE FROM vacation_group_approvals WHERE "vacationID" = $1"#,
            &[&(vacation_id.id() as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    let deleted = client
        .execute(
            r#"DELETE FROM vacations WHERE "vacationID" = $1"#,
            &[&(vacation_id.id() as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    if deleted == 0 {
        Err(VacationAppDbError::NotFound)
    } else {
        Ok(())
    }
}

pub async fn create_employee(
    config: &VactionAppConfigs,
    username: &str,
    email: &str,
) -> Result<EmployeeRecord, VacationAppDbError> {
    let db = connect_database(config).await?;
    let row = db
        .client()
        .query_one(
            r#"
            INSERT INTO employees ("username", "email")
            VALUES ($1, $2)
            RETURNING "employeeID", "username", "email", "avatarUrl"
            "#,
            &[&username, &email],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    row_to_employee(&row)
}

pub async fn get_employee(
    config: &VactionAppConfigs,
    employee_id: EmployeeID,
) -> Result<EmployeeRecord, VacationAppDbError> {
    let db = connect_database(config).await?;
    let rows = db
        .client()
        .query(
            r#"
            SELECT "employeeID", "username", "email", "avatarUrl"
            FROM employees
            WHERE "employeeID" = $1
            "#,
            &[&(employee_id.id() as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    rows.first()
        .ok_or(VacationAppDbError::NotFound)
        .and_then(row_to_employee)
}

pub async fn list_employees(
    config: &VactionAppConfigs,
) -> Result<Vec<EmployeeRecord>, VacationAppDbError> {
    let db = connect_database(config).await?;
    let rows = db
        .client()
        .query(
            r#"
            SELECT "employeeID", "username", "email", "avatarUrl"
            FROM employees
            ORDER BY "employeeID"
            "#,
            &[],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    rows.iter().map(row_to_employee).collect()
}

pub async fn update_employee(
    config: &VactionAppConfigs,
    employee_id: EmployeeID,
    username: &str,
    email: &str,
) -> Result<EmployeeRecord, VacationAppDbError> {
    let db = connect_database(config).await?;
    let row = db
        .client()
        .query_opt(
            r#"
            UPDATE employees
            SET "username" = $2, "email" = $3
            WHERE "employeeID" = $1
            RETURNING "employeeID", "username", "email", "avatarUrl"
            "#,
            &[&(employee_id.id() as i32), &username, &email],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?
        .ok_or(VacationAppDbError::NotFound)?;

    row_to_employee(&row)
}

pub async fn update_employee_avatar(
    config: &VactionAppConfigs,
    employee_id: EmployeeID,
    avatar_url: Option<&str>,
) -> Result<EmployeeRecord, VacationAppDbError> {
    let db = connect_database(config).await?;
    let row = db
        .client()
        .query_opt(
            r#"
            UPDATE employees
            SET "avatarUrl" = $2
            WHERE "employeeID" = $1
            RETURNING "employeeID", "username", "email", "avatarUrl"
            "#,
            &[&(employee_id.id() as i32), &avatar_url],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?
        .ok_or(VacationAppDbError::NotFound)?;

    row_to_employee(&row)
}

pub async fn delete_employee(
    config: &VactionAppConfigs,
    employee_id: EmployeeID,
) -> Result<(), VacationAppDbError> {
    let db = connect_database(config).await?;
    delete_employee_group_data(db.client(), employee_id.id()).await?;
    let deleted = db
        .client()
        .execute(
            r#"DELETE FROM employees WHERE "employeeID" = $1"#,
            &[&(employee_id.id() as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    if deleted == 0 {
        Err(VacationAppDbError::NotFound)
    } else {
        Ok(())
    }
}

pub async fn create_organization(
    config: &VactionAppConfigs,
    owner_id: u32,
    name: &str,
) -> Result<Organization, VacationAppDbError> {
    let db = connect_database(config).await?;
    let row = db
        .client()
        .query_one(
            r#"
            INSERT INTO organizations ("ownerID", "name")
            VALUES ($1, $2)
            RETURNING "organizationID", "ownerID", "name"
            "#,
            &[&(owner_id as i32), &name],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    row_to_organization(&row, &[])
}

pub async fn get_organization(
    config: &VactionAppConfigs,
    organization_id: OrganizationID,
) -> Result<Organization, VacationAppDbError> {
    let db = connect_database(config).await?;
    let row = db
        .client()
        .query_opt(
            r#"
            SELECT "organizationID", "ownerID", "name"
            FROM organizations
            WHERE "organizationID" = $1
            "#,
            &[&(organization_id.id() as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?
        .ok_or(VacationAppDbError::NotFound)?;

    let employee_ids = load_organization_employee_ids(db.client(), organization_id).await?;
    row_to_organization(&row, &employee_ids)
}

pub async fn list_organizations(
    config: &VactionAppConfigs,
) -> Result<Vec<Organization>, VacationAppDbError> {
    let db = connect_database(config).await?;
    let rows = db
        .client()
        .query(
            r#"
            SELECT "organizationID", "ownerID", "name"
            FROM organizations
            ORDER BY "organizationID"
            "#,
            &[],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    let mut organizations = Vec::with_capacity(rows.len());
    for row in rows {
        let organization_id = OrganizationID::new(row.get::<_, i32>(0) as u32);
        let employee_ids = load_organization_employee_ids(db.client(), organization_id).await?;
        organizations.push(row_to_organization(&row, &employee_ids)?);
    }
    Ok(organizations)
}

pub async fn update_organization(
    config: &VactionAppConfigs,
    organization_id: OrganizationID,
    owner_id: u32,
    name: &str,
) -> Result<Organization, VacationAppDbError> {
    let db = connect_database(config).await?;
    let row = db
        .client()
        .query_opt(
            r#"
            UPDATE organizations
            SET "ownerID" = $2, "name" = $3
            WHERE "organizationID" = $1
            RETURNING "organizationID", "ownerID", "name"
            "#,
            &[
                &(organization_id.id() as i32),
                &(owner_id as i32),
                &name,
            ],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?
        .ok_or(VacationAppDbError::NotFound)?;

    let employee_ids = load_organization_employee_ids(db.client(), organization_id).await?;
    row_to_organization(&row, &employee_ids)
}

pub async fn delete_organization(
    config: &VactionAppConfigs,
    organization_id: OrganizationID,
) -> Result<(), VacationAppDbError> {
    let db = connect_database(config).await?;
    db.client()
        .execute(
            r#"DELETE FROM organization_employees WHERE "organizationID" = $1"#,
            &[&(organization_id.id() as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    db.client()
        .execute(
            r#"DELETE FROM groups WHERE "organizationID" = $1"#,
            &[&(organization_id.id() as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    let deleted = db
        .client()
        .execute(
            r#"DELETE FROM organizations WHERE "organizationID" = $1"#,
            &[&(organization_id.id() as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    if deleted == 0 {
        Err(VacationAppDbError::NotFound)
    } else {
        Ok(())
    }
}

pub async fn add_employee_to_organization(
    config: &VactionAppConfigs,
    organization_id: OrganizationID,
    employee_id: EmployeeID,
) -> Result<(), VacationAppDbError> {
    let db = connect_database(config).await?;
    db.client()
        .execute(
            r#"
            INSERT INTO organization_employees ("organizationID", "employeeID")
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
            &[
                &(organization_id.id() as i32),
                &(employee_id.id() as i32),
            ],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    Ok(())
}

pub async fn remove_employee_from_organization(
    config: &VactionAppConfigs,
    organization_id: OrganizationID,
    employee_id: EmployeeID,
) -> Result<(), VacationAppDbError> {
    let db = connect_database(config).await?;
    let deleted = db
        .client()
        .execute(
            r#"
            DELETE FROM organization_employees
            WHERE "organizationID" = $1 AND "employeeID" = $2
            "#,
            &[
                &(organization_id.id() as i32),
                &(employee_id.id() as i32),
            ],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    if deleted == 0 {
        Err(VacationAppDbError::NotFound)
    } else {
        Ok(())
    }
}

pub async fn create_user(
    config: &VactionAppConfigs,
    email: &str,
    organization_id: u32,
) -> Result<UserRecord, VacationAppDbError> {
    let db = connect_database(config).await?;
    let row = db
        .client()
        .query_one(
            r#"
            INSERT INTO users ("email", "organizationID")
            VALUES ($1, $2)
            RETURNING "userID", "email", "organizationID"
            "#,
            &[&email, &(organization_id as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    row_to_user(&row)
}

pub async fn get_user(
    config: &VactionAppConfigs,
    user_id: UserID,
) -> Result<UserRecord, VacationAppDbError> {
    let db = connect_database(config).await?;
    let rows = db
        .client()
        .query(
            r#"
            SELECT "userID", "email", "organizationID"
            FROM users
            WHERE "userID" = $1
            "#,
            &[&(user_id.id() as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    rows.first()
        .ok_or(VacationAppDbError::NotFound)
        .and_then(row_to_user)
}

pub async fn list_users(
    config: &VactionAppConfigs,
) -> Result<Vec<UserRecord>, VacationAppDbError> {
    let db = connect_database(config).await?;
    let rows = db
        .client()
        .query(
            r#"
            SELECT "userID", "email", "organizationID"
            FROM users
            ORDER BY "userID"
            "#,
            &[],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    rows.iter().map(row_to_user).collect()
}

pub async fn list_users_for_organization(
    config: &VactionAppConfigs,
    organization_id: u32,
) -> Result<Vec<UserRecord>, VacationAppDbError> {
    let db = connect_database(config).await?;
    let rows = db
        .client()
        .query(
            r#"
            SELECT "userID", "email", "organizationID"
            FROM users
            WHERE "organizationID" = $1
            ORDER BY "userID"
            "#,
            &[&(organization_id as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    rows.iter().map(row_to_user).collect()
}

pub async fn update_user(
    config: &VactionAppConfigs,
    user_id: UserID,
    email: &str,
    organization_id: u32,
) -> Result<UserRecord, VacationAppDbError> {
    let db = connect_database(config).await?;
    let row = db
        .client()
        .query_opt(
            r#"
            UPDATE users
            SET "email" = $2, "organizationID" = $3
            WHERE "userID" = $1
            RETURNING "userID", "email", "organizationID"
            "#,
            &[
                &(user_id.id() as i32),
                &email,
                &(organization_id as i32),
            ],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?
        .ok_or(VacationAppDbError::NotFound)?;

    row_to_user(&row)
}

pub async fn delete_user(
    config: &VactionAppConfigs,
    user_id: UserID,
) -> Result<(), VacationAppDbError> {
    let db = connect_database(config).await?;
    let deleted = db
        .client()
        .execute(
            r#"DELETE FROM users WHERE "userID" = $1"#,
            &[&(user_id.id() as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    if deleted == 0 {
        Err(VacationAppDbError::NotFound)
    } else {
        Ok(())
    }
}

pub async fn create_group(
    config: &VactionAppConfigs,
    organization_id: u32,
    name: &str,
) -> Result<GroupRecord, VacationAppDbError> {
    let db = connect_database(config).await?;
    let row = db
        .client()
        .query_one(
            r#"
            INSERT INTO groups ("organizationID", "name")
            VALUES ($1, $2)
            RETURNING "groupID", "organizationID", "name"
            "#,
            &[&(organization_id as i32), &name],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    row_to_group(&row)
}

pub async fn get_group(
    config: &VactionAppConfigs,
    group_id: u32,
) -> Result<GroupRecord, VacationAppDbError> {
    let db = connect_database(config).await?;
    let rows = db
        .client()
        .query(
            r#"
            SELECT "groupID", "organizationID", "name"
            FROM groups
            WHERE "groupID" = $1
            "#,
            &[&(group_id as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    rows.first()
        .ok_or(VacationAppDbError::NotFound)
        .and_then(row_to_group)
}

pub async fn list_groups_for_organization(
    config: &VactionAppConfigs,
    organization_id: u32,
) -> Result<Vec<GroupRecord>, VacationAppDbError> {
    let db = connect_database(config).await?;
    let rows = db
        .client()
        .query(
            r#"
            SELECT "groupID", "organizationID", "name"
            FROM groups
            WHERE "organizationID" = $1
            ORDER BY "groupID"
            "#,
            &[&(organization_id as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    rows.iter().map(row_to_group).collect()
}

pub async fn update_group(
    config: &VactionAppConfigs,
    group_id: u32,
    name: &str,
) -> Result<GroupRecord, VacationAppDbError> {
    let db = connect_database(config).await?;
    let row = db
        .client()
        .query_opt(
            r#"
            UPDATE groups
            SET "name" = $2
            WHERE "groupID" = $1
            RETURNING "groupID", "organizationID", "name"
            "#,
            &[&(group_id as i32), &name],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?
        .ok_or(VacationAppDbError::NotFound)?;

    row_to_group(&row)
}

pub async fn delete_group(
    config: &VactionAppConfigs,
    group_id: u32,
) -> Result<(), VacationAppDbError> {
    let db = connect_database(config).await?;
    delete_group_membership_data(db.client(), group_id).await?;
    let deleted = db
        .client()
        .execute(r#"DELETE FROM groups WHERE "groupID" = $1"#, &[&(group_id as i32)])
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    if deleted == 0 {
        Err(VacationAppDbError::NotFound)
    } else {
        Ok(())
    }
}

pub async fn load_group_membership_store(
    config: &VactionAppConfigs,
) -> Result<GroupMembershipStore, VacationAppDbError> {
    let db = connect_database(config).await?;
    let client = db.client();
    let mut store = GroupMembershipStore::new();

    let permission_rows = client
        .query(
            r#"
            SELECT "employeeID",
                   "createGroups",
                   "approveVacations",
                   "approveOwnVacations",
                   "requestVacation",
                   "editVacations",
                   "deleteVacations",
                   "editOwnAccount"
            FROM employee_permissions
            ORDER BY "employeeID"
            "#,
            &[],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    for row in permission_rows {
        store.set_employee_permissions(
            row.get::<_, i32>(0) as u32,
            EmployeePermissions {
                create_groups: row.get(1),
                approve_vacations: row.get(2),
                approve_own_vacations: row.get(3),
                request_vacation: row.get(4),
                edit_vacations: row.get(5),
                delete_vacations: row.get(6),
                edit_own_account: row.get(7),
            },
        );
    }

    let member_rows = client
        .query(
            r#"
            SELECT "groupID", "employeeID"
            FROM group_members
            ORDER BY "groupID", "employeeID"
            "#,
            &[],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    for row in member_rows {
        store.restore_member(row.get::<_, i32>(0) as u32, row.get::<_, i32>(1) as u32);
    }

    let member_permission_rows = client
        .query(
            r#"
            SELECT "groupID",
                   "employeeID",
                   "canApproveVacations",
                   "canApproveOwnVacations",
                   "canEditGroup",
                   "canAddMembers",
                   "canRemoveMembers",
                   "canDeleteGroup"
            FROM group_member_permissions
            ORDER BY "groupID", "employeeID"
            "#,
            &[],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    for row in member_permission_rows {
        store.restore_member_permissions(
            row.get::<_, i32>(0) as u32,
            row.get::<_, i32>(1) as u32,
            GroupMemberPermissions {
                can_approve_vacations: row.get(2),
                can_approve_own_vacations: row.get(3),
                can_edit_group: row.get(4),
                can_add_members: row.get(5),
                can_remove_members: row.get(6),
                can_delete_group: row.get(7),
            },
        );
    }

    Ok(store)
}

pub async fn save_employee_permissions(
    config: &VactionAppConfigs,
    employee_id: u32,
    permissions: &EmployeePermissions,
) -> Result<(), VacationAppDbError> {
    let db = connect_database(config).await?;
    db.client()
        .execute(
            r#"
            INSERT INTO employee_permissions (
                "employeeID",
                "createGroups",
                "approveVacations",
                "approveOwnVacations",
                "requestVacation",
                "editVacations",
                "deleteVacations",
                "editOwnAccount"
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT ("employeeID") DO UPDATE SET
                "createGroups" = EXCLUDED."createGroups",
                "approveVacations" = EXCLUDED."approveVacations",
                "approveOwnVacations" = EXCLUDED."approveOwnVacations",
                "requestVacation" = EXCLUDED."requestVacation",
                "editVacations" = EXCLUDED."editVacations",
                "deleteVacations" = EXCLUDED."deleteVacations",
                "editOwnAccount" = EXCLUDED."editOwnAccount"
            "#,
            &[
                &(employee_id as i32),
                &permissions.create_groups,
                &permissions.approve_vacations,
                &permissions.approve_own_vacations,
                &permissions.request_vacation,
                &permissions.edit_vacations,
                &permissions.delete_vacations,
                &permissions.edit_own_account,
            ],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    Ok(())
}

pub async fn save_group_member(
    config: &VactionAppConfigs,
    group_id: u32,
    employee_id: u32,
) -> Result<(), VacationAppDbError> {
    let db = connect_database(config).await?;
    db.client()
        .execute(
            r#"
            INSERT INTO group_members ("groupID", "employeeID")
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
            &[&(group_id as i32), &(employee_id as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    Ok(())
}

pub async fn delete_group_member(
    config: &VactionAppConfigs,
    group_id: u32,
    employee_id: u32,
) -> Result<(), VacationAppDbError> {
    let db = connect_database(config).await?;
    delete_group_member_data(db.client(), group_id, employee_id).await
}

pub async fn save_group_member_permissions(
    config: &VactionAppConfigs,
    group_id: u32,
    employee_id: u32,
    permissions: &GroupMemberPermissions,
) -> Result<(), VacationAppDbError> {
    let db = connect_database(config).await?;
    db.client()
        .execute(
            r#"
            INSERT INTO group_member_permissions (
                "groupID",
                "employeeID",
                "canApproveVacations",
                "canApproveOwnVacations",
                "canEditGroup",
                "canAddMembers",
                "canRemoveMembers",
                "canDeleteGroup"
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT ("groupID", "employeeID") DO UPDATE SET
                "canApproveVacations" = EXCLUDED."canApproveVacations",
                "canApproveOwnVacations" = EXCLUDED."canApproveOwnVacations",
                "canEditGroup" = EXCLUDED."canEditGroup",
                "canAddMembers" = EXCLUDED."canAddMembers",
                "canRemoveMembers" = EXCLUDED."canRemoveMembers",
                "canDeleteGroup" = EXCLUDED."canDeleteGroup"
            "#,
            &[
                &(group_id as i32),
                &(employee_id as i32),
                &permissions.can_approve_vacations,
                &permissions.can_approve_own_vacations,
                &permissions.can_edit_group,
                &permissions.can_add_members,
                &permissions.can_remove_members,
                &permissions.can_delete_group,
            ],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    Ok(())
}

pub async fn replace_employee_group_memberships(
    config: &VactionAppConfigs,
    employee_id: u32,
    group_ids: &[u32],
) -> Result<(), VacationAppDbError> {
    let db = connect_database(config).await?;
    let client = db.client();
    client
        .execute(
            r#"DELETE FROM group_member_permissions WHERE "employeeID" = $1"#,
            &[&(employee_id as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    client
        .execute(
            r#"DELETE FROM group_members WHERE "employeeID" = $1"#,
            &[&(employee_id as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    for group_id in group_ids {
        client
            .execute(
                r#"
                INSERT INTO group_members ("groupID", "employeeID")
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
                &[&(*group_id as i32), &(employee_id as i32)],
            )
            .await
            .map_err(|_| VacationAppDbError::QueryFailed)?;
    }
    Ok(())
}

async fn delete_group_member_data(
    client: &tokio_postgres::Client,
    group_id: u32,
    employee_id: u32,
) -> Result<(), VacationAppDbError> {
    client
        .execute(
            r#"
            DELETE FROM group_member_permissions
            WHERE "groupID" = $1 AND "employeeID" = $2
            "#,
            &[&(group_id as i32), &(employee_id as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    client
        .execute(
            r#"
            DELETE FROM group_members
            WHERE "groupID" = $1 AND "employeeID" = $2
            "#,
            &[&(group_id as i32), &(employee_id as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    Ok(())
}

async fn delete_group_membership_data(
    client: &tokio_postgres::Client,
    group_id: u32,
) -> Result<(), VacationAppDbError> {
    client
        .execute(
            r#"DELETE FROM group_member_permissions WHERE "groupID" = $1"#,
            &[&(group_id as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    client
        .execute(
            r#"DELETE FROM group_members WHERE "groupID" = $1"#,
            &[&(group_id as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    Ok(())
}

async fn delete_employee_group_data(
    client: &tokio_postgres::Client,
    employee_id: u32,
) -> Result<(), VacationAppDbError> {
    client
        .execute(
            r#"DELETE FROM employee_permissions WHERE "employeeID" = $1"#,
            &[&(employee_id as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    client
        .execute(
            r#"DELETE FROM group_member_permissions WHERE "employeeID" = $1"#,
            &[&(employee_id as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    client
        .execute(
            r#"DELETE FROM group_members WHERE "employeeID" = $1"#,
            &[&(employee_id as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;
    Ok(())
}

async fn connect_database(config: &VactionAppConfigs) -> Result<PostgresDatabase, VacationAppDbError> {
    if config.database_url().is_empty() {
        return Err(VacationAppDbError::Config(
            "database.database_url must be set in VactionAppConfigs".into(),
        ));
    }

    let server = PostgresConnector::connect_url(config.database_url())
        .await
        .map_err(map_postgres_error)?;

    server
        .open(PostgresServer::query_db_name(
            DBQueryOps::Equal,
            config.db_name(),
        ))
        .await
        .map_err(map_postgres_error)
}

async fn load_vacations(
    client: &tokio_postgres::Client,
    employee_id: Option<EmployeeID>,
) -> Result<Vec<Vacation>, VacationAppDbError> {
    let rows = if let Some(employee_id) = employee_id {
        client
            .query(
                r#"
                SELECT "vacationID", "employeeID",
                       "startYear", "startMonth", "startDay",
                       "finalYear", "finalMonth", "finalDay",
                       "status", "approvedBy", "coveredBy", "deniedBy",
                       "details", "denialReason"
                FROM vacations
                WHERE "employeeID" = $1
                ORDER BY "vacationID"
                "#,
                &[&(employee_id.id() as i32)],
            )
            .await
    } else {
        client
            .query(
                r#"
                SELECT "vacationID", "employeeID",
                       "startYear", "startMonth", "startDay",
                       "finalYear", "finalMonth", "finalDay",
                       "status", "approvedBy", "coveredBy", "deniedBy",
                       "details", "denialReason"
                FROM vacations
                ORDER BY "vacationID"
                "#,
                &[],
            )
            .await
    }
    .map_err(|_| VacationAppDbError::QueryFailed)?;

    rows.iter().map(row_to_vacation).collect()
}

async fn update_vacation_status(
    config: &VactionAppConfigs,
    vacation_id: VacationID,
    status: &str,
    approved_by: Option<EmployeeID>,
    covered_by: Option<EmployeeID>,
    denied_by: Option<EmployeeID>,
) -> Result<Vacation, VacationAppDbError> {
    let db = connect_database(config).await?;
    let row = db
        .client()
        .query_one(
            r#"
            UPDATE vacations
            SET "status" = $2,
                "approvedBy" = $3,
                "coveredBy" = $4,
                "deniedBy" = $5
            WHERE "vacationID" = $1
            RETURNING "vacationID", "employeeID",
                      "startYear", "startMonth", "startDay",
                      "finalYear", "finalMonth", "finalDay",
                      "status", "approvedBy", "coveredBy", "deniedBy",
                      "details", "denialReason"
            "#,
            &[
                &(vacation_id.id() as i32),
                &status,
                &approved_by.map(|id| id.id() as i32),
                &covered_by.map(|id| id.id() as i32),
                &denied_by.map(|id| id.id() as i32),
            ],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    row_to_vacation(&row)
}

async fn load_organization_employee_ids(
    client: &tokio_postgres::Client,
    organization_id: OrganizationID,
) -> Result<Vec<u32>, VacationAppDbError> {
    let rows = client
        .query(
            r#"
            SELECT "employeeID"
            FROM organization_employees
            WHERE "organizationID" = $1
            ORDER BY "employeeID"
            "#,
            &[&(organization_id.id() as i32)],
        )
        .await
        .map_err(|_| VacationAppDbError::QueryFailed)?;

    Ok(rows
        .iter()
        .map(|row| row.get::<_, i32>(0) as u32)
        .collect())
}

fn row_to_employee(row: &tokio_postgres::Row) -> Result<EmployeeRecord, VacationAppDbError> {
    Ok(EmployeeRecord {
        employee_id: EmployeeID::new(row.get::<_, i32>(0) as u32),
        username: row.get(1),
        email: row.get(2),
        avatar_url: row.get(3),
    })
}

fn row_to_user(row: &tokio_postgres::Row) -> Result<UserRecord, VacationAppDbError> {
    Ok(UserRecord {
        user_id: UserID::new(row.get::<_, i32>(0) as u32),
        email: row.get(1),
        organization_id: row.get::<_, i32>(2) as u32,
    })
}

fn row_to_group(row: &tokio_postgres::Row) -> Result<GroupRecord, VacationAppDbError> {
    Ok(GroupRecord {
        group_id: row.get::<_, i32>(0) as u32,
        organization_id: row.get::<_, i32>(1) as u32,
        name: row.get(2),
    })
}

fn row_to_organization(
    row: &tokio_postgres::Row,
    employee_ids: &[u32],
) -> Result<Organization, VacationAppDbError> {
    let organization_id = OrganizationID::new(row.get::<_, i32>(0) as u32);
    let owner_id = row.get::<_, i32>(1) as u32;
    let name: String = row.get(2);

    Ok(Organization::new(owner_id, name, organization_id).with_employee_ids(employee_ids.to_vec()))
}

fn row_to_vacation(row: &tokio_postgres::Row) -> Result<Vacation, VacationAppDbError> {
    let vacation_id: i32 = row.get(0);
    let employee_id: i32 = row.get(1);
    let start_year: i16 = row.get(2);
    let start_month: i16 = row.get(3);
    let start_day: i16 = row.get(4);
    let final_year: i16 = row.get(5);
    let final_month: i16 = row.get(6);
    let final_day: i16 = row.get(7);
    let status: String = row.get(8);
    let approved_by: Option<i32> = row.get(9);
    let covered_by: Option<i32> = row.get(10);
    let denied_by: Option<i32> = row.get(11);
    let details: String = row.try_get(12).unwrap_or_default();
    let denial_reason: String = row.try_get(13).unwrap_or_default();

    let status = parse_vacation_status(&status, approved_by, covered_by, denied_by)?;

    Ok(Vacation::new(
        Date::new(start_year as u8, start_month as u8, start_day as u8),
        Date::new(final_year as u8, final_month as u8, final_day as u8),
        EmployeeID::new(employee_id as u32),
        VacationID::new(vacation_id as u32),
        status,
    )
    .with_details(details)
    .with_denial_reason(denial_reason))
}

fn parse_vacation_status(
    status: &str,
    approved_by: Option<i32>,
    covered_by: Option<i32>,
    denied_by: Option<i32>,
) -> Result<VacationStatus, VacationAppDbError> {
    match status {
        "awaiting" => Ok(VacationStatus::AwaitingConfirmation),
        "approved" => {
            let approved_by = approved_by.ok_or(VacationAppDbError::InvalidData)?;
            let covered_by = covered_by.ok_or(VacationAppDbError::InvalidData)?;
            Ok(VacationStatus::Approved {
                approvedBy: EmployeeID::new(approved_by as u32),
                coveredBy: EmployeeID::new(covered_by as u32),
            })
        }
        "denied" => {
            let denied_by = denied_by.ok_or(VacationAppDbError::InvalidData)?;
            Ok(VacationStatus::Denied {
                deniedBy: EmployeeID::new(denied_by as u32),
            })
        }
        "cancellation_requested" => {
            let requested_by = denied_by.ok_or(VacationAppDbError::InvalidData)?;
            let approved_by = approved_by.ok_or(VacationAppDbError::InvalidData)?;
            let covered_by = covered_by.ok_or(VacationAppDbError::InvalidData)?;
            Ok(VacationStatus::CancellationRequested {
                requestedBy: EmployeeID::new(requested_by as u32),
                approvedBy: EmployeeID::new(approved_by as u32),
                coveredBy: EmployeeID::new(covered_by as u32),
            })
        }
        "cancelled" => {
            let cancelled_by = denied_by.ok_or(VacationAppDbError::InvalidData)?;
            Ok(VacationStatus::Cancelled {
                cancelledBy: EmployeeID::new(cancelled_by as u32),
            })
        }
        _ => Err(VacationAppDbError::InvalidData),
    }
}

fn map_postgres_error(error: PostgresError) -> VacationAppDbError {
    match error {
        PostgresError::ConnectionFailed => VacationAppDbError::ConnectionFailed,
        PostgresError::DatabaseNotFound
        | PostgresError::QueryFailed
        | PostgresError::CreateDatabaseDenied => VacationAppDbError::QueryFailed,
        PostgresError::InvalidConfig | PostgresError::InvalidConnectionUrl => {
            VacationAppDbError::Config("invalid postgres configuration".into())
        }
        PostgresError::UnsupportedQueryOp => VacationAppDbError::QueryFailed,
    }
}

fn map_auth_error(error: crate::auth::AuthError) -> VacationAppDbError {
    match error {
        crate::auth::AuthError::DatabaseUnavailable => VacationAppDbError::ConnectionFailed,
        crate::auth::AuthError::SessionNotFound
        | crate::auth::AuthError::UserNotFound
        | crate::auth::AuthError::EmployeeNotFound
        | crate::auth::AuthError::InvalidCredentials => VacationAppDbError::NotFound,
        crate::auth::AuthError::NotImplemented => VacationAppDbError::QueryFailed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{load_config_from, VactionAppConfigs};
    use std::path::PathBuf;
    use tokio::sync::Mutex;

    static DB_LOCK: Mutex<()> = Mutex::const_new(());

    fn workspace_config_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("server_configs.toml")
    }

    fn load_test_config() -> Option<VactionAppConfigs> {
        match load_config_from(workspace_config_path()) {
            Ok(config) => Some(config),
            Err(err) => {
                eprintln!("skipping vacation_app_db tests: could not load config ({err:?})");
                None
            }
        }
    }

    async fn with_db_test<F, Fut>(test: F)
    where
        F: FnOnce(VactionAppConfigs) -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let _guard = DB_LOCK.lock().await;
        let Some(config) = load_test_config() else {
            return;
        };

        if setup_db(&config).await.is_err() {
            eprintln!("skipping vacation_app_db tests: could not connect to database");
            return;
        }

        test(config).await;
    }

    #[tokio::test]
    async fn setup_db_is_idempotent() {
        with_db_test(|config| async move {
            setup_db(&config).await.expect("first setup");
            setup_db(&config).await.expect("second setup");
        })
        .await;
    }

    #[tokio::test]
    async fn vacation_request_and_lookup_flow() {
        with_db_test(|config| async move {
            let created = request_vacation(
                &config,
                EmployeeID::new(10),
                Date::new(26, 8, 1),
                Date::new(26, 8, 15),
            )
            .await
            .expect("create vacation");

            assert_eq!(created.employee_id().id(), 10);
            assert!(matches!(
                created.status(),
                VacationStatus::AwaitingConfirmation
            ));

            let loaded = get_vacation(&config, created.vacation_id())
                .await
                .expect("load vacation");
            assert_eq!(loaded.vacation_id().id(), created.vacation_id().id());

            let employee_vacations =
                get_vacations_for_employee(&config, EmployeeID::new(10))
                    .await
                    .expect("employee vacations");
            assert!(!employee_vacations.is_empty());
        })
        .await;
    }

    #[tokio::test]
    async fn employee_and_organization_crud_flow() {
        with_db_test(|config| async move {
            let employee = create_employee(&config, "alice", "alice@test.com")
                .await
                .expect("create employee");

            let organization = create_organization(&config, 1, "Acme Corp")
                .await
                .expect("create organization");

            add_employee_to_organization(
                &config,
                organization.organization_id(),
                employee.employee_id(),
            )
            .await
            .expect("link employee");

            let loaded = get_organization(&config, organization.organization_id())
                .await
                .expect("get organization");
            assert_eq!(loaded.name(), "Acme Corp");
            assert!(loaded.employee_ids().contains(&employee.employee_id().id()));

            let updated = update_employee(
                &config,
                employee.employee_id(),
                "alice2",
                "alice2@test.com",
            )
            .await
            .expect("update employee");
            assert_eq!(updated.username(), "alice2");

            delete_employee(&config, employee.employee_id())
                .await
                .expect("delete employee");
        })
        .await;
    }
}
