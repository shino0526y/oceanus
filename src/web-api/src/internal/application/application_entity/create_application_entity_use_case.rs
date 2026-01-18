mod create_application_entity_command;

pub use create_application_entity_command::CreateApplicationEntityCommand;

use crate::internal::domain::{
    entity::ApplicationEntity, error::RepositoryError, repository::ApplicationEntityRepository,
};
use chrono::Utc;
use std::sync::Arc;

pub struct CreateApplicationEntityUseCase {
    repository: Arc<dyn ApplicationEntityRepository>,
}

impl CreateApplicationEntityUseCase {
    pub fn new(repository: Arc<dyn ApplicationEntityRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        command: CreateApplicationEntityCommand,
    ) -> Result<ApplicationEntity, RepositoryError> {
        let entity = ApplicationEntity::new(
            command.title,
            command.host,
            command.port,
            command.comment,
            Utc::now(),
            Utc::now(),
        );

        self.repository.add(&entity).await
    }
}
