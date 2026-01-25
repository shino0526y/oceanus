use crate::internal::domain::entity::Session;

#[async_trait::async_trait]
pub trait SessionRepository: Send + Sync {
    /// セッションを保存（新規作成または更新）
    async fn save(&self, session: Session);

    /// セッションIDからセッション情報を取得
    async fn find_by_session_id(&self, session_id: &str) -> Option<Session>;

    /// セッションを削除
    async fn delete(&self, session_id: &str);

    /// 期限切れセッションを削除（メモリリーク対策）
    async fn cleanup_expired_sessions(&self);
}
