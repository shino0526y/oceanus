use crate::internal::domain::{
    entity::User,
    error::RepositoryError,
    repository::UserRepository,
    value_object::{Id, Role},
};
use chrono::{DateTime, Utc};
use sqlx::{FromRow, Pool, Postgres};

#[derive(FromRow)]
struct UserRecord {
    id: String,
    name: String,
    role: i16,
    password_hash: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<UserRecord> for User {
    type Error = String;

    fn try_from(record: UserRecord) -> Result<Self, Self::Error> {
        let user_id = Id::new(record.id)?;
        let role = Role::from_i16(record.role)?;
        Ok(User::new(
            user_id,
            record.name,
            role,
            record.password_hash,
            record.created_at,
            record.updated_at,
        ))
    }
}

pub struct PostgresUserRepository {
    pool: Pool<Postgres>,
}

impl PostgresUserRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_all(&self) -> Result<Vec<User>, RepositoryError> {
        let records = sqlx::query_as::<_, UserRecord>(
            "SELECT id, name, role, password_hash, created_at, updated_at
             FROM users
             ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Other {
            message: format!("データベース処理でエラーが発生しました: {e}"),
        })?;

        let entities = records
            .into_iter()
            .map(|r| {
                r.try_into()
                    .expect("DBレコードからエンティティへの変換は成功するはず")
            })
            .collect::<Vec<_>>();
        Ok(entities)
    }
}
