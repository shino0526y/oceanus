use crate::internal::application::{
    auth::{
        AuthenticateUserUseCase,
        authenticate_user_use_case::{AuthenticateUserCommand, AuthenticationError},
    },
    session::CreateSessionUseCase,
};
use crate::internal::domain::value_object::Id;
use std::sync::Arc;

pub struct LoginUseCase {
    authenticate_user_use_case: Arc<AuthenticateUserUseCase>,
    create_session_use_case: Arc<CreateSessionUseCase>,
}

pub struct LoginCommand {
    pub user_id: Id,
    pub password: String,
}

impl LoginUseCase {
    pub fn new(
        authenticate_user_use_case: Arc<AuthenticateUserUseCase>,
        create_session_use_case: Arc<CreateSessionUseCase>,
    ) -> Self {
        Self {
            authenticate_user_use_case,
            create_session_use_case,
        }
    }

    pub async fn execute(
        &self,
        command: LoginCommand,
    ) -> Result<(String, String), AuthenticationError> {
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

        Ok((session_id, csrf_token))
    }
}
