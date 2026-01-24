use crate::internal::domain::{
    entity::LoginFailureCount,
    error::RepositoryError,
    repository::{LoginFailureCountRepository, UserRepository},
    value_object::Id,
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use chrono::Utc;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum AuthenticationError {
    #[error("ユーザーIDもしくはパスワードが間違っています")]
    InvalidCredentials,

    #[error("ロックされています。管理者にお問い合わせください。")]
    Locked,

    #[error("認証に失敗しました: {message}")]
    Other { message: String },
}

pub struct AuthenticateUserUseCase {
    user_repository: Arc<dyn UserRepository>,
    login_failure_count_repository: Arc<dyn LoginFailureCountRepository>,
}

pub struct AuthenticateUserCommand<'a> {
    pub user_id: &'a Id,
    pub password: &'a str,
}

impl AuthenticateUserUseCase {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        login_failure_count_repository: Arc<dyn LoginFailureCountRepository>,
    ) -> Self {
        Self {
            user_repository,
            login_failure_count_repository,
        }
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

        // ログイン失敗情報を取得（存在しない場合は新規作成）
        let mut login_failure_count = self
            .login_failure_count_repository
            .find_by_user_uuid(user.uuid())
            .await
            .map_err(|e| AuthenticationError::Other {
                message: format!("リポジトリエラー: {e}"),
            })?
            .unwrap_or_else(|| LoginFailureCount::new(*user.uuid()));

        // ロックチェック
        if login_failure_count.is_locked() {
            return Err(AuthenticationError::Locked);
        }

        // パスワード検証
        let parsed_hash =
            PasswordHash::new(user.password_hash()).map_err(|e| AuthenticationError::Other {
                message: format!("パスワードハッシュの解析に失敗しました: {e}"),
            })?;

        let password_valid = Argon2::default()
            .verify_password(command.password.as_bytes(), &parsed_hash)
            .is_ok();

        if !password_valid {
            // 認証失敗: ログイン失敗回数をインクリメント
            login_failure_count.increment(Utc::now());
            // エラーは無視（ログイン失敗回数の保存に失敗しても認証エラーを返す）
            let _ = self
                .login_failure_count_repository
                .save(&login_failure_count)
                .await;
            return Err(AuthenticationError::InvalidCredentials);
        }

        // 認証成功: ログイン失敗情報を削除（リセット）
        if login_failure_count.failure_count() > 0 {
            // エラーは無視（リセットに失敗しても認証は成功とする）
            let _ = self
                .login_failure_count_repository
                .delete(user.uuid())
                .await;
        }

        // 認証成功、ユーザーUUIDを返す
        Ok(*user.uuid())
    }
}
