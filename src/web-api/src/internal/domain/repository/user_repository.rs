use crate::internal::domain::{entity::User, error::RepositoryError, value_object::Id};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<User>, RepositoryError>;

    async fn find_by_id(&self, id: &Id) -> Result<Option<User>, RepositoryError>;

    async fn add(&self, user: &User) -> Result<User, RepositoryError>;

    async fn update(&self, old_id: &Id, user: &User) -> Result<User, RepositoryError>;

    async fn delete(
        &self,
        id: &Id,
        deleted_by: &Uuid,
        deleted_at: &DateTime<Utc>,
    ) -> Result<(), RepositoryError>;
}
