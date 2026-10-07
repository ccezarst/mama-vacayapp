use crate::employees::{EmployeeID, VacationID};
use crate::organizations::Organization;
use crate::users::UserID;
use crate::Date;

#[derive(Debug, Clone)]
pub struct Event {
    date: Date,
    event_type: EventType,
}

impl Event {
    pub fn new(date: Date, event_type: EventType) -> Self {
        Self { date, event_type }
    }

    pub fn date(&self) -> Date {
        self.date
    }

    pub fn event_type(&self) -> &EventType {
        &self.event_type
    }
}

#[derive(Debug, Clone)]
pub enum EventType {
    VacationRequest {
        employee_id: EmployeeID,
        vacation_id: VacationID,
    },
    VacationApproval {
        employee_id: EmployeeID,
        vacation_id: VacationID,
        approved_by: Option<EmployeeID>,
        approved_by_user: Option<UserID>,
    },
    VacationDenial {
        employee_id: EmployeeID,
        vacation_id: VacationID,
        denied_by: Option<EmployeeID>,
        denied_by_user: Option<UserID>,
        reason: Option<String>,
    },
    VacationGroupVote {
        employee_id: EmployeeID,
        vacation_id: VacationID,
        group_id: u32,
        decided_by: EmployeeID,
        approved: bool,
    },
    VacationModified {
        employee_id: EmployeeID,
        vacation_id: VacationID,
        modified_by_employee: Option<EmployeeID>,
        modified_by_user: Option<UserID>,
    },
    VacationDeleted {
        employee_id: EmployeeID,
        vacation_id: VacationID,
        deleted_by_employee: Option<EmployeeID>,
        deleted_by_user: Option<UserID>,
    },
    VacationCancellationRequested {
        employee_id: EmployeeID,
        vacation_id: VacationID,
        requested_by: EmployeeID,
    },
    VacationCancellationApproved {
        employee_id: EmployeeID,
        vacation_id: VacationID,
        cancelled_by: Option<EmployeeID>,
        cancelled_by_user: Option<UserID>,
    },
    VacationCancellationDenied {
        employee_id: EmployeeID,
        vacation_id: VacationID,
        denied_by: Option<EmployeeID>,
        denied_by_user: Option<UserID>,
    },
    UserAdded {
        user_id: UserID,
    },
    UserModified {
        user_id: UserID,
    },
    UserDeleted {
        user_id: UserID,
    },
    GroupAdded {
        group_id: u32,
    },
    GroupModified {
        group_id: u32,
    },
    GroupDeleted {
        group_id: u32,
    },
    EmployeeAdded {
        employee_id: EmployeeID,
        added_by: UserID,
    },
    EmployeeModified {
        employee_id: EmployeeID,
        modified_by_user: Option<UserID>,
        modified_by_employee: Option<EmployeeID>,
    },
    EmployeeDeleted {
        employee_id: EmployeeID,
        deleted_by: UserID,
    },
}

impl EventType {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::VacationRequest { .. } => "vacation_request",
            Self::VacationApproval { .. } => "vacation_approval",
            Self::VacationDenial { .. } => "vacation_denial",
            Self::VacationGroupVote { .. } => "vacation_group_vote",
            Self::VacationModified { .. } => "vacation_modified",
            Self::VacationDeleted { .. } => "vacation_deleted",
            Self::VacationCancellationRequested { .. } => "vacation_cancellation_requested",
            Self::VacationCancellationApproved { .. } => "vacation_cancellation_approved",
            Self::VacationCancellationDenied { .. } => "vacation_cancellation_denied",
            Self::UserAdded { .. } => "user_added",
            Self::UserModified { .. } => "user_modified",
            Self::UserDeleted { .. } => "user_deleted",
            Self::GroupAdded { .. } => "group_added",
            Self::GroupModified { .. } => "group_modified",
            Self::GroupDeleted { .. } => "group_deleted",
            Self::EmployeeAdded { .. } => "employee_added",
            Self::EmployeeModified { .. } => "employee_modified",
            Self::EmployeeDeleted { .. } => "employee_deleted",
        }
    }
}

pub trait ObjectWithEvents {
    fn get_event_history(org: &Organization) -> Option<Vec<Event>>;
    fn get_event_history_by_id(org_id: u32) -> Option<Vec<Event>>;
}
