//! Compose outbound emails from application [`EventType`]s.
//!
//! Keeps recipient selection and copy separate from SMTP transport so any
//! [`crate::email::Mailer`] can deliver the resulting messages.

use std::collections::HashMap;

use crate::config::EmailSettings;
use crate::email::{EmailAddress, EmailBody, EmailErrors, EmailMessage};
use crate::employees::{EmployeeID, VacationID};
use crate::events::EventType;

/// Directory contact used when addressing event emails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmployeeContact {
    pub id: u32,
    pub email: String,
    pub username: String,
}

impl EmployeeContact {
    pub fn new(id: u32, email: impl Into<String>, username: impl Into<String>) -> Self {
        Self {
            id,
            email: email.into(),
            username: username.into(),
        }
    }

    pub fn as_address(&self) -> EmailAddress {
        EmailAddress::with_name(&self.email, &self.username)
    }
}

/// Resolved inputs for turning one event into zero or more emails.
#[derive(Debug, Clone)]
pub struct EventEmailContext {
    pub settings: EmailSettings,
    pub employees: HashMap<u32, EmployeeContact>,
    /// Pre-resolved approver employee IDs (vacation request / cancellation).
    pub approver_ids: Vec<u32>,
}

impl EventEmailContext {
    pub fn from_settings(settings: EmailSettings) -> Self {
        Self {
            settings,
            employees: HashMap::new(),
            approver_ids: Vec::new(),
        }
    }

    pub fn with_employees(mut self, employees: impl IntoIterator<Item = EmployeeContact>) -> Self {
        self.employees = employees.into_iter().map(|e| (e.id, e)).collect();
        self
    }

    pub fn with_approver_ids(mut self, ids: impl IntoIterator<Item = u32>) -> Self {
        self.approver_ids = ids.into_iter().collect();
        self
    }

    fn from_address(&self) -> EmailAddress {
        match &self.settings.from_name {
            Some(name) => EmailAddress::with_name(&self.settings.from_address, name),
            None => EmailAddress::new(&self.settings.from_address),
        }
    }

    fn contact(&self, id: u32) -> Option<&EmployeeContact> {
        self.employees.get(&id)
    }

    fn contact_for(&self, id: EmployeeID) -> Option<&EmployeeContact> {
        self.contact(id.id())
    }
}

/// Builds transport-ready messages for a single application event.
///
/// Unsupported / uninteresting event types yield an empty list (not an error).
pub fn messages_for_event(
    event: &EventType,
    ctx: &EventEmailContext,
) -> Result<Vec<EmailMessage>, EmailErrors> {
    match event {
        EventType::EmployeeAdded { employee_id, .. } => {
            employee_added_messages(*employee_id, ctx)
        }
        EventType::VacationRequest {
            employee_id,
            vacation_id,
        } => vacation_request_messages(*employee_id, *vacation_id, ctx),
        EventType::VacationCancellationRequested {
            employee_id,
            vacation_id,
            requested_by,
        } => cancellation_requested_messages(*employee_id, *vacation_id, *requested_by, ctx),
        EventType::VacationApproval {
            employee_id,
            vacation_id,
            approved_by,
            ..
        } => vacation_decision_messages(
            *employee_id,
            *vacation_id,
            *approved_by,
            true,
            false,
            None,
            ctx,
        ),
        EventType::VacationDenial {
            employee_id,
            vacation_id,
            denied_by,
            reason,
            ..
        } => vacation_decision_messages(
            *employee_id,
            *vacation_id,
            *denied_by,
            false,
            false,
            reason.as_deref(),
            ctx,
        ),
        EventType::VacationCancellationApproved {
            employee_id,
            vacation_id,
            cancelled_by,
            ..
        } => vacation_decision_messages(
            *employee_id,
            *vacation_id,
            *cancelled_by,
            true,
            true,
            None,
            ctx,
        ),
        EventType::VacationCancellationDenied {
            employee_id,
            vacation_id,
            denied_by,
            ..
        } => vacation_decision_messages(
            *employee_id,
            *vacation_id,
            *denied_by,
            false,
            true,
            None,
            ctx,
        ),
        _ => Ok(Vec::new()),
    }
}

fn employee_added_messages(
    employee_id: EmployeeID,
    ctx: &EventEmailContext,
) -> Result<Vec<EmailMessage>, EmailErrors> {
    let Some(employee) = ctx.contact_for(employee_id) else {
        return Ok(Vec::new());
    };
    let msg = EmailMessage::builder()
        .from(ctx.from_address())
        .to(employee.as_address())
        .subject("Your employee account was created")
        .text_body(format!(
            "Hi {name},\n\n\
             An employee account has been created for you on Vaction App.\n\
             You can sign in with this email address ({email}).\n\n\
             — Vaction App\n",
            name = employee.username,
            email = employee.email,
        ))
        .build()?;
    Ok(vec![msg])
}

fn vacation_request_messages(
    employee_id: EmployeeID,
    vacation_id: VacationID,
    ctx: &EventEmailContext,
) -> Result<Vec<EmailMessage>, EmailErrors> {
    let requester_name = ctx
        .contact_for(employee_id)
        .map(|c| c.username.as_str())
        .unwrap_or("An employee");
    let mut messages = Vec::new();

    for approver_id in &ctx.approver_ids {
        // Don't notify the requester about their own request.
        if *approver_id == employee_id.id() {
            continue;
        }
        let Some(approver) = ctx.contact(*approver_id) else {
            continue;
        };
        messages.push(
            EmailMessage::builder()
                .from(ctx.from_address())
                .to(approver.as_address())
                .subject(format!("Vacation request from {requester_name}"))
                .text_body(format!(
                    "Hi {name},\n\n\
                     {requester_name} submitted a vacation request (#{vid}) that needs your approval.\n\
                     Open Vaction App to approve or deny it.\n\n\
                     — Vaction App\n",
                    name = approver.username,
                    vid = vacation_id.id(),
                ))
                .build()?,
        );
    }
    Ok(messages)
}

fn cancellation_requested_messages(
    employee_id: EmployeeID,
    vacation_id: VacationID,
    requested_by: EmployeeID,
    ctx: &EventEmailContext,
) -> Result<Vec<EmailMessage>, EmailErrors> {
    let owner_name = ctx
        .contact_for(employee_id)
        .map(|c| c.username.as_str())
        .unwrap_or("An employee");
    let requester_name = ctx
        .contact_for(requested_by)
        .map(|c| c.username.as_str())
        .unwrap_or("Someone");
    let mut messages = Vec::new();

    for approver_id in &ctx.approver_ids {
        if *approver_id == requested_by.id() {
            continue;
        }
        let Some(approver) = ctx.contact(*approver_id) else {
            continue;
        };
        messages.push(
            EmailMessage::builder()
                .from(ctx.from_address())
                .to(approver.as_address())
                .subject(format!("Cancellation request for {owner_name}'s vacation"))
                .text_body(format!(
                    "Hi {name},\n\n\
                     {requester_name} requested cancellation of vacation #{vid} \
                     for {owner_name}. It needs your approval.\n\
                     Open Vaction App to approve or deny the cancellation.\n\n\
                     — Vaction App\n",
                    name = approver.username,
                    vid = vacation_id.id(),
                ))
                .build()?,
        );
    }

    // If someone else requested cancel on the owner's vacation, tell the owner.
    if requested_by.id() != employee_id.id() {
        if let Some(owner) = ctx.contact_for(employee_id) {
            messages.push(
                EmailMessage::builder()
                    .from(ctx.from_address())
                    .to(owner.as_address())
                    .subject("Cancellation requested for your vacation")
                    .text_body(format!(
                        "Hi {name},\n\n\
                         {requester_name} requested cancellation of your vacation #{vid}.\n\
                         Approvers will review the request.\n\n\
                         — Vaction App\n",
                        name = owner.username,
                        vid = vacation_id.id(),
                    ))
                    .build()?,
            );
        }
    }

    Ok(messages)
}

fn vacation_decision_messages(
    employee_id: EmployeeID,
    vacation_id: VacationID,
    decided_by: Option<EmployeeID>,
    approved: bool,
    is_cancellation: bool,
    reason: Option<&str>,
    ctx: &EventEmailContext,
) -> Result<Vec<EmailMessage>, EmailErrors> {
    let Some(employee) = ctx.contact_for(employee_id) else {
        return Ok(Vec::new());
    };
    let decider_name = decided_by
        .and_then(|id| ctx.contact_for(id))
        .map(|c| c.username.as_str())
        .unwrap_or("an administrator");

    let (subject, body) = if is_cancellation {
        if approved {
            (
                "Your vacation cancellation was approved".to_string(),
                format!(
                    "Hi {name},\n\n\
                     Your cancellation request for vacation #{vid} was approved by {decider}.\n\
                     The vacation is now cancelled.\n\n\
                     — Vaction App\n",
                    name = employee.username,
                    vid = vacation_id.id(),
                    decider = decider_name,
                ),
            )
        } else {
            (
                "Your vacation cancellation was denied".to_string(),
                format!(
                    "Hi {name},\n\n\
                     Your cancellation request for vacation #{vid} was denied by {decider}.\n\
                     The vacation remains approved.\n\n\
                     — Vaction App\n",
                    name = employee.username,
                    vid = vacation_id.id(),
                    decider = decider_name,
                ),
            )
        }
    } else if approved {
        (
            "Your vacation request was approved".to_string(),
            format!(
                "Hi {name},\n\n\
                 Your vacation request #{vid} was approved by {decider}.\n\n\
                 — Vaction App\n",
                name = employee.username,
                vid = vacation_id.id(),
                decider = decider_name,
            ),
        )
    } else {
        let reason_line = reason
            .filter(|r| !r.is_empty())
            .map(|r| format!("\nReason: {r}\n"))
            .unwrap_or_default();
        (
            "Your vacation request was denied".to_string(),
            format!(
                "Hi {name},\n\n\
                 Your vacation request #{vid} was denied by {decider}.{reason_line}\n\
                 — Vaction App\n",
                name = employee.username,
                vid = vacation_id.id(),
                decider = decider_name,
                reason_line = reason_line,
            ),
        )
    };

    let msg = EmailMessage::builder()
        .from(ctx.from_address())
        .to(employee.as_address())
        .subject(subject)
        .body(EmailBody::Text(body))
        .build()?;
    Ok(vec![msg])
}

/// Whether this event type participates in the outbound email system.
pub fn event_sends_email(event: &EventType) -> bool {
    matches!(
        event,
        EventType::EmployeeAdded { .. }
            | EventType::VacationRequest { .. }
            | EventType::VacationCancellationRequested { .. }
            | EventType::VacationApproval { .. }
            | EventType::VacationDenial { .. }
            | EventType::VacationCancellationApproved { .. }
            | EventType::VacationCancellationDenied { .. }
    )
}

/// Build a password-change confirmation message for an explicit recipient email.
pub fn password_changed_message(
    to_email: &str,
    username: &str,
    settings: &EmailSettings,
) -> Result<EmailMessage, EmailErrors> {
    let from = match &settings.from_name {
        Some(name) => EmailAddress::with_name(&settings.from_address, name),
        None => EmailAddress::new(&settings.from_address),
    };
    EmailMessage::builder()
        .from(from)
        .to(EmailAddress::with_name(to_email, username))
        .subject("Your password was changed")
        .text_body(format!(
            "Hi {username},\n\n\
             Your Vaction App password was changed.\n\
             If you did not make this change, contact your administrator.\n\n\
             — Vaction App\n",
        ))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EmailSettings;
    use crate::users::UserID;

    fn settings() -> EmailSettings {
        EmailSettings {
            from_address: "noreply@test.local".into(),
            from_name: Some("Vaction".into()),
            ..EmailSettings::default()
        }
    }

    fn ctx_with(employees: Vec<EmployeeContact>, approvers: Vec<u32>) -> EventEmailContext {
        EventEmailContext::from_settings(settings())
            .with_employees(employees)
            .with_approver_ids(approvers)
    }

    #[test]
    fn employee_added_emails_new_account() {
        let ctx = ctx_with(
            vec![EmployeeContact::new(7, "alice@ex.com", "Alice")],
            vec![],
        );
        let msgs = messages_for_event(
            &EventType::EmployeeAdded {
                employee_id: EmployeeID::new(7),
                added_by: UserID::new(1),
            },
            &ctx,
        )
        .unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].to[0].email, "alice@ex.com");
        assert!(msgs[0].subject.contains("created"));
    }

    #[test]
    fn vacation_request_emails_approvers_not_requester() {
        let ctx = ctx_with(
            vec![
                EmployeeContact::new(1, "bob@ex.com", "Bob"),
                EmployeeContact::new(2, "mgr@ex.com", "Manager"),
                EmployeeContact::new(3, "hr@ex.com", "HR"),
            ],
            vec![1, 2, 3],
        );
        let msgs = messages_for_event(
            &EventType::VacationRequest {
                employee_id: EmployeeID::new(1),
                vacation_id: VacationID::new(99),
            },
            &ctx,
        )
        .unwrap();
        let recipients: Vec<_> = msgs.iter().map(|m| m.to[0].email.as_str()).collect();
        assert_eq!(recipients, vec!["mgr@ex.com", "hr@ex.com"]);
        assert!(msgs[0].subject.contains("Bob"));
    }

    #[test]
    fn vacation_approval_emails_employee() {
        let ctx = ctx_with(
            vec![
                EmployeeContact::new(1, "bob@ex.com", "Bob"),
                EmployeeContact::new(2, "mgr@ex.com", "Manager"),
            ],
            vec![],
        );
        let msgs = messages_for_event(
            &EventType::VacationApproval {
                employee_id: EmployeeID::new(1),
                vacation_id: VacationID::new(5),
                approved_by: Some(EmployeeID::new(2)),
                approved_by_user: None,
            },
            &ctx,
        )
        .unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].to[0].email, "bob@ex.com");
        assert!(msgs[0].subject.contains("approved"));
    }

    #[test]
    fn cancellation_request_notifies_approvers_and_owner_when_third_party() {
        let ctx = ctx_with(
            vec![
                EmployeeContact::new(1, "bob@ex.com", "Bob"),
                EmployeeContact::new(2, "mgr@ex.com", "Manager"),
                EmployeeContact::new(3, "creator@ex.com", "Creator"),
            ],
            vec![2],
        );
        let msgs = messages_for_event(
            &EventType::VacationCancellationRequested {
                employee_id: EmployeeID::new(1),
                vacation_id: VacationID::new(8),
                requested_by: EmployeeID::new(3),
            },
            &ctx,
        )
        .unwrap();
        let recipients: Vec<_> = msgs.iter().map(|m| m.to[0].email.as_str()).collect();
        assert!(recipients.contains(&"mgr@ex.com"));
        assert!(recipients.contains(&"bob@ex.com"));
    }
}
