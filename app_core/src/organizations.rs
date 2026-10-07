use crate::users::User;
use crate::events::Event;
use crate::employees::Employee;

#[derive(Clone)]
pub struct Organization{
    ownerID: u32,
    employeeIDs: Vec<u32>,
    groupIDs: Vec<u32>,
    name: String,
    organizationID: OrganizationID,
    eventHistoryIDs: Vec<u32>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrganizationID{
    id: u32
}

impl OrganizationID {
    pub fn new(id: u32) -> Self {
        Self { id }
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}

impl Organization {
    pub fn new(owner_id: u32, name: String, organization_id: OrganizationID) -> Self {
        Self {
            ownerID: owner_id,
            employeeIDs: Vec::new(),
            groupIDs: Vec::new(),
            name,
            organizationID: organization_id,
            eventHistoryIDs: Vec::new(),
        }
    }

    pub fn owner_id(&self) -> u32 {
        self.ownerID
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn organization_id(&self) -> OrganizationID {
        self.organizationID
    }

    pub fn employee_ids(&self) -> &[u32] {
        &self.employeeIDs
    }

    pub fn group_ids(&self) -> &[u32] {
        &self.groupIDs
    }

    pub fn with_employee_ids(mut self, employee_ids: Vec<u32>) -> Self {
        self.employeeIDs = employee_ids;
        self
    }

    pub fn with_group_ids(mut self, group_ids: Vec<u32>) -> Self {
        self.groupIDs = group_ids;
        self
    }
}