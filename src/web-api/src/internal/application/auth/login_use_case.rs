use crate::internal::application::{
    auth::{
        AuthenticateUserUseCase,
        authenticate_user_use_case::{AuthenticateUserCommand, AuthenticationError},
    },
    session::CreateSessionUseCase,
};
use crate::internal::domain::repository::UserRepository;
use crate::internal::domain::value_object::Id;
use std::sync::Arc;

pub struct LoginUseCase {
    authenticate_user_use_case: Arc<AuthenticateUserUseCase>,
    create_session_use_case: Arc<CreateSessionUseCase>,
    user_repository: Arc<dyn UserRepository>,
}

pub struct LoginCommand {
    pub user_id: Id,
    pub password: String,
}

impl LoginUseCase {
    pub fn new(
        authenticate_user_use_case: Arc<AuthenticateUserUseCase>,
        create_session_use_case: Arc<CreateSessionUseCase>,
        user_repository: Arc<dyn UserRepository>,
    ) -> Self {
        Self {
            authenticate_user_use_case,
            create_session_use_case,
            user_repository,
        }
    }

    pub async fn execute(
        &self,
        command: LoginCommand,
    ) -> Result<(String, String, i16), AuthenticationError> {
        // 認証
        let authenticate_user_command = AuthenticateUserCommand {
            user_id: &command.user_id,
            password: &command.password,
        };
        let user_uuid = self
            .authenticate_user_use_case
            .execute(authenticate_user_command)
            .await?;

        // セッション確立
        let (session_id, csrf_token) = self.create_session_use_case.execute(user_uuid).await;

        // ユーザー情報からロールを取得
        let user = self
            .user_repository
            .find_by_uuid(&user_uuid)
            .await
            .map_err(|_e| AuthenticationError::Other {
                message: "リポジトリエラー".to_string(),
            })?
            .ok_or(AuthenticationError::Other {
                message: "ユーザーが見つかりません".to_string(),
            })?;

        let role_i16 = user.role().as_i16();

        Ok((session_id, csrf_token, role_i16))
    }
}
