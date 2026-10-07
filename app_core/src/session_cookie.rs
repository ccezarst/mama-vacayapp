use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::api::{AppError, VacationAppApi};
use crate::auth::{AuthError, Session, SessionType};
use crate::employees::EmployeeID;
use crate::users::UserID;

pub const USER_SESSION_COOKIE: &str = "vaction_user_session";
pub const EMPLOYEE_SESSION_COOKIE: &str = "vaction_employee_session";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionKind {
    User,
    Employee,
}

impl SessionKind {
    pub fn cookie_name(self) -> &'static str {
        match self {
            Self::User => USER_SESSION_COOKIE,
            Self::Employee => EMPLOYEE_SESSION_COOKIE,
        }
    }

    pub fn from_role(role: &str) -> Self {
        if role == "employee" {
            Self::Employee
        } else {
            Self::User
        }
    }
}

/// Serializable session snapshot stored in the browser cookie.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: u32,
    pub kind: SessionKind,
    pub subject_id: u32,
}

impl SessionInfo {
    pub fn from_session(session: &Session) -> Self {
        match session.session_type() {
            SessionType::User { userID } => Self {
                session_id: session.id(),
                kind: SessionKind::User,
                subject_id: userID.id(),
            },
            SessionType::Employee { employeeID } => Self {
                session_id: session.id(),
                kind: SessionKind::Employee,
                subject_id: employeeID.id(),
            },
        }
    }

    pub fn user_id(&self) -> Result<UserID, AppError> {
        if self.kind == SessionKind::User {
            Ok(UserID::new(self.subject_id))
        } else {
            Err(AppError::Forbidden)
        }
    }

    pub fn employee_id(&self) -> Result<EmployeeID, AppError> {
        if self.kind == SessionKind::Employee {
            Ok(EmployeeID::new(self.subject_id))
        } else {
            Err(AppError::Forbidden)
        }
    }
}

pub fn parse_cookie_header(header: &str) -> HashMap<String, String> {
    header
        .split(';')
        .filter_map(|part| {
            let (name, value) = part.trim().split_once('=')?;
            Some((name.to_string(), value.to_string()))
        })
        .collect()
}

pub fn read_session_cookie(header: &str, kind: SessionKind) -> Result<SessionInfo, AppError> {
    let cookies = parse_cookie_header(header);
    let raw = cookies
        .get(kind.cookie_name())
        .ok_or(AppError::Auth(AuthError::SessionNotFound))?;
    let decoded = url_decode(raw);
    serde_json::from_str(&decoded)
        .map_err(|_| AppError::Auth(AuthError::SessionNotFound))
}

pub fn build_set_cookie(info: &SessionInfo) -> String {
    let encoded = url_encode(&serde_json::to_string(info).expect("session json"));
    // Separate cookie names so one admin + one employee session can coexist.
    format!(
        "{}={encoded}; Path=/; HttpOnly; SameSite=Lax",
        info.kind.cookie_name()
    )
}

pub fn build_clear_cookie(kind: SessionKind) -> String {
    format!(
        "{}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0",
        kind.cookie_name()
    )
}

/// Reads whichever of the two portal cookies may be present (without validating).
pub fn read_optional_sessions(header: &str) -> DualSessions {
    DualSessions {
        admin: read_session_cookie(header, SessionKind::User).ok(),
        employee: read_session_cookie(header, SessionKind::Employee).ok(),
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DualSessions {
    pub admin: Option<SessionInfo>,
    pub employee: Option<SessionInfo>,
}

pub async fn validate_cookie_session<A: VacationAppApi + ?Sized>(
    app: &A,
    info: &SessionInfo,
) -> Result<Session, AppError> {
    let session = app.validate_session(info.session_id).await?;
    let actual = SessionInfo::from_session(&session);
    if actual.session_id != info.session_id
        || actual.kind != info.kind
        || actual.subject_id != info.subject_id
    {
        return Err(AppError::Auth(AuthError::SessionNotFound));
    }
    Ok(session)
}

fn url_encode(input: &str) -> String {
    input
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

fn url_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let Ok(value) = u8::from_str_radix(
                std::str::from_utf8(&bytes[index + 1..index + 3]).unwrap_or(""),
                16,
            ) {
                out.push(value);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_session_cookie() {
        let info = SessionInfo {
            session_id: 7,
            kind: SessionKind::User,
            subject_id: 3,
        };
        let encoded = url_encode(&serde_json::to_string(&info).unwrap());
        let decoded: SessionInfo = serde_json::from_str(&url_decode(&encoded)).unwrap();
        assert_eq!(info, decoded);
    }
}
