use crate::internal::domain::repository::SessionRepository;
use std::sync::Arc;

pub struct DeleteSessionUseCase {
    session_repository: Arc<dyn SessionRepository>,
}

impl DeleteSessionUseCase {
    pub fn new(session_repository: Arc<dyn SessionRepository>) -> Self {
        Self { session_repository }
    }

    /// セッションを削除
    pub async fn execute(&self, session_id: &str) {
        self.session_repository
            .delete_by_session_id(session_id)
            .await;
    }
}
