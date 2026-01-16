use crate::internal::domain::entity::ApplicationEntity;
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListApplicationEntitiesOutputElement {
    pub title: String,
    pub host: String,
    pub port: u16,
    pub comment: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ApplicationEntity> for ListApplicationEntitiesOutputElement {
    fn from(entity: ApplicationEntity) -> Self {
        Self {
            title: entity.title().to_string(),
            host: entity.host().to_string(),
            port: entity.port().value(),
            comment: entity.comment().to_string(),
            created_at: *entity.created_at(),
            updated_at: *entity.updated_at(),
        }
    }
}
