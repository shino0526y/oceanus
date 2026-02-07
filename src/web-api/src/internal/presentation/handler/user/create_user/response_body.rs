use crate::internal::domain::entity::User;
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserResponseBody {
    pub id: String,
    pub name: String,
    pub role: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for CreateUserResponseBody {
    fn from(user: User) -> Self {
        Self {
            id: user.id().value().into(),
            name: user.name().value().into(),
            role: user.role().as_i16(),
            created_at: *user.created_at(),
            updated_at: *user.updated_at(),
        }
    }
}
