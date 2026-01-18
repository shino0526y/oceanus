use crate::internal::domain::repository::SessionRepository;
use std::sync::Arc;

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

    /// セッションの有効期限を延長
    pub async fn execute(&self, session_id: &str) -> bool {
        // セッションを取得（期限切れは自動削除される）
        let Some(mut session) = self.session_repository.find_by_session_id(session_id).await else {
            return false;
        };

        session.extend();

        self.session_repository.save(session).await;

        true
    }

    /// セッションの有効期限を延長（CSRF検証付き）
    pub async fn execute_with_csrf_validation(
        &self,
        session_id: &str,
        csrf_token: &str,
    ) -> Result<bool, CsrfValidationError> {
        // セッションを取得（期限切れは自動削除される）
        let Some(mut session) = self.session_repository.find_by_session_id(session_id).await else {
            return Ok(false);
        };

        // CSRFトークンを検証
        if !session.validate_csrf_token(csrf_token) {
            return Err(CsrfValidationError::InvalidToken);
        }

        session.extend();

        self.session_repository.save(session).await;

        Ok(true)
    }
}
