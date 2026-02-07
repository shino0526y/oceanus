use crate::internal::domain::{
    error::RepositoryError,
    repository::{LoginFailureCountRepository, SessionRepository, UserRepository},
    value_object::{Id, Role},
};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub struct DeleteUserUseCase {
    user_repository: Arc<dyn UserRepository>,
    login_failure_count_repository: Arc<dyn LoginFailureCountRepository>,
    session_repository: Arc<dyn SessionRepository>,
}

pub struct DeleteUserCommand {
    pub id: Id,
    pub deleted_by: Uuid,
    pub deleted_at: DateTime<Utc>,
}

impl DeleteUserUseCase {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        login_failure_count_repository: Arc<dyn LoginFailureCountRepository>,
        session_repository: Arc<dyn SessionRepository>,
    ) -> Self {
        Self {
            user_repository,
            login_failure_count_repository,
            session_repository,
        }
    }

    pub async fn execute(&self, command: DeleteUserCommand) -> Result<(), DeleteUserError> {
        // 削除対象ユーザーを取得
        let target_user = self
            .user_repository
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

        // 削除者の権限を確認: 情シスは管理者を削除できない
        match self.user_repository.find_by_uuid(&command.deleted_by).await {
            Ok(Some(actor)) => {
                if actor.role() == Role::ItStaff && target_user.role() == Role::Admin {
                    return Err(DeleteUserError::Forbidden);
                }
            }
            Ok(None) => {
                return Err(DeleteUserError::Repository(RepositoryError::NotFound {
                    resource: "ユーザー".to_string(),
                    key: command.deleted_by.to_string(),
                }));
            }
            Err(e) => return Err(DeleteUserError::Repository(e)),
        }

        // ログイン失敗情報を明示的に削除（CASCADEに依存しない）
        self.login_failure_count_repository
            .delete(target_user.uuid())
            .await?;

        // 関連セッションを削除
        self.session_repository
            .delete_by_user_uuid(target_user.uuid())
            .await;

        // ユーザーを削除
        self.user_repository
            .delete(&command.id, &command.deleted_by, &command.deleted_at)
            .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteUserError {
    #[error("自分自身を削除することはできません")]
    CannotDeleteSelf,
    #[error("権限がありません")]
    Forbidden,
    #[error("{0}")]
    Repository(#[from] RepositoryError),
}
