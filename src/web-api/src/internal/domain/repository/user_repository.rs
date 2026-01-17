use crate::internal::domain::{entity::User, error::RepositoryError};

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<User>, RepositoryError>;
    async fn add(&self, user: User) -> Result<User, RepositoryError>;
}
