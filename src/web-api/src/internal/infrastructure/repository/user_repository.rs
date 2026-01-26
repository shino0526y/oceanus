use crate::internal::domain::{
    entity::User,
    error::RepositoryError,
    repository::UserRepository,
    value_object::{Id, Role},
};
use chrono::{DateTime, Utc};
use sqlx::{FromRow, Pool, Postgres};
use uuid::Uuid;

#[derive(FromRow)]
struct UserRecord {
    uuid: Uuid,
    id: String,
    name: String,
    role: i16,
    password_hash: String,
    created_by: Uuid,
    created_at: DateTime<Utc>,
    updated_by: Uuid,
    updated_at: DateTime<Utc>,
}

impl TryFrom<UserRecord> for User {
    type Error = String;

    fn try_from(record: UserRecord) -> Result<Self, Self::Error> {
        let user_id = Id::new(record.id)?;
        let role = Role::from_i16(record.role)?;
        Ok(User::construct(
            record.uuid,
            user_id,
            record.name,
            role,
            record.password_hash,
            record.created_by,
            record.created_at,
            record.updated_by,
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
            "SELECT uuid, id, name, role, password_hash, created_by, created_at, updated_by, updated_at
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

    async fn find_by_uuid(&self, uuid: &Uuid) -> Result<Option<User>, RepositoryError> {
        let record = sqlx::query_as::<_, UserRecord>(
            "SELECT uuid, id, name, role, password_hash, created_by, created_at, updated_by, updated_at
             FROM users
             WHERE uuid = $1",
        )
        .bind(uuid)
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

    async fn find_by_id(&self, id: &Id) -> Result<Option<User>, RepositoryError> {
        let record = sqlx::query_as::<_, UserRecord>(
            "SELECT uuid, id, name, role, password_hash, created_by, created_at, updated_by, updated_at
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

    async fn add(&self, user: &User) -> Result<User, RepositoryError> {
        let record = sqlx::query_as::<_, UserRecord>(
            "INSERT INTO users (uuid, id, name, role, password_hash, created_by, created_at, updated_by, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             RETURNING uuid, id, name, role, password_hash, created_by, created_at, updated_by, updated_at",
        )
        .bind(user.uuid())
        .bind(user.id().value())
        .bind(user.name())
        .bind(user.role().as_i16())
        .bind(user.password_hash())
        .bind(user.created_by())
        .bind(user.created_at())
        .bind(user.updated_by())
        .bind(user.updated_at())
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
        let record = sqlx::query_as::<_, UserRecord>(
            "UPDATE users
             SET id = $1, name = $2, role = $3, password_hash = $4, updated_by = $5, updated_at = $6
             WHERE id = $7
             RETURNING uuid, id, name, role, password_hash, created_by, created_at, updated_by, updated_at",
        )
        .bind(user.id().value())
        .bind(user.name())
        .bind(user.role().as_i16())
        .bind(user.password_hash())
        .bind(user.updated_by())
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
                    key: user.id().value().to_string(),
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

    async fn delete(
        &self,
        id: &Id,
        deleted_by: &Uuid,
        deleted_at: &DateTime<Utc>,
    ) -> Result<(), RepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| RepositoryError::Other {
                message: format!("トランザクションの開始に失敗しました: {e}"),
            })?;

        // 削除済みテーブルにINSERT
        let rows_affected = sqlx::query(
            "INSERT INTO users_deleted (uuid, id, name, role, password_hash, created_by, created_at, updated_by, updated_at, deleted_by, deleted_at)
             SELECT uuid, id, name, role, password_hash, created_by, created_at, updated_by, updated_at, $1, $2
             FROM users
             WHERE id = $3",
        )
        .bind(deleted_by)
        .bind(deleted_at)
        .bind(id.value())
        .execute(&mut *tx)
        .await
        .map_err(|e| RepositoryError::Other {
            message: format!("データベース処理でエラーが発生しました: {e}"),
        })?
        .rows_affected();

        if rows_affected == 0 {
            return Err(RepositoryError::NotFound {
                resource: "ユーザー".to_string(),
                key: id.value().to_string(),
            });
        }

        // 元テーブルからDELETE（login_failure_counts も CASCADE で削除される）
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id.value())
            .execute(&mut *tx)
            .await
            .map_err(|e| RepositoryError::Other {
                message: format!("データベース処理でエラーが発生しました: {e}"),
            })?;

        tx.commit().await.map_err(|e| RepositoryError::Other {
            message: format!("トランザクションのコミットに失敗しました: {e}"),
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
pub struct TestUserRepository {
    inner: Arc<RwLock<HashMap<Uuid, User>>>,
}

#[cfg(test)]
impl TestUserRepository {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn find_uuid_by_id(&self, id: &Id) -> Option<Uuid> {
        self.inner
            .read()
            .unwrap()
            .values()
            .find(|e| e.id() == id)
            .cloned()
            .map(|e| *e.uuid())
    }
}

#[cfg(test)]
#[async_trait::async_trait]
impl UserRepository for TestUserRepository {
    async fn find_all(&self) -> Result<Vec<User>, RepositoryError> {
        let mut entities = self
            .inner
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect::<Vec<User>>();
        entities.sort_by(|a, b| b.created_at().cmp(&a.created_at()));
        Ok(entities)
    }

    async fn find_by_uuid(&self, uuid: &Uuid) -> Result<Option<User>, RepositoryError> {
        Ok(self.inner.read().unwrap().get(uuid).cloned())
    }

    async fn find_by_id(&self, id: &Id) -> Result<Option<User>, RepositoryError> {
        Ok(self
            .inner
            .read()
            .unwrap()
            .values()
            .find(|e| e.id() == id)
            .cloned())
    }

    async fn add(&self, user: &User) -> Result<User, RepositoryError> {
        let exists = self.find_by_id(user.id()).await?.is_some();
        if exists {
            return Err(RepositoryError::AlreadyExists {
                resource: "ユーザー".to_string(),
                key: user.id().value().to_string(),
            });
        }

        self.inner
            .write()
            .unwrap()
            .insert(*user.uuid(), user.clone());
        Ok(user.clone())
    }

    async fn update(&self, old_id: &Id, user: &User) -> Result<User, RepositoryError> {
        let uuid = self.find_uuid_by_id(old_id).unwrap();
        self.inner.write().unwrap().insert(uuid, user.clone());
        Ok(user.clone())
    }

    async fn delete(
        &self,
        id: &Id,
        _deleted_by: &Uuid,
        _deleted_at: &DateTime<Utc>,
    ) -> Result<(), RepositoryError> {
        let uuid = self.find_uuid_by_id(id).unwrap();
        self.inner.write().unwrap().remove(&uuid);
        Ok(())
    }
}
