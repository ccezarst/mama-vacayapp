use crate::Date;
use crate::events::Event;
use crate::organizations::Organization;
use crate::auth::{EmployeeSession};

pub struct Employee{
    username: String,
    email: String,
    photo: Option<String>,
    employeeID: EmployeeID,
    session: EmployeeSession,
    organization: Organization,
    groups: Vec<Group>,
    details: (String, String),
    ePerms: EPermissions,
    // TODO: max/min vac days
}

#[derive(Clone)]
pub struct Vacation{
    startDate: Date,
    finalDate: Date,
    employeeID: EmployeeID,
    vacationID: VacationID,
    status: VacationStatus,
    details: String,
    denial_reason: String,
}

#[derive(Clone, Copy, Debug)]
pub enum VacationStatus{
    AwaitingConfirmation,
    Approved{approvedBy: EmployeeID, coveredBy: EmployeeID},
    Denied{deniedBy: EmployeeID},
    /// Approved vacation with a pending cancellation request.
    /// `approvedBy` / `coveredBy` are kept so a denied cancel can restore Approval.
    CancellationRequested {
        requestedBy: EmployeeID,
        approvedBy: EmployeeID,
        coveredBy: EmployeeID,
    },
    Cancelled {
        cancelledBy: EmployeeID,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VacationID {
    id: u32,
}

impl VacationID {
    pub fn new(id: u32) -> Self {
        Self { id }
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}

impl Vacation {
    pub fn new(
        startDate: Date,
        finalDate: Date,
        employeeID: EmployeeID,
        vacationID: VacationID,
        status: VacationStatus,
    ) -> Self {
        Self {
            startDate,
            finalDate,
            employeeID,
            vacationID,
            status,
            details: String::new(),
            denial_reason: String::new(),
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = details.into();
        self
    }

    pub fn with_denial_reason(mut self, reason: impl Into<String>) -> Self {
        self.denial_reason = reason.into();
        self
    }

    pub fn start_date(&self) -> Date {
        self.startDate
    }

    pub fn final_date(&self) -> Date {
        self.finalDate
    }

    pub fn employee_id(&self) -> EmployeeID {
        self.employeeID
    }

    pub fn vacation_id(&self) -> VacationID {
        self.vacationID
    }

    pub fn status(&self) -> &VacationStatus {
        &self.status
    }

    pub fn details(&self) -> &str {
        &self.details
    }

    pub fn denial_reason(&self) -> &str {
        &self.denial_reason
    }

    pub fn approved_by(&self) -> Option<EmployeeID> {
        match &self.status {
            VacationStatus::Approved { approvedBy, .. } => Some(*approvedBy),
            VacationStatus::CancellationRequested { approvedBy, .. } => Some(*approvedBy),
            _ => None,
        }
    }

    pub fn covered_by(&self) -> Option<EmployeeID> {
        match &self.status {
            VacationStatus::Approved { coveredBy, .. } => Some(*coveredBy),
            VacationStatus::CancellationRequested { coveredBy, .. } => Some(*coveredBy),
            _ => None,
        }
    }

    pub fn denied_by(&self) -> Option<EmployeeID> {
        match &self.status {
            VacationStatus::Denied { deniedBy } => Some(*deniedBy),
            _ => None,
        }
    }

    pub fn cancellation_requested_by(&self) -> Option<EmployeeID> {
        match &self.status {
            VacationStatus::CancellationRequested { requestedBy, .. } => Some(*requestedBy),
            _ => None,
        }
    }

    pub fn cancelled_by(&self) -> Option<EmployeeID> {
        match &self.status {
            VacationStatus::Cancelled { cancelledBy } => Some(*cancelledBy),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmployeeID {
    id: u32,
}

impl EmployeeID {
    pub fn new(id: u32) -> Self {
        Self { id }
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}

pub struct Group{

}

pub struct GroupID{
    id: u32
}

pub struct EPermissions{

}