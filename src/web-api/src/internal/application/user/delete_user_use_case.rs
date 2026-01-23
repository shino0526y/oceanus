use crate::internal::domain::{
    error::RepositoryError, repository::UserRepository, value_object::Id,
};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub struct DeleteUserUseCase {
    repository: Arc<dyn UserRepository>,
}

pub struct DeleteUserCommand {
    pub id: Id,
    pub deleted_by: Uuid,
    pub deleted_at: DateTime<Utc>,
}

impl DeleteUserUseCase {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, command: DeleteUserCommand) -> Result<(), DeleteUserError> {
        // 削除対象ユーザーを取得
        let target_user = self
            .repository
            .find_by_id(&command.id)
            .await?
            .ok_or_else(|| {
                DeleteUserError::Repository(RepositoryError::NotFound {
                    resource: "ユーザー".to_string(),
                    key: command.id.value().to_string(),
                })
            })?;

        // 自分自身は削除できない
        if target_user.uuid() == &command.deleted_by {
            return Err(DeleteUserError::CannotDeleteSelf);
        }

        self.repository
            .delete(&command.id, &command.deleted_by, &command.deleted_at)
            .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteUserError {
    #[error("自分自身を削除することはできません")]
    CannotDeleteSelf,
    #[error("{0}")]
    Repository(#[from] RepositoryError),
}
