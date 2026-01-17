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

    async fn find_by_id(&self, id: &Id) -> Result<Option<User>, RepositoryError> {
        let record = sqlx::query_as::<_, UserRecord>(
            "SELECT id, name, role, password_hash, created_at, updated_at
             FROM users
             WHERE id = $1",
        )
        .bind(id.value())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Other {
            message: format!("データベース処理でエラーが発生しました: {e}"),
        })?;

        match record {
            Some(record) => {
                let entity = record
                    .try_into()
                    .expect("DBレコードからエンティティへの変換は成功するはず");
                Ok(Some(entity))
            }
            None => Ok(None),
        }
    }

    async fn add(&self, user: User) -> Result<User, RepositoryError> {
        let record = sqlx::query_as::<_, UserRecord>(
            "INSERT INTO users (id, name, role, password_hash)
             VALUES ($1, $2, $3, $4)
             RETURNING id, name, role, password_hash, created_at, updated_at",
        )
        .bind(user.id().value())
        .bind(user.name())
        .bind(user.role().as_i16())
        .bind(user.password_hash())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error()
                && db_err.is_unique_violation()
            {
                return RepositoryError::AlreadyExists {
                    resource: "ユーザー".to_string(),
                    key: user.id().value().to_string(),
                };
            }
            RepositoryError::Other {
                message: format!("データベース処理でエラーが発生しました: {e}"),
            }
        })?;

        let entity = record
            .try_into()
            .expect("DBレコードからエンティティへの変換は成功するはず");
        Ok(entity)
    }

    async fn update(&self, old_id: &Id, user: &User) -> Result<User, RepositoryError> {
        let new_id = user.id();
        let record = sqlx::query_as::<_, UserRecord>(
            "UPDATE users
             SET id = $1, name = $2, role = $3, password_hash = $4, updated_at = $5
             WHERE id = $6
             RETURNING id, name, role, password_hash, created_at, updated_at",
        )
        .bind(new_id.value())
        .bind(user.name())
        .bind(user.role().as_i16())
        .bind(user.password_hash())
        .bind(user.updated_at())
        .bind(old_id.value())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error()
                && db_err.is_unique_violation()
            {
                return RepositoryError::AlreadyExists {
                    resource: "ユーザー".to_string(),
                    key: new_id.value().to_string(),
                };
            }
            RepositoryError::Other {
                message: format!("データベース処理でエラーが発生しました: {e}"),
            }
        })?;

        match record {
            Some(record) => {
                let entity = record
                    .try_into()
                    .expect("DBレコードからエンティティへの変換は成功するはず");
                Ok(entity)
            }
            None => Err(RepositoryError::NotFound {
                resource: "ユーザー".to_string(),
                key: old_id.value().to_string(),
            }),
        }
    }
}
