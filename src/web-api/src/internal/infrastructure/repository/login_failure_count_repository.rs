use crate::internal::domain::{
    entity::LoginFailureCount, error::RepositoryError, repository::LoginFailureCountRepository,
};
use chrono::{DateTime, Utc};
use sqlx::{FromRow, Pool, Postgres};
use uuid::Uuid;

#[derive(FromRow)]
struct LoginFailureCountRecord {
    user_uuid: Uuid,
    failure_count: i16,
    last_failure_at: Option<DateTime<Utc>>,
}

impl From<LoginFailureCountRecord> for LoginFailureCount {
    fn from(record: LoginFailureCountRecord) -> Self {
        LoginFailureCount::construct(
            record.user_uuid,
            record.failure_count,
            record.last_failure_at,
        )
    }
}

pub struct PostgresLoginFailureCountRepository {
    pool: Pool<Postgres>,
}

impl PostgresLoginFailureCountRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl LoginFailureCountRepository for PostgresLoginFailureCountRepository {
    async fn find_by_user_uuid(
        &self,
        user_uuid: &Uuid,
    ) -> Result<Option<LoginFailureCount>, RepositoryError> {
        let record = sqlx::query_as::<_, LoginFailureCountRecord>(
            "SELECT user_uuid, failure_count, last_failure_at
             FROM login_failure_counts
             WHERE user_uuid = $1",
        )
        .bind(user_uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Other {
            message: format!("データベース処理でエラーが発生しました: {e}"),
        })?;

        Ok(record.map(|r| r.into()))
    }

    async fn save(&self, login_failure_count: &LoginFailureCount) -> Result<(), RepositoryError> {
        sqlx::query(
            "INSERT INTO login_failure_counts (user_uuid, failure_count, last_failure_at)
             VALUES ($1, $2, $3)
             ON CONFLICT (user_uuid) DO UPDATE
             SET failure_count = $2, last_failure_at = $3",
        )
        .bind(login_failure_count.user_uuid())
        .bind(login_failure_count.failure_count())
        .bind(login_failure_count.last_failure_at())
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Other {
            message: format!("データベース処理でエラーが発生しました: {e}"),
        })?;

        Ok(())
    }

    async fn delete(&self, user_uuid: &Uuid) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM login_failure_counts WHERE user_uuid = $1")
            .bind(user_uuid)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::Other {
                message: format!("データベース処理でエラーが発生しました: {e}"),
            })?;

        Ok(())
    }
}

#[cfg(test)]
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[cfg(test)]
pub struct TestLoginFailureCountRepository {
    inner: Arc<RwLock<HashMap<Uuid, LoginFailureCount>>>,
}

#[cfg(test)]
impl TestLoginFailureCountRepository {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[cfg(test)]
#[async_trait::async_trait]
impl LoginFailureCountRepository for TestLoginFailureCountRepository {
    async fn find_by_user_uuid(
        &self,
        user_uuid: &Uuid,
    ) -> Result<Option<LoginFailureCount>, RepositoryError> {
        Ok(self.inner.read().unwrap().get(user_uuid).cloned())
    }

    async fn save(&self, login_failure_count: &LoginFailureCount) -> Result<(), RepositoryError> {
        self.inner.write().unwrap().insert(
            *login_failure_count.user_uuid(),
            login_failure_count.clone(),
        );
        Ok(())
    }

    async fn delete(&self, user_uuid: &Uuid) -> Result<(), RepositoryError> {
        self.inner.write().unwrap().remove(user_uuid);
        Ok(())
    }
}
