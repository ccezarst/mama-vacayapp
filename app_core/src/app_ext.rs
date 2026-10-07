use serde::{Deserialize, Serialize};

use crate::api::{AppError, VacationAppApi};
use crate::employees::{EmployeeID, Vacation, VacationID};
use crate::organizations::{Organization, OrganizationID};
use crate::users::UserID;
use crate::vacation_app_db::EmployeeRecord;
use crate::Date;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmployeePermissions {
    /// Global: may create new groups in the organization.
    pub create_groups: bool,
    pub approve_vacations: bool,
    /// Allows an employee to approve or deny their own vacation requests.
    pub approve_own_vacations: bool,
    pub request_vacation: bool,
    /// Allows editing any vacation's dates/details at any time.
    pub edit_vacations: bool,
    /// Allows deleting vacations.
    pub delete_vacations: bool,
    /// Allows the employee to edit their own username, email, and password.
    pub edit_own_account: bool,
}

impl EmployeePermissions {
    pub fn default_employee() -> Self {
        Self {
            create_groups: false,
            approve_vacations: false,
            approve_own_vacations: false,
            request_vacation: true,
            edit_vacations: false,
            delete_vacations: false,
            edit_own_account: true,
        }
    }

    pub fn can_administer_groups(&self) -> bool {
        self.create_groups
    }
}

/// Permissions scoped to one membership inside a group.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GroupMemberPermissions {
    pub can_approve_vacations: bool,
    /// Allows approving/denying this group's vote on the member's own vacation.
    #[serde(default)]
    pub can_approve_own_vacations: bool,
    pub can_edit_group: bool,
    pub can_add_members: bool,
    pub can_remove_members: bool,
    pub can_delete_group: bool,
}

impl GroupMemberPermissions {
    pub fn group_admin() -> Self {
        Self {
            can_approve_vacations: true,
            can_approve_own_vacations: false,
            can_edit_group: true,
            can_add_members: true,
            can_remove_members: true,
            can_delete_group: true,
        }
    }
}

/// Per-group vote on a vacation that requires multi-group approval.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VacationGroupApproval {
    pub group_id: u32,
    pub group_name: String,
    pub status: String,
    pub decided_by: Option<u32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MockPaymentDetails {
    pub billing_name: String,
    pub card_last4: String,
    pub plan: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GroupDetail {
    pub id: u32,
    pub organization_id: u32,
    pub name: String,
    pub member_ids: Vec<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmployeeProfile {
    pub id: u32,
    pub username: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub organization_id: Option<u32>,
    pub group_ids: Vec<u32>,
    pub permissions: EmployeePermissions,
}

#[derive(Debug, Clone, Serialize)]
pub struct VacationSummary {
    pub id: u32,
    pub start: String,
    pub end: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VacationStats {
    pub total: usize,
    pub awaiting: usize,
    pub approved: usize,
    pub denied: usize,
    pub requests: Vec<VacationSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdminEmployeeDetail {
    pub profile: EmployeeProfile,
    pub vacations: VacationStats,
}

pub fn vacation_summary(vacation: &Vacation) -> VacationSummary {
    VacationSummary {
        id: vacation.vacation_id().id(),
        start: format!(
            "{:02}-{:02}-{:02}",
            vacation.start_date().year(),
            vacation.start_date().month(),
            vacation.start_date().day()
        ),
        end: format!(
            "{:02}-{:02}-{:02}",
            vacation.final_date().year(),
            vacation.final_date().month(),
            vacation.final_date().day()
        ),
        status: format!("{:?}", vacation.status()),
    }
}

pub fn vacation_stats_for(vacations: &[Vacation]) -> VacationStats {
    let requests: Vec<VacationSummary> = vacations.iter().map(vacation_summary).collect();
    let awaiting = requests
        .iter()
        .filter(|v| v.status.contains("Awaiting") || v.status.contains("CancellationRequested"))
        .count();
    let approved = requests
        .iter()
        .filter(|v| v.status.contains("Approved") && !v.status.contains("Cancellation"))
        .count();
    let denied = requests
        .iter()
        .filter(|v| v.status.contains("Denied") || v.status.contains("Cancelled"))
        .count();
    VacationStats {
        total: requests.len(),
        awaiting,
        approved,
        denied,
        requests,
    }
}

/// Extended operations used by the prototype demo (permissions, groups, billing).
pub trait VacationAppExt: VacationAppApi {
    async fn register_employee_login(
        &self,
        email: &str,
        employee_id: EmployeeID,
        password: &str,
    ) -> Result<(), AppError>;

    async fn get_user_payment(&self, user_id: UserID) -> Result<MockPaymentDetails, AppError>;
    async fn set_user_payment(
        &self,
        user_id: UserID,
        payment: MockPaymentDetails,
    ) -> Result<MockPaymentDetails, AppError>;

    async fn get_employee_profile(&self, employee_id: EmployeeID)
        -> Result<EmployeeProfile, AppError>;

    async fn update_own_account(
        &self,
        actor_id: EmployeeID,
        username: &str,
        email: &str,
        password: Option<&str>,
        password_change_email: Option<&str>,
    ) -> Result<EmployeeProfile, AppError>;

    async fn update_own_avatar(
        &self,
        actor_id: EmployeeID,
        avatar_url: Option<&str>,
    ) -> Result<EmployeeProfile, AppError>;

    async fn list_employees_visible_to(
        &self,
        actor_id: EmployeeID,
    ) -> Result<Vec<EmployeeRecord>, AppError>;

    async fn list_vacations_visible_to(
        &self,
        actor_id: EmployeeID,
    ) -> Result<Vec<Vacation>, AppError>;

    async fn get_employee_permissions(
        &self,
        actor_id: EmployeeID,
        target_id: EmployeeID,
    ) -> Result<EmployeePermissions, AppError>;

    async fn set_employee_permissions(
        &self,
        actor_id: EmployeeID,
        target_id: EmployeeID,
        permissions: EmployeePermissions,
    ) -> Result<EmployeePermissions, AppError>;

    async fn list_group_details(&self, organization_id: u32) -> Result<Vec<GroupDetail>, AppError>;

    async fn create_group_as_employee(
        &self,
        actor_id: EmployeeID,
        organization_id: u32,
        name: &str,
    ) -> Result<GroupDetail, AppError>;

    async fn update_group_as_employee(
        &self,
        actor_id: EmployeeID,
        group_id: u32,
        name: &str,
    ) -> Result<GroupDetail, AppError>;

    async fn delete_group_as_employee(
        &self,
        actor_id: EmployeeID,
        group_id: u32,
    ) -> Result<(), AppError>;

    async fn add_group_member(
        &self,
        actor_id: EmployeeID,
        group_id: u32,
        member_id: EmployeeID,
    ) -> Result<GroupDetail, AppError>;

    async fn remove_group_member(
        &self,
        actor_id: EmployeeID,
        group_id: u32,
        member_id: EmployeeID,
    ) -> Result<GroupDetail, AppError>;

    async fn get_group_member_permissions(
        &self,
        actor_id: EmployeeID,
        group_id: u32,
        member_id: EmployeeID,
    ) -> Result<GroupMemberPermissions, AppError>;

    async fn set_group_member_permissions(
        &self,
        actor_id: EmployeeID,
        group_id: u32,
        member_id: EmployeeID,
        permissions: GroupMemberPermissions,
    ) -> Result<GroupMemberPermissions, AppError>;

    async fn admin_get_employee_detail(
        &self,
        user_id: UserID,
        employee_id: EmployeeID,
    ) -> Result<AdminEmployeeDetail, AppError>;

    async fn admin_update_employee(
        &self,
        user_id: UserID,
        employee_id: EmployeeID,
        username: &str,
        email: &str,
        password: Option<&str>,
    ) -> Result<EmployeeProfile, AppError>;

    async fn admin_set_employee_permissions(
        &self,
        user_id: UserID,
        employee_id: EmployeeID,
        permissions: EmployeePermissions,
    ) -> Result<EmployeePermissions, AppError>;

    async fn admin_set_employee_groups(
        &self,
        user_id: UserID,
        employee_id: EmployeeID,
        group_ids: Vec<u32>,
    ) -> Result<EmployeeProfile, AppError>;

    async fn admin_delete_employee(
        &self,
        user_id: UserID,
        employee_id: EmployeeID,
    ) -> Result<(), AppError>;

    async fn admin_update_organization(
        &self,
        user_id: UserID,
        organization_id: OrganizationID,
        name: &str,
    ) -> Result<Organization, AppError>;

    async fn admin_create_group(
        &self,
        user_id: UserID,
        organization_id: u32,
        name: &str,
    ) -> Result<GroupDetail, AppError>;

    async fn admin_update_group(
        &self,
        user_id: UserID,
        group_id: u32,
        name: &str,
    ) -> Result<GroupDetail, AppError>;

    async fn admin_delete_group(&self, user_id: UserID, group_id: u32) -> Result<(), AppError>;

    async fn admin_add_group_member(
        &self,
        user_id: UserID,
        group_id: u32,
        member_id: EmployeeID,
    ) -> Result<GroupDetail, AppError>;

    async fn admin_remove_group_member(
        &self,
        user_id: UserID,
        group_id: u32,
        member_id: EmployeeID,
    ) -> Result<GroupDetail, AppError>;

    async fn admin_get_group_member_permissions(
        &self,
        user_id: UserID,
        group_id: u32,
        member_id: EmployeeID,
    ) -> Result<GroupMemberPermissions, AppError>;

    async fn admin_set_group_member_permissions(
        &self,
        user_id: UserID,
        group_id: u32,
        member_id: EmployeeID,
        permissions: GroupMemberPermissions,
    ) -> Result<GroupMemberPermissions, AppError>;

    async fn request_vacation_with_permissions(
        &self,
        employee_id: EmployeeID,
        start_date: Date,
        final_date: Date,
        details: &str,
    ) -> Result<Vacation, AppError>;

    async fn approve_vacation_with_permissions(
        &self,
        actor_id: EmployeeID,
        vacation_id: VacationID,
        covered_by: EmployeeID,
        group_id: Option<u32>,
    ) -> Result<Vacation, AppError>;

    async fn deny_vacation_with_permissions(
        &self,
        actor_id: EmployeeID,
        vacation_id: VacationID,
        group_id: Option<u32>,
        reason: Option<&str>,
    ) -> Result<Vacation, AppError>;

    async fn request_vacation_cancellation_with_permissions(
        &self,
        actor_id: EmployeeID,
        vacation_id: VacationID,
    ) -> Result<Vacation, AppError>;

    async fn approve_vacation_cancellation_with_permissions(
        &self,
        actor_id: EmployeeID,
        vacation_id: VacationID,
        group_id: Option<u32>,
    ) -> Result<Vacation, AppError>;

    async fn deny_vacation_cancellation_with_permissions(
        &self,
        actor_id: EmployeeID,
        vacation_id: VacationID,
        group_id: Option<u32>,
    ) -> Result<Vacation, AppError>;

    async fn vacation_group_approvals(
        &self,
        vacation_id: VacationID,
    ) -> Result<Vec<VacationGroupApproval>, AppError>;

    async fn vacation_group_approvals_for_many(
        &self,
        vacation_ids: &[u32],
    ) -> Result<Vec<(u32, Vec<VacationGroupApproval>)>, AppError>;

    async fn update_vacation_with_permissions(
        &self,
        actor_id: EmployeeID,
        vacation_id: VacationID,
        start_date: Date,
        final_date: Date,
        details: &str,
    ) -> Result<Vacation, AppError>;

    async fn delete_vacation_with_permissions(
        &self,
        actor_id: EmployeeID,
        vacation_id: VacationID,
    ) -> Result<(), AppError>;

    async fn admin_list_org_vacations(
        &self,
        user_id: UserID,
    ) -> Result<Vec<Vacation>, AppError>;

    async fn admin_update_vacation(
        &self,
        user_id: UserID,
        vacation_id: VacationID,
        start_date: Date,
        final_date: Date,
        details: &str,
    ) -> Result<Vacation, AppError>;

    async fn admin_approve_vacation(
        &self,
        user_id: UserID,
        vacation_id: VacationID,
    ) -> Result<Vacation, AppError>;

    async fn admin_deny_vacation(
        &self,
        user_id: UserID,
        vacation_id: VacationID,
        reason: Option<&str>,
    ) -> Result<Vacation, AppError>;

    async fn admin_delete_vacation(
        &self,
        user_id: UserID,
        vacation_id: VacationID,
    ) -> Result<(), AppError>;

    async fn admin_list_vacation_events(
        &self,
        user_id: UserID,
        vacation_id: VacationID,
    ) -> Result<Vec<crate::vacation_app_db::EventRecord>, AppError>;

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
        self.get_employee_profile(employee.employee_id()).await
    }
}
