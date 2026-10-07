//! Group membership and permission rules shared by the prototype and real backends.
//!
//! Group shells (id / organization / name) live in the database. Membership lists and
//! permission maps are owned by [`GroupMembershipStore`], so authorization and member
//! mutations can be unit-tested without Postgres and reused by any storage adapter.

use std::collections::HashMap;

use crate::api::AppError;
use crate::app_ext::{
    EmployeePermissions, EmployeeProfile, GroupDetail, GroupMemberPermissions,
};
use crate::vacation_app_db::GroupRecord;

/// In-memory (or adapter-backed) overlay for group membership and permissions.
#[derive(Clone, Debug, Default)]
pub struct GroupMembershipStore {
    employee_permissions: HashMap<u32, EmployeePermissions>,
    group_members: HashMap<u32, Vec<u32>>,
    /// Keyed as `(group_id, employee_id)`.
    group_member_permissions: HashMap<(u32, u32), GroupMemberPermissions>,
}

impl GroupMembershipStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn employee_permissions(&self, employee_id: u32) -> EmployeePermissions {
        self.employee_permissions
            .get(&employee_id)
            .cloned()
            .unwrap_or_else(EmployeePermissions::default_employee)
    }

    pub fn set_employee_permissions(
        &mut self,
        employee_id: u32,
        permissions: EmployeePermissions,
    ) {
        self.employee_permissions.insert(employee_id, permissions);
    }

    pub fn ensure_default_employee_permissions(&mut self, employee_id: u32) {
        self.employee_permissions
            .entry(employee_id)
            .or_insert_with(EmployeePermissions::default_employee);
    }

    pub fn remove_employee(&mut self, employee_id: u32) {
        self.employee_permissions.remove(&employee_id);
        for members in self.group_members.values_mut() {
            members.retain(|id| *id != employee_id);
        }
        self.group_member_permissions
            .retain(|(_, eid), _| *eid != employee_id);
    }

    pub fn group_ids_for_employee(&self, employee_id: u32) -> Vec<u32> {
        self.group_members
            .iter()
            .filter_map(|(group_id, members)| {
                if members.contains(&employee_id) {
                    Some(*group_id)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Employees visible on the actor's schedule: self plus anyone sharing a group.
    pub fn visible_colleague_ids(&self, actor_id: u32) -> Vec<u32> {
        let mut ids = vec![actor_id];
        for group_id in self.group_ids_for_employee(actor_id) {
            for member_id in self.members(group_id) {
                if !ids.contains(&member_id) {
                    ids.push(member_id);
                }
            }
        }
        ids
    }

    pub fn shares_group_with(&self, actor_id: u32, other_id: u32) -> bool {
        if actor_id == other_id {
            return true;
        }
        self.group_ids_for_employee(actor_id)
            .into_iter()
            .any(|group_id| self.is_member(group_id, other_id))
    }

    pub fn member_permissions(
        &self,
        group_id: u32,
        employee_id: u32,
    ) -> GroupMemberPermissions {
        self.group_member_permissions
            .get(&(group_id, employee_id))
            .cloned()
            .unwrap_or_default()
    }

    pub fn is_member(&self, group_id: u32, employee_id: u32) -> bool {
        self.group_members
            .get(&group_id)
            .map(|members| members.contains(&employee_id))
            .unwrap_or(false)
    }

    pub fn members(&self, group_id: u32) -> Vec<u32> {
        self.group_members
            .get(&group_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn ensure_group(&mut self, group_id: u32) {
        self.group_members.entry(group_id).or_default();
    }

    /// Restore membership from persistent storage (no authorization checks).
    pub fn restore_member(&mut self, group_id: u32, employee_id: u32) {
        self.ensure_group(group_id);
        let members = self.group_members.entry(group_id).or_default();
        if !members.contains(&employee_id) {
            members.push(employee_id);
        }
    }

    /// Restore group-scoped permissions from persistent storage (no authorization checks).
    pub fn restore_member_permissions(
        &mut self,
        group_id: u32,
        employee_id: u32,
        permissions: GroupMemberPermissions,
    ) {
        self.group_member_permissions
            .insert((group_id, employee_id), permissions);
    }

    pub fn clear_group(&mut self, group_id: u32) {
        self.group_members.remove(&group_id);
        self.group_member_permissions
            .retain(|(gid, _), _| *gid != group_id);
    }

    pub fn to_detail(&self, group: &GroupRecord) -> GroupDetail {
        GroupDetail {
            id: group.group_id(),
            organization_id: group.organization_id(),
            name: group.name().to_string(),
            member_ids: self.members(group.group_id()),
        }
    }

    pub fn list_details(&self, groups: &[GroupRecord]) -> Vec<GroupDetail> {
        groups.iter().map(|group| self.to_detail(group)).collect()
    }

    /// Replace an employee's group memberships with `group_ids` (admin path).
    pub fn replace_employee_groups(&mut self, employee_id: u32, group_ids: &[u32]) {
        for members in self.group_members.values_mut() {
            members.retain(|id| *id != employee_id);
        }
        for group_id in group_ids {
            let members = self.group_members.entry(*group_id).or_default();
            if !members.contains(&employee_id) {
                members.push(employee_id);
            }
        }
    }

    /// Adds the employee to the group (if needed) and grants full group-admin rights.
    pub fn grant_group_admin(&mut self, group_id: u32, employee_id: u32) {
        self.ensure_group(group_id);
        let members = self.group_members.entry(group_id).or_default();
        if !members.contains(&employee_id) {
            members.push(employee_id);
        }
        self.group_member_permissions
            .insert((group_id, employee_id), GroupMemberPermissions::group_admin());
    }

    pub fn set_employee_permissions_as_actor(
        &mut self,
        actor: &EmployeeProfile,
        target_id: u32,
        permissions: EmployeePermissions,
    ) -> Result<EmployeePermissions, AppError> {
        if !actor.permissions.can_administer_groups() {
            return Err(AppError::Forbidden);
        }
        self.set_employee_permissions(target_id, permissions.clone());
        Ok(permissions)
    }

    pub fn authorize_create_group(
        actor: &EmployeeProfile,
        organization_id: u32,
    ) -> Result<(), AppError> {
        if !actor.permissions.create_groups {
            return Err(AppError::Forbidden);
        }
        if actor.organization_id != Some(organization_id) {
            return Err(AppError::Forbidden);
        }
        Ok(())
    }

    pub fn authorize_update_group(
        &self,
        actor: &EmployeeProfile,
        group: &GroupRecord,
    ) -> Result<(), AppError> {
        let member_perms = self.member_permissions(group.group_id(), actor.id);
        if !member_perms.can_edit_group {
            return Err(AppError::Forbidden);
        }
        ensure_same_organization(actor, group.organization_id())
    }

    pub fn authorize_delete_group(
        &self,
        actor: &EmployeeProfile,
        group: &GroupRecord,
    ) -> Result<(), AppError> {
        let member_perms = self.member_permissions(group.group_id(), actor.id);
        if !member_perms.can_delete_group {
            return Err(AppError::Forbidden);
        }
        ensure_same_organization(actor, group.organization_id())
    }

    pub fn authorize_add_member(
        &self,
        actor: &EmployeeProfile,
        group: &GroupRecord,
    ) -> Result<(), AppError> {
        let member_perms = self.member_permissions(group.group_id(), actor.id);
        if !member_perms.can_add_members {
            return Err(AppError::Forbidden);
        }
        ensure_same_organization(actor, group.organization_id())
    }

    pub fn authorize_remove_member(
        &self,
        actor: &EmployeeProfile,
        group: &GroupRecord,
    ) -> Result<(), AppError> {
        let member_perms = self.member_permissions(group.group_id(), actor.id);
        if !member_perms.can_remove_members {
            return Err(AppError::Forbidden);
        }
        ensure_same_organization(actor, group.organization_id())
    }

    pub fn add_member(
        &mut self,
        actor: &EmployeeProfile,
        group: &GroupRecord,
        member_id: u32,
    ) -> Result<GroupDetail, AppError> {
        self.authorize_add_member(actor, group)?;
        let members = self.group_members.entry(group.group_id()).or_default();
        if !members.contains(&member_id) {
            members.push(member_id);
        }
        Ok(self.to_detail(group))
    }

    pub fn remove_member(
        &mut self,
        actor: &EmployeeProfile,
        group: &GroupRecord,
        member_id: u32,
    ) -> Result<GroupDetail, AppError> {
        self.authorize_remove_member(actor, group)?;
        let members = self
            .group_members
            .get_mut(&group.group_id())
            .ok_or(AppError::NotFound)?;
        let before = members.len();
        members.retain(|id| *id != member_id);
        if members.len() == before {
            return Err(AppError::NotFound);
        }
        self.group_member_permissions
            .remove(&(group.group_id(), member_id));
        Ok(self.to_detail(group))
    }

    /// Admin path: add a member without employee group-permission checks.
    pub fn admin_add_member(&mut self, group: &GroupRecord, member_id: u32) -> GroupDetail {
        let members = self.group_members.entry(group.group_id()).or_default();
        if !members.contains(&member_id) {
            members.push(member_id);
        }
        self.to_detail(group)
    }

    /// Admin path: remove a member without employee group-permission checks.
    pub fn admin_remove_member(
        &mut self,
        group: &GroupRecord,
        member_id: u32,
    ) -> Result<GroupDetail, AppError> {
        let members = self
            .group_members
            .get_mut(&group.group_id())
            .ok_or(AppError::NotFound)?;
        let before = members.len();
        members.retain(|id| *id != member_id);
        if members.len() == before {
            return Err(AppError::NotFound);
        }
        self.group_member_permissions
            .remove(&(group.group_id(), member_id));
        Ok(self.to_detail(group))
    }

    pub fn get_member_permissions_as_actor(
        &self,
        actor: &EmployeeProfile,
        group: &GroupRecord,
        member_id: u32,
    ) -> Result<GroupMemberPermissions, AppError> {
        let actor_member_perms = self.member_permissions(group.group_id(), actor.id);
        if !actor_member_perms.can_edit_group && actor.id != member_id {
            return Err(AppError::Forbidden);
        }
        ensure_same_organization(actor, group.organization_id())?;
        if !self.is_member(group.group_id(), member_id) {
            return Err(AppError::NotFound);
        }
        Ok(self.member_permissions(group.group_id(), member_id))
    }

    pub fn set_member_permissions_as_actor(
        &mut self,
        actor: &EmployeeProfile,
        group: &GroupRecord,
        member_id: u32,
        permissions: GroupMemberPermissions,
    ) -> Result<GroupMemberPermissions, AppError> {
        let actor_member_perms = self.member_permissions(group.group_id(), actor.id);
        if !actor_member_perms.can_edit_group {
            return Err(AppError::Forbidden);
        }
        ensure_same_organization(actor, group.organization_id())?;
        if !self.is_member(group.group_id(), member_id) {
            return Err(AppError::NotFound);
        }
        self.group_member_permissions
            .insert((group.group_id(), member_id), permissions.clone());
        Ok(permissions)
    }

    /// Admin path: read member permissions without employee group-permission checks.
    pub fn admin_get_member_permissions(
        &self,
        group: &GroupRecord,
        member_id: u32,
    ) -> Result<GroupMemberPermissions, AppError> {
        if !self.is_member(group.group_id(), member_id) {
            return Err(AppError::NotFound);
        }
        Ok(self.member_permissions(group.group_id(), member_id))
    }

    /// Admin path: set member permissions without employee group-permission checks.
    pub fn admin_set_member_permissions(
        &mut self,
        group: &GroupRecord,
        member_id: u32,
        permissions: GroupMemberPermissions,
    ) -> Result<GroupMemberPermissions, AppError> {
        if !self.is_member(group.group_id(), member_id) {
            return Err(AppError::NotFound);
        }
        self.group_member_permissions
            .insert((group.group_id(), member_id), permissions.clone());
        Ok(permissions)
    }

    /// Groups that must vote on this employee's vacations (all memberships).
    pub fn approval_group_ids_for_employee(&self, employee_id: u32) -> Vec<u32> {
        let mut ids = self.group_ids_for_employee(employee_id);
        ids.sort_unstable();
        ids
    }

    /// Employee IDs who can approve (or deny) a vacation for `employee_id`.
    ///
    /// Includes global org approvers, group approvers on required approval groups,
    /// and self-approvers when they have `approve_own_vacations`.
    pub fn approver_ids_for_vacation(
        &self,
        employee_id: u32,
        org_employee_ids: &[u32],
    ) -> Vec<u32> {
        let mut ids = Vec::new();

        for &id in org_employee_ids {
            let perms = self.employee_permissions(id);
            if id == employee_id {
                if perms.approve_own_vacations {
                    ids.push(id);
                }
            } else if perms.approve_vacations {
                ids.push(id);
            }
        }

        for group_id in self.approval_group_ids_for_employee(employee_id) {
            for member_id in self.members(group_id) {
                let member_perms = self.member_permissions(group_id, member_id);
                if member_id == employee_id {
                    if member_perms.can_approve_own_vacations {
                        ids.push(member_id);
                    }
                } else if member_perms.can_approve_vacations {
                    ids.push(member_id);
                }
            }
        }

        ids.sort_unstable();
        ids.dedup();
        ids
    }

    /// Whether anyone can approve this employee's vacation requests.
    ///
    /// Returns true if the requester can self-approve (global or group), any org
    /// coworker has global `approve_vacations`, or another member of a required
    /// approval group has `can_approve_vacations`. When false, requests should be
    /// auto-approved.
    pub fn anyone_can_approve_vacation_for(
        &self,
        employee_id: u32,
        org_employee_ids: &[u32],
    ) -> bool {
        if self.employee_permissions(employee_id).approve_own_vacations {
            return true;
        }
        if org_employee_ids.iter().any(|id| {
            *id != employee_id && self.employee_permissions(*id).approve_vacations
        }) {
            return true;
        }
        for group_id in self.approval_group_ids_for_employee(employee_id) {
            let member_perms = self.member_permissions(group_id, employee_id);
            if member_perms.can_approve_own_vacations {
                return true;
            }
            if self.members(group_id).into_iter().any(|member_id| {
                member_id != employee_id
                    && self
                        .member_permissions(group_id, member_id)
                        .can_approve_vacations
            }) {
                return true;
            }
        }
        false
    }

    /// Whether the actor may cast an approval/denial vote for `group_id` on a
    /// vacation owned by `vacation_owner_id`.
    ///
    /// Approving others requires global `approve_vacations` or group
    /// `can_approve_vacations`. Approving one's own requires global
    /// `approve_own_vacations` or group `can_approve_own_vacations`.
    pub fn actor_can_vote_for_group(
        &self,
        actor: &EmployeeProfile,
        group_id: u32,
        vacation_owner_id: u32,
    ) -> bool {
        let is_own = actor.id == vacation_owner_id;
        if is_own {
            if actor.permissions.approve_own_vacations {
                return true;
            }
            return self.is_member(group_id, actor.id)
                && self
                    .member_permissions(group_id, actor.id)
                    .can_approve_own_vacations;
        }
        if actor.permissions.approve_vacations {
            return true;
        }
        self.is_member(group_id, actor.id)
            && self
                .member_permissions(group_id, actor.id)
                .can_approve_vacations
    }

    /// Pending group IDs the actor is allowed to decide for this vacation owner.
    pub fn actor_votable_pending_groups(
        &self,
        actor: &EmployeeProfile,
        pending_group_ids: &[u32],
        vacation_owner_id: u32,
    ) -> Vec<u32> {
        pending_group_ids
            .iter()
            .copied()
            .filter(|group_id| self.actor_can_vote_for_group(actor, *group_id, vacation_owner_id))
            .collect()
    }

    fn is_group_creator_perms(perms: &GroupMemberPermissions) -> bool {
        perms.can_delete_group || (perms.can_edit_group && perms.can_add_members)
    }

    /// Vacation owner, or a group creator for any shared group with `employee_id`,
    /// may request cancellation of an approved vacation.
    pub fn actor_can_request_cancellation(
        &self,
        actor: &EmployeeProfile,
        employee_id: u32,
    ) -> bool {
        if actor.id == employee_id {
            return true;
        }
        self.group_ids_for_employee(employee_id).into_iter().any(|group_id| {
            self.is_member(group_id, actor.id)
                && Self::is_group_creator_perms(&self.member_permissions(group_id, actor.id))
        })
    }

    /// Global approve/delete, or group approve on a shared group, may accept/deny cancellation.
    pub fn actor_can_decide_cancellation(
        &self,
        actor: &EmployeeProfile,
        employee_id: u32,
    ) -> bool {
        if actor.permissions.delete_vacations || actor.permissions.approve_vacations {
            return true;
        }
        if actor.id == employee_id && actor.permissions.approve_own_vacations {
            return true;
        }
        self.group_ids_for_employee(employee_id)
            .into_iter()
            .any(|group_id| self.actor_can_vote_for_group(actor, group_id, employee_id))
    }
}

fn ensure_same_organization(actor: &EmployeeProfile, organization_id: u32) -> Result<(), AppError> {
    if actor.organization_id != Some(organization_id) {
        Err(AppError::Forbidden)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn actor_with(perms: EmployeePermissions, org: u32, id: u32) -> EmployeeProfile {
        EmployeeProfile {
            id,
            username: "alice".into(),
            email: "alice@test.com".into(),
            avatar_url: None,
            organization_id: Some(org),
            group_ids: vec![],
            permissions: perms,
        }
    }

    fn creator_actor() -> EmployeeProfile {
        actor_with(
            EmployeePermissions {
                create_groups: true,
                ..EmployeePermissions::default_employee()
            },
            1,
            10,
        )
    }

    fn viewer_actor() -> EmployeeProfile {
        actor_with(EmployeePermissions::default_employee(), 1, 11)
    }

    fn sample_group() -> GroupRecord {
        GroupRecord::new(5, 1, "Managers")
    }

    fn store_with_group_admin(actor_id: u32) -> (GroupMembershipStore, GroupRecord) {
        let mut store = GroupMembershipStore::new();
        let group = sample_group();
        store.grant_group_admin(group.group_id(), actor_id);
        (store, group)
    }

    #[test]
    fn visible_colleague_ids_includes_self_and_shared_group_members() {
        let mut store = GroupMembershipStore::new();
        store.restore_member(1, 10);
        store.restore_member(1, 11);
        store.restore_member(2, 11);
        store.restore_member(2, 12);
        store.restore_member(3, 13);

        let mut visible = store.visible_colleague_ids(10);
        visible.sort();
        assert_eq!(visible, vec![10, 11]);

        let mut visible_b = store.visible_colleague_ids(11);
        visible_b.sort();
        assert_eq!(visible_b, vec![10, 11, 12]);

        assert_eq!(store.visible_colleague_ids(13), vec![13]);
        assert!(store.shares_group_with(10, 11));
        assert!(!store.shares_group_with(10, 12));
        assert!(store.shares_group_with(10, 10));
    }

    #[test]
    fn create_requires_permission_and_same_org() {
        let admin = creator_actor();
        assert!(GroupMembershipStore::authorize_create_group(&admin, 1).is_ok());
        assert_eq!(
            GroupMembershipStore::authorize_create_group(&admin, 2),
            Err(AppError::Forbidden)
        );
        assert_eq!(
            GroupMembershipStore::authorize_create_group(&viewer_actor(), 1),
            Err(AppError::Forbidden)
        );
    }

    #[test]
    fn add_and_remove_members_with_group_permission() {
        let admin = creator_actor();
        let (mut store, group) = store_with_group_admin(admin.id);

        let detail = store.add_member(&admin, &group, 42).unwrap();
        assert_eq!(detail.member_ids, vec![admin.id, 42]);
        assert!(store.is_member(5, 42));

        let detail = store.remove_member(&admin, &group, 42).unwrap();
        assert_eq!(detail.member_ids, vec![admin.id]);
        assert!(!store.is_member(5, 42));
    }

    #[test]
    fn add_member_forbidden_without_group_permission() {
        let mut store = GroupMembershipStore::new();
        let group = sample_group();
        store.ensure_group(group.group_id());
        assert_eq!(
            store.add_member(&viewer_actor(), &group, 42),
            Err(AppError::Forbidden)
        );
    }

    #[test]
    fn group_scoped_edit_permission_allows_rename_auth() {
        let admin = creator_actor();
        let (mut store, group) = store_with_group_admin(admin.id);
        store.add_member(&admin, &group, 11).unwrap();
        store
            .set_member_permissions_as_actor(
                &admin,
                &group,
                11,
                GroupMemberPermissions {
                    can_approve_vacations: false,
                    can_approve_own_vacations: false,
                    can_edit_group: true,
                    can_add_members: false,
                    can_remove_members: false,
                    can_delete_group: false,
                },
            )
            .unwrap();

        assert!(store
            .authorize_update_group(&viewer_actor(), &group)
            .is_ok());
    }

    #[test]
    fn delete_requires_group_delete_permission() {
        let admin = creator_actor();
        let (store, group) = store_with_group_admin(admin.id);
        assert!(store.authorize_delete_group(&admin, &group).is_ok());
        assert_eq!(
            store.authorize_delete_group(&viewer_actor(), &group),
            Err(AppError::Forbidden)
        );
    }

    #[test]
    fn member_can_read_own_permissions() {
        let admin = creator_actor();
        let (mut store, group) = store_with_group_admin(admin.id);
        store.add_member(&admin, &group, 11).unwrap();

        let perms = store
            .get_member_permissions_as_actor(&viewer_actor(), &group, 11)
            .unwrap();
        assert_eq!(perms, GroupMemberPermissions::default());
    }

    #[test]
    fn member_cannot_read_others_permissions_without_edit() {
        let admin = creator_actor();
        let (mut store, group) = store_with_group_admin(admin.id);
        store.add_member(&admin, &group, 11).unwrap();
        store.add_member(&admin, &group, 12).unwrap();

        assert_eq!(
            store.get_member_permissions_as_actor(&viewer_actor(), &group, 12),
            Err(AppError::Forbidden)
        );
    }

    #[test]
    fn replace_employee_groups_rewrites_membership() {
        let mut store = GroupMembershipStore::new();
        store.ensure_group(1);
        store.ensure_group(2);
        store.replace_employee_groups(7, &[1, 2]);
        store.replace_employee_groups(7, &[2]);
        assert!(!store.is_member(1, 7));
        assert!(store.is_member(2, 7));
        assert_eq!(store.group_ids_for_employee(7), vec![2]);
    }

    #[test]
    fn set_employee_permissions_requires_create_groups() {
        let mut store = GroupMembershipStore::new();
        let perms = EmployeePermissions {
            create_groups: true,
            ..EmployeePermissions::default_employee()
        };
        assert_eq!(
            store.set_employee_permissions_as_actor(&viewer_actor(), 99, perms.clone()),
            Err(AppError::Forbidden)
        );
        let saved = store
            .set_employee_permissions_as_actor(&creator_actor(), 99, perms.clone())
            .unwrap();
        assert_eq!(saved, perms);
        assert_eq!(store.employee_permissions(99), perms);
    }

    #[test]
    fn clear_group_drops_members_and_member_perms() {
        let admin = creator_actor();
        let (mut store, group) = store_with_group_admin(admin.id);
        store.add_member(&admin, &group, 3).unwrap();
        store
            .set_member_permissions_as_actor(
                &admin,
                &group,
                3,
                GroupMemberPermissions {
                    can_approve_vacations: true,
                    can_approve_own_vacations: false,
                    can_edit_group: false,
                    can_add_members: false,
                    can_remove_members: false,
                    can_delete_group: false,
                },
            )
            .unwrap();
        store.clear_group(group.group_id());
        assert!(!store.is_member(5, 3));
        assert_eq!(
            store.member_permissions(5, 3),
            GroupMemberPermissions::default()
        );
    }

    #[test]
    fn admin_can_set_member_permissions_without_group_edit() {
        let admin = creator_actor();
        let (mut store, group) = store_with_group_admin(admin.id);
        store.admin_add_member(&group, 42);
        let perms = GroupMemberPermissions {
            can_approve_vacations: true,
            can_approve_own_vacations: false,
            can_edit_group: true,
            can_add_members: false,
            can_remove_members: false,
            can_delete_group: false,
        };
        let saved = store
            .admin_set_member_permissions(&group, 42, perms.clone())
            .unwrap();
        assert_eq!(saved, perms);
        assert_eq!(
            store.admin_get_member_permissions(&group, 42).unwrap(),
            perms
        );
        assert_eq!(
            store.admin_get_member_permissions(&group, 99),
            Err(AppError::NotFound)
        );
    }

    #[test]
    fn actor_can_vote_with_group_or_global_approve() {
        let admin = creator_actor();
        let (mut store, group) = store_with_group_admin(admin.id);
        store.add_member(&admin, &group, 11).unwrap();
        store
            .set_member_permissions_as_actor(
                &admin,
                &group,
                11,
                GroupMemberPermissions {
                    can_approve_vacations: true,
                    can_approve_own_vacations: false,
                    can_edit_group: false,
                    can_add_members: false,
                    can_remove_members: false,
                    can_delete_group: false,
                },
            )
            .unwrap();

        // Approving someone else's vacation.
        assert!(store.actor_can_vote_for_group(&viewer_actor(), group.group_id(), admin.id));
        assert!(!store.actor_can_vote_for_group(&viewer_actor(), 99, admin.id));

        // Approver cannot vote on their own vacation without own-approve permission.
        assert!(!store.actor_can_vote_for_group(&viewer_actor(), group.group_id(), 11));

        store
            .set_member_permissions_as_actor(
                &admin,
                &group,
                11,
                GroupMemberPermissions {
                    can_approve_vacations: true,
                    can_approve_own_vacations: true,
                    can_edit_group: false,
                    can_add_members: false,
                    can_remove_members: false,
                    can_delete_group: false,
                },
            )
            .unwrap();
        assert!(store.actor_can_vote_for_group(&viewer_actor(), group.group_id(), 11));

        let mut global = viewer_actor();
        global.id = 99;
        global.permissions.approve_vacations = true;
        assert!(store.actor_can_vote_for_group(&global, group.group_id(), admin.id));
        // Global approve_vacations alone does not allow approving own.
        assert!(!store.actor_can_vote_for_group(&global, group.group_id(), 99));

        assert_eq!(
            store.approval_group_ids_for_employee(admin.id),
            vec![group.group_id()]
        );
    }

    #[test]
    fn anyone_can_approve_false_when_no_approvers() {
        let mut store = GroupMembershipStore::new();
        store.set_employee_permissions(1, EmployeePermissions::default_employee());
        store.set_employee_permissions(2, EmployeePermissions::default_employee());
        assert!(!store.anyone_can_approve_vacation_for(1, &[1, 2]));
    }

    #[test]
    fn anyone_can_approve_via_own_global_or_group() {
        let mut store = GroupMembershipStore::new();
        store.set_employee_permissions(
            1,
            EmployeePermissions {
                approve_own_vacations: true,
                ..EmployeePermissions::default_employee()
            },
        );
        assert!(store.anyone_can_approve_vacation_for(1, &[1]));

        let mut store = GroupMembershipStore::new();
        store.set_employee_permissions(1, EmployeePermissions::default_employee());
        store.set_employee_permissions(
            2,
            EmployeePermissions {
                approve_vacations: true,
                ..EmployeePermissions::default_employee()
            },
        );
        assert!(store.anyone_can_approve_vacation_for(1, &[1, 2]));

        let admin = creator_actor();
        let (mut store, group) = store_with_group_admin(admin.id);
        // Group admin has can_approve_vacations, so they can approve member 11.
        store.add_member(&admin, &group, 11).unwrap();
        store.set_employee_permissions(11, EmployeePermissions::default_employee());
        assert!(store.anyone_can_approve_vacation_for(11, &[admin.id, 11]));

        // Alone with only can_approve_vacations (not own) → cannot approve self.
        let mut store = GroupMembershipStore::new();
        store.ensure_group(5);
        store.restore_member(5, 11);
        store.set_employee_permissions(11, EmployeePermissions::default_employee());
        store.restore_member_permissions(
            5,
            11,
            GroupMemberPermissions {
                can_approve_vacations: true,
                can_approve_own_vacations: false,
                ..GroupMemberPermissions::default()
            },
        );
        assert!(!store.anyone_can_approve_vacation_for(11, &[11]));

        // Alone with can_approve_own_vacations → can self-approve.
        store.restore_member_permissions(
            5,
            11,
            GroupMemberPermissions {
                can_approve_vacations: true,
                can_approve_own_vacations: true,
                ..GroupMemberPermissions::default()
            },
        );
        assert!(store.anyone_can_approve_vacation_for(11, &[11]));

        // Member alone in a group without approve permission → nobody can approve.
        let mut store = GroupMembershipStore::new();
        store.ensure_group(5);
        store.restore_member(5, 11);
        store.set_employee_permissions(11, EmployeePermissions::default_employee());
        store.restore_member_permissions(5, 11, GroupMemberPermissions::default());
        assert!(!store.anyone_can_approve_vacation_for(11, &[11]));
    }

    #[test]
    fn owner_can_request_own_cancellation_without_being_group_creator() {
        let owner = viewer_actor(); // id 11, no create_groups / creator perms
        let mut store = GroupMembershipStore::new();
        store.ensure_group(5);
        store.restore_member(5, owner.id);
        store.restore_member_permissions(5, owner.id, GroupMemberPermissions::default());

        assert!(store.actor_can_request_cancellation(&owner, owner.id));
        assert!(!store.actor_can_request_cancellation(&owner, 99));
    }

    #[test]
    fn group_creator_can_still_request_member_cancellation() {
        let admin = creator_actor();
        let (mut store, group) = store_with_group_admin(admin.id);
        store.add_member(&admin, &group, 42).unwrap();

        assert!(store.actor_can_request_cancellation(&admin, 42));
        assert!(!store.actor_can_request_cancellation(&viewer_actor(), 42));
    }

    #[test]
    fn approver_ids_includes_global_and_group_approvers() {
        let admin = creator_actor();
        let (mut store, group) = store_with_group_admin(admin.id);
        store.add_member(&admin, &group, 11).unwrap();
        store.set_employee_permissions(11, EmployeePermissions::default_employee());
        store.set_employee_permissions(
            admin.id,
            EmployeePermissions {
                approve_vacations: true,
                ..EmployeePermissions::default_employee()
            },
        );

        let ids = store.approver_ids_for_vacation(11, &[admin.id, 11]);
        assert!(ids.contains(&admin.id));
        assert!(!ids.contains(&11));
    }
}
