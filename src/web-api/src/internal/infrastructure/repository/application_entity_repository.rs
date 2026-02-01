use crate::internal::domain::{
    entity::ApplicationEntity, error::RepositoryError, repository::ApplicationEntityRepository,
    value_object::Port,
};
use chrono::{DateTime, Utc};
use dicom_lib::core::value::value_representations::ae::AeValue;
use sqlx::{FromRow, Pool, Postgres};
use uuid::Uuid;

#[derive(FromRow)]
struct ApplicationEntityRecord {
    uuid: Uuid,
    title: String,
    host: String,
    port: i32,
    comment: String,
    created_by: Uuid,
    created_at: DateTime<Utc>,
    updated_by: Uuid,
    updated_at: DateTime<Utc>,
}

impl TryFrom<ApplicationEntityRecord> for ApplicationEntity {
    type Error = String;

    fn try_from(record: ApplicationEntityRecord) -> Result<Self, Self::Error> {
        let title = AeValue::from_string(&record.title)
            .map_err(|e| format!("AEタイトルが不正です: {e}"))?;
        let port = Port::from_i32(record.port).map_err(|e| format!("ポート番号が不正です: {e}"))?;
        Ok(ApplicationEntity::construct(
            record.uuid,
            title,
            record.host,
            port,
            record.comment,
            record.created_by,
            record.created_at,
            record.updated_by,
            record.updated_at,
        ))
    }
}

pub struct PostgresApplicationEntityRepository {
    pool: Pool<Postgres>,
}

impl PostgresApplicationEntityRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ApplicationEntityRepository for PostgresApplicationEntityRepository {
    async fn find_all(&self) -> Result<Vec<ApplicationEntity>, RepositoryError> {
        let records = sqlx::query_as::<_, ApplicationEntityRecord>(
            "SELECT uuid, title, host, port, comment, created_by, created_at, updated_by, updated_at
             FROM application_entities
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

    async fn find_by_title(
        &self,
        title: &AeValue,
    ) -> Result<Option<ApplicationEntity>, RepositoryError> {
        let record = sqlx::query_as::<_, ApplicationEntityRecord>(
            "SELECT uuid, title, host, port, comment, created_by, created_at, updated_by, updated_at
             FROM application_entities
             WHERE title = $1",
        )
        .bind(title.value())
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

    async fn add(&self, entity: &ApplicationEntity) -> Result<ApplicationEntity, RepositoryError> {
        let record = sqlx::query_as::<_, ApplicationEntityRecord>(
            "INSERT INTO application_entities (uuid, title, host, port, comment, created_by, created_at, updated_by, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             RETURNING uuid, title, host, port, comment, created_by, created_at, updated_by, updated_at",
        )
        .bind(entity.uuid())
        .bind(entity.title().value())
        .bind(entity.host())
        .bind(entity.port().value() as i32)
        .bind(entity.comment())
        .bind(entity.created_by())
        .bind(entity.created_at())
        .bind(entity.updated_by())
        .bind(entity.updated_at())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error()
                && db_err.is_unique_violation()
            {
                return RepositoryError::Conflict {
                    resource: "AEタイトル".to_string(),
                    key: entity.title().value().to_string(),
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

    async fn update(
        &self,
        old_title: &AeValue,
        entity: &ApplicationEntity,
    ) -> Result<ApplicationEntity, RepositoryError> {
        let record = sqlx::query_as::<_, ApplicationEntityRecord>(
            "UPDATE application_entities
             SET title = $1, host = $2, port = $3, comment = $4, updated_by = $5, updated_at = $6
             WHERE title = $7
             RETURNING uuid, title, host, port, comment, created_by, created_at, updated_by, updated_at",
        )
        .bind(entity.title().value())
        .bind(entity.host())
        .bind(entity.port().value() as i32)
        .bind(entity.comment())
        .bind(entity.updated_by())
        .bind(entity.updated_at())
        .bind(old_title.value())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error()
                && db_err.is_unique_violation()
            {
                return RepositoryError::Conflict {
                    resource: "AEタイトル".to_string(),
                    key: entity.title().value().to_string(),
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
                resource: "AEタイトル".to_string(),
                key: old_title.value().to_string(),
            }),
        }
    }

    async fn delete(
        &self,
        title: &AeValue,
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
            "INSERT INTO application_entities_deleted (uuid, title, host, port, comment, created_by, created_at, updated_by, updated_at, deleted_by, deleted_at)
             SELECT uuid, title, host, port, comment, created_by, created_at, updated_by, updated_at, $1, $2
             FROM application_entities
             WHERE title = $3",
        )
        .bind(deleted_by)
        .bind(deleted_at)
        .bind(title.value())
        .execute(&mut *tx)
        .await
        .map_err(|e| RepositoryError::Other {
            message: format!("データベース処理でエラーが発生しました: {e}"),
        })?
        .rows_affected();

        if rows_affected == 0 {
            return Err(RepositoryError::NotFound {
                resource: "AEタイトル".to_string(),
                key: title.value().to_string(),
            });
        }

        // 元テーブルからDELETE
        sqlx::query("DELETE FROM application_entities WHERE title = $1")
            .bind(title.value())
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
pub struct TestApplicationEntityRepository {
    inner: Arc<RwLock<HashMap<Uuid, ApplicationEntity>>>,
}

#[cfg(test)]
impl TestApplicationEntityRepository {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn find_uuid_by_title(&self, title: &AeValue) -> Option<Uuid> {
        self.inner
            .read()
            .unwrap()
            .values()
            .find(|e| e.title() == title)
            .cloned()
            .map(|e| *e.uuid())
    }
}

#[cfg(test)]
#[async_trait::async_trait]
impl ApplicationEntityRepository for TestApplicationEntityRepository {
    async fn find_all(&self) -> Result<Vec<ApplicationEntity>, RepositoryError> {
        Ok(self.inner.read().unwrap().values().cloned().collect())
    }

    async fn find_by_title(
        &self,
        title: &AeValue,
    ) -> Result<Option<ApplicationEntity>, RepositoryError> {
        Ok(self
            .inner
            .read()
            .unwrap()
            .values()
            .find(|e| e.title() == title)
            .cloned())
    }

    async fn add(&self, entity: &ApplicationEntity) -> Result<ApplicationEntity, RepositoryError> {
        // タイトルが既存エンティティと競合する場合はエラー
        if self.find_by_title(entity.title()).await?.is_some() {
            return Err(RepositoryError::Conflict {
                resource: "AEタイトル".to_string(),
                key: entity.title().value().to_string(),
            });
        }
        // ホスト名/IPアドレスとポート番号の組が既存エンティティと競合する場合はエラー
        if self
            .inner
            .read()
            .unwrap()
            .values()
            .any(|e| e.host() == entity.host() && e.port() == entity.port())
        {
            return Err(RepositoryError::Conflict {
                resource: "AEタイトル".to_string(),
                key: entity.title().value().to_string(),
            });
        }
        self.inner
            .write()
            .unwrap()
            .insert(*entity.uuid(), entity.clone());
        Ok(entity.clone())
    }

    async fn update(
        &self,
        target_title: &AeValue,
        application_entity: &ApplicationEntity,
    ) -> Result<ApplicationEntity, RepositoryError> {
        // 更新対象のエンティティが存在しない場合はエラー
        let Some(existing_uuid) = self.find_uuid_by_title(target_title) else {
            return Err(RepositoryError::NotFound {
                resource: "AEタイトル".to_string(),
                key: target_title.value().to_string(),
            });
        };

        // 更新後のタイトルが他のエンティティと重複する場合はエラー
        let inner = self.inner.read().unwrap();
        if application_entity.title() != target_title
            && inner
                .values()
                .any(|e| e.title() == application_entity.title())
        {
            return Err(RepositoryError::Conflict {
                resource: "AEタイトル".to_string(),
                key: application_entity.title().value().to_string(),
            });
        }
        // 更新後のホスト名/IPアドレスとポート番号の組が他のエンティティと重複する場合はエラー
        if inner.values().any(|e| {
            e.uuid() != &existing_uuid
                && e.host() == application_entity.host()
                && e.port() == application_entity.port()
        }) {
            return Err(RepositoryError::Conflict {
                resource: "AEタイトル".to_string(),
                key: application_entity.title().value().to_string(),
            });
        }
        drop(inner);

        self.inner
            .write()
            .unwrap()
            .insert(existing_uuid, application_entity.clone());
        Ok(application_entity.clone())
    }

    async fn delete(
        &self,
        target_title: &AeValue,
        _deleted_by: &Uuid,
        _deleted_at: &DateTime<Utc>,
    ) -> Result<(), RepositoryError> {
        // 削除対象のタイトルを持つエンティティが存在しない場合はエラー
        let Some(uuid) = self.find_uuid_by_title(target_title) else {
            return Err(RepositoryError::NotFound {
                resource: "AEタイトル".to_string(),
                key: target_title.value().to_string(),
            });
        };

        self.inner.write().unwrap().remove(&uuid);
        Ok(())
    }
}
