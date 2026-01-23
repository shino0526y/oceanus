use crate::internal::domain::{
    entity::User,
    error::RepositoryError,
    repository::UserRepository,
    value_object::{Id, Role},
};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub struct UpdateUserUseCase {
    repository: Arc<dyn UserRepository>,
}

pub struct UpdateUserCommand {
    pub old_id: Id,

    pub id: Id,
    pub name: String,
    pub role: Role,
    pub password: Option<String>,
    pub updated_by: Uuid,
    pub updated_at: DateTime<Utc>,
}

impl UpdateUserUseCase {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, command: UpdateUserCommand) -> Result<User, UpdateUserError> {
        let mut entity = self
            .repository
            .find_by_id(&command.old_id)
            .await?
            .ok_or_else(|| {
                UpdateUserError::Repository(RepositoryError::NotFound {
                    resource: "ユーザー".to_string(),
                    key: command.old_id.value().to_string(),
                })
            })?;

        // パスワードが指定された場合のみハッシュ化、それ以外は既存のハッシュを維持
        let password_hash = match command.password {
            Some(password) => {
                // 空文字列は許可しない
                if password.is_empty() {
                    return Err(UpdateUserError::EmptyPassword);
                }
                let salt = SaltString::generate(&mut OsRng);
                let argon2 = Argon2::default();
                argon2
                    .hash_password(password.as_bytes(), &salt)
                    .map_err(|e| UpdateUserError::PasswordHashError(e.to_string()))?
                    .to_string()
            }
            None => entity.password_hash().to_string(),
        };

        // エンティティを変更し、変更があれば保存
        let is_changed = entity.update(
            command.id,
            command.name,
            command.role,
            password_hash,
            command.updated_by,
            command.updated_at,
        );
        if !is_changed {
            return Ok(entity);
        }
        self.repository
            .update(&command.old_id, &entity)
            .await
            .map_err(UpdateUserError::Repository)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateUserError {
    #[error("パスワードは1文字以上で入力してください")]
    EmptyPassword,
    #[error("パスワードのハッシュ化に失敗しました: {0}")]
    PasswordHashError(String),
    #[error("{0}")]
    Repository(#[from] RepositoryError),
}
