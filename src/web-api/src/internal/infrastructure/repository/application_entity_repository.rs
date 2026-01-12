use crate::internal::domain::{
    entity::ApplicationEntity, error::RepositoryError, repository::ApplicationEntityRepository,
    value_object::Port,
};
use chrono::{DateTime, Utc};
use dicom_lib::core::value::value_representations::ae::AeValue;
use sqlx::{FromRow, Pool, Postgres};

#[derive(FromRow)]
struct ApplicationEntityRecord {
    title: String,
    host: String,
    port: i32,
    comment: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<ApplicationEntityRecord> for ApplicationEntity {
    type Error = String;

    fn try_from(record: ApplicationEntityRecord) -> Result<Self, Self::Error> {
        let title = AeValue::from_string(&record.title)
            .map_err(|e| format!("AEタイトルが不正です: {e}"))?;
        let port = Port::from_i32(record.port).map_err(|e| format!("ポート番号が不正です: {e}"))?;
        Ok(ApplicationEntity::new(
            title,
            record.host,
            port,
            record.comment,
            record.created_at,
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
            "SELECT title, host, port, comment, created_at, updated_at
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

    async fn add(&self, entity: &ApplicationEntity) -> Result<ApplicationEntity, RepositoryError> {
        let ae_title = entity.title().value();
        let record = sqlx::query_as::<_, ApplicationEntityRecord>(
            "INSERT INTO application_entities (title, host, port, comment)
             VALUES ($1, $2, $3, $4)
             RETURNING title, host, port, comment, created_at, updated_at",
        )
        .bind(ae_title)
        .bind(entity.host())
        .bind(entity.port().value() as i32)
        .bind(entity.comment())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error()
                && db_err.is_unique_violation()
            {
                return RepositoryError::Duplicate {
                    resource: "AEタイトル".to_string(),
                    key: ae_title.to_string(),
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
}
