use crate::internal::domain::entity::User;
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct ListUsersOutputElement {
    pub id: String,
    pub name: String,
    pub role: u8,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for ListUsersOutputElement {
    fn from(user: User) -> Self {
        let role = user.role();
        Self {
            id: user.id().to_string(),
            name: user.name().to_string(),
            role: role.as_u8(),
            created_at: *user.created_at(),
            updated_at: *user.updated_at(),
        }
    }
}
