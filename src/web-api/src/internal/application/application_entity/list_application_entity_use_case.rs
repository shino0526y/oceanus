use crate::internal::domain::{
    entity::ApplicationEntity, error::RepositoryError, repository::ApplicationEntityRepository,
};
use std::sync::Arc;

pub struct ListApplicationEntitiesUseCase {
    repository: Arc<dyn ApplicationEntityRepository>,
}

impl ListApplicationEntitiesUseCase {
    pub fn new(repository: Arc<dyn ApplicationEntityRepository>) -> Self {
        Self { repository }
    }

    pub async fn list_application_entities(
        &self,
    ) -> Result<Vec<ApplicationEntity>, RepositoryError> {
        self.repository.find_all().await
    }
}
