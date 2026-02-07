use crate::internal::domain::{entity::LoginFailureCount, error::RepositoryError};
use uuid::Uuid;

#[async_trait::async_trait]
pub trait LoginFailureCountRepository: Send + Sync {
    /// すべてのログイン失敗情報を取得する
    async fn find_all(&self) -> Result<Vec<LoginFailureCount>, RepositoryError>;

    /// ユーザーUUIDでログイン失敗情報を取得する（存在しない場合はNone）
    async fn find_by_user_uuid(
        &self,
        user_uuid: &Uuid,
    ) -> Result<Option<LoginFailureCount>, RepositoryError>;

    /// ログイン失敗情報を保存する（UPSERT: 存在しない場合は作成、存在する場合は更新）
    async fn save(&self, login_failure_count: &LoginFailureCount) -> Result<(), RepositoryError>;

    /// ログイン失敗情報を削除する
    async fn delete(&self, user_uuid: &Uuid) -> Result<(), RepositoryError>;
}
