use crate::internal::domain::value_object::Port;
use chrono::{DateTime, Utc};
use dicom_lib::core::value::value_representations::ae::AeValue;

pub struct ApplicationEntity {
    title: AeValue,
    host: String,
    port: Port,
    comment: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl ApplicationEntity {
    pub fn new(
        title: AeValue,
        host: impl Into<String>,
        port: Port,
        comment: impl Into<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            title,
            host: host.into(),
            port,
            comment: comment.into(),
            created_at,
            updated_at,
        }
    }

    pub fn title(&self) -> &AeValue {
        &self.title
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> &Port {
        &self.port
    }

    pub fn comment(&self) -> &str {
        &self.comment
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}
