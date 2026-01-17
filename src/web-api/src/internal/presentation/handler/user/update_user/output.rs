use crate::internal::domain::entity::User;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserOutput {
    pub id: String,
    pub name: String,
    pub role: i16,
    pub created_at: String,
    pub updated_at: String,
}

impl From<User> for UpdateUserOutput {
    fn from(user: User) -> Self {
        Self {
            id: user.id().value().to_string(),
            name: user.name().to_string(),
            role: user.role().as_i16(),
            created_at: user.created_at().to_rfc3339(),
            updated_at: user.updated_at().to_rfc3339(),
        }
    }
}
