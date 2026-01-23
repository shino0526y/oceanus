use crate::internal::domain::{error::RepositoryError, repository::ApplicationEntityRepository};
use chrono::{DateTime, Utc};
use dicom_lib::core::value::value_representations::ae::AeValue;
use std::sync::Arc;
use uuid::Uuid;

pub struct DeleteApplicationEntityUseCase {
    repository: Arc<dyn ApplicationEntityRepository>,
}

pub struct DeleteApplicationEntityCommand {
    pub title: AeValue,
    pub deleted_by: Uuid,
    pub deleted_at: DateTime<Utc>,
}

impl DeleteApplicationEntityUseCase {
    pub fn new(repository: Arc<dyn ApplicationEntityRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        command: DeleteApplicationEntityCommand,
    ) -> Result<(), RepositoryError> {
        self.repository
            .delete(&command.title, &command.deleted_by, &command.deleted_at)
            .await
    }
}
