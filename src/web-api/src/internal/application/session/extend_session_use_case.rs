use crate::internal::domain::repository::SessionRepository;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug)]
pub enum CsrfValidationError {
    InvalidToken,
}

pub struct ExtendSessionUseCase {
    session_repository: Arc<dyn SessionRepository>,
}

impl ExtendSessionUseCase {
    pub fn new(session_repository: Arc<dyn SessionRepository>) -> Self {
        Self { session_repository }
    }

    /// セッションの有効期限を延長し、ユーザーUUIDを返す
    pub async fn execute(&self, session_id: &str) -> Option<Uuid> {
        // セッションを取得（期限切れは自動削除される）
        let mut session = (self.session_repository.find_by_session_id(session_id).await)?;

        let user_uuid = *session.user_uuid();
        session.extend();
        self.session_repository.save(session).await;

        Some(user_uuid)
    }

    /// セッションの有効期限を延長（CSRF検証付き）し、ユーザーUUIDを返す
    pub async fn execute_with_csrf_validation(
        &self,
        session_id: &str,
        csrf_token: &str,
    ) -> Result<Option<Uuid>, CsrfValidationError> {
        // セッションを取得（期限切れは自動削除される）
        let Some(mut session) = self.session_repository.find_by_session_id(session_id).await else {
            return Ok(None);
        };

        // CSRFトークンを検証
        if !session.validate_csrf_token(csrf_token) {
            return Err(CsrfValidationError::InvalidToken);
        }

        let user_uuid = *session.user_uuid();
        session.extend();
        self.session_repository.save(session).await;

        Ok(Some(user_uuid))
    }
}
