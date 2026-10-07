use crate::auth::UserSession;

pub struct User {
    email: String,
    session: UserSession,
    organizationID: u32,
    userID: UserID,
    paymentDetails: UserPayDetails,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserID {
    id: u32,
}

impl UserID {
    pub fn new(id: u32) -> Self {
        Self { id }
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}

pub struct UserPayDetails {}
