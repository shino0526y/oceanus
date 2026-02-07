use crate::internal::domain::{
    entity::ApplicationEntity,
    error::RepositoryError,
    repository::ApplicationEntityRepository,
    value_object::{HostName, Port},
};
use chrono::{DateTime, Utc};
use dicom_lib::core::value::value_representations::ae::AeValue;
use std::sync::Arc;
use uuid::Uuid;

pub struct CreateApplicationEntityUseCase {
    repository: Arc<dyn ApplicationEntityRepository>,
}

pub struct CreateApplicationEntityCommand {
    pub title: AeValue,
    pub host: HostName,
    pub port: Port,
    pub comment: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

impl CreateApplicationEntityUseCase {
    pub fn new(repository: Arc<dyn ApplicationEntityRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        command: CreateApplicationEntityCommand,
    ) -> Result<ApplicationEntity, RepositoryError> {
        let entity = ApplicationEntity::create(
            command.title,
            command.host,
            command.port,
            command.comment,
            command.created_by,
            command.created_at,
        );

        self.repository.add(&entity).await
    }
}
