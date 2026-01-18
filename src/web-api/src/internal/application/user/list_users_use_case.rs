use crate::internal::domain::{entity::User, error::RepositoryError, repository::UserRepository};
use std::sync::Arc;

pub struct ListUsersUseCase {
    repository: Arc<dyn UserRepository>,
}

impl ListUsersUseCase {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<Vec<User>, RepositoryError> {
        self.repository.find_all().await
    }
}
