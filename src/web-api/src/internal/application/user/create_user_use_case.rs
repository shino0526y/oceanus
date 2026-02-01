use crate::internal::domain::{
    entity::User,
    error::RepositoryError,
    repository::UserRepository,
    value_object::{Id, Role, UserName},
};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub struct CreateUserUseCase {
    repository: Arc<dyn UserRepository>,
}

pub struct CreateUserCommand {
    pub id: Id,
    pub name: UserName,
    pub role: Role,
    pub password: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

impl CreateUserUseCase {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, command: CreateUserCommand) -> Result<User, CreateUserError> {
        // パスワードの空文字チェック
        if command.password.is_empty() {
            return Err(CreateUserError::EmptyPassword);
        }

        // 作成者の権限を確認: 情シスは管理者の作成を行えない
        match self.repository.find_by_uuid(&command.created_by).await {
            Ok(Some(actor)) => {
                if actor.role() == Role::ItStaff && command.role == Role::Admin {
                    return Err(CreateUserError::Forbidden);
                }
            }
            Ok(None) => {
                return Err(CreateUserError::Repository(RepositoryError::NotFound {
                    resource: "ユーザー".to_string(),
                    key: command.created_by.to_string(),
                }));
            }
            Err(e) => return Err(CreateUserError::Repository(e)),
        }

        // パスワードのハッシュ化(argon2id)
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(command.password.as_bytes(), &salt)
            .map_err(|e| CreateUserError::PasswordHashError(e.to_string()))?
            .to_string();

        let user = User::create(
            command.id,
            command.name,
            command.role,
            password_hash,
            command.created_by,
            command.created_at,
        );

        self.repository
            .add(&user)
            .await
            .map_err(CreateUserError::Repository)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateUserError {
    #[error("パスワードは1文字以上で入力してください")]
    EmptyPassword,
    #[error("権限がありません")]
    Forbidden,
    #[error("パスワードのハッシュ化に失敗しました: {0}")]
    PasswordHashError(String),
    #[error("{0}")]
    Repository(#[from] RepositoryError),
}
