use crate::internal::domain::{
    error::RepositoryError, repository::UserRepository, value_object::Id,
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AuthenticationError {
    #[error("ユーザーIDもしくはパスワードが間違っています")]
    InvalidCredentials,

    #[error("認証に失敗しました: {message}")]
    Other { message: String },
}

pub struct AuthenticateUserUseCase {
    user_repository: Arc<dyn UserRepository>,
}

pub struct AuthenticateUserCommand<'a> {
    pub user_id: &'a Id,
    pub password: &'a str,
}

impl AuthenticateUserUseCase {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    /// ユーザーを認証し、認証成功時にユーザーUUIDを返す
    pub async fn execute(
        &self,
        command: AuthenticateUserCommand<'_>,
    ) -> Result<Uuid, AuthenticationError> {
        // ユーザー取得
        let user = self
            .user_repository
            .find_by_id(command.user_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound { .. } => AuthenticationError::InvalidCredentials,
                _ => AuthenticationError::Other {
                    message: format!("リポジトリエラー: {e}"),
                },
            })?
            .ok_or(AuthenticationError::InvalidCredentials)?;

        // パスワード検証
        let parsed_hash =
            PasswordHash::new(user.password_hash()).map_err(|e| AuthenticationError::Other {
                message: format!("パスワードハッシュの解析に失敗しました: {e}"),
            })?;

        Argon2::default()
            .verify_password(command.password.as_bytes(), &parsed_hash)
            .map_err(|_| AuthenticationError::InvalidCredentials)?;

        // 認証成功、ユーザーUUIDを返す
        Ok(*user.uuid())
    }
}
