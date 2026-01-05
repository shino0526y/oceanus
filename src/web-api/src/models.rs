use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApplicationEntity {
    pub title: String,
    pub host: String,
    pub port: i32,
    pub comment: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApplicationEntity {
    pub title: String,
    pub host: String,
    pub port: i32,
    pub comment: String,
}
