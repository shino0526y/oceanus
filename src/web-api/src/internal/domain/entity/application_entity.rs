use crate::internal::domain::value_object::Port;
use chrono::{DateTime, Utc};
use dicom_lib::core::value::value_representations::ae::AeValue;
use uuid::{NoContext, Timestamp, Uuid};

pub struct ApplicationEntity {
    uuid: Uuid,
    title: AeValue,
    host: String,
    port: Port,
    comment: String,
    created_by: Uuid,
    created_at: DateTime<Utc>,
    updated_by: Uuid,
    updated_at: DateTime<Utc>,
}

impl ApplicationEntity {
    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    pub fn title(&self) -> &AeValue {
        &self.title
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> &Port {
        &self.port
    }

    pub fn comment(&self) -> &str {
        &self.comment
    }

    pub fn created_by(&self) -> &Uuid {
        &self.created_by
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_by(&self) -> &Uuid {
        &self.updated_by
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    #[allow(clippy::too_many_arguments)]
    pub fn construct(
        uuid: Uuid,
        title: AeValue,
        host: impl Into<String>,
        port: Port,
        comment: impl Into<String>,
        created_by: Uuid,
        created_at: DateTime<Utc>,
        updated_by: Uuid,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            uuid,
            title,
            host: host.into(),
            port,
            comment: comment.into(),
            created_by,
            created_at,
            updated_by,
            updated_at,
        }
    }

    pub fn create(
        title: AeValue,
        host: impl Into<String>,
        port: Port,
        comment: impl Into<String>,
        created_by: Uuid,
        created_at: DateTime<Utc>,
    ) -> Self {
        let timestamp = Timestamp::from_unix(NoContext, created_at.timestamp_millis() as u64, 0);

        Self {
            uuid: Uuid::new_v7(timestamp),
            title,
            host: host.into(),
            port,
            comment: comment.into(),
            created_by,
            created_at,
            updated_by: created_by,
            updated_at: created_at,
        }
    }

    /// アプリケーションエンティティを更新する。ただし、変更があった場合のみ更新を行う。
    ///
    /// # Returns
    /// 変更があった場合は`true`、変更がなかった場合は`false`を返す。
    pub fn update(
        &mut self,
        title: AeValue,
        host: impl Into<String>,
        port: Port,
        comment: impl Into<String>,
        updated_by: Uuid,
        updated_at: DateTime<Utc>,
    ) -> bool {
        assert!(
            updated_at >= self.created_at,
            "`updated_at`は`created_at`よりも前にはできません (created_at={}, updated_at={})",
            self.created_at,
            updated_at,
        );

        // 変更がない場合は何もしない
        let host = host.into();
        let comment = comment.into();
        if title == self.title && host == self.host && port == self.port && comment == self.comment
        {
            return false;
        }

        self.title = title;
        self.host = host;
        self.port = port;
        self.comment = comment;
        self.updated_by = updated_by;
        self.updated_at = updated_at;

        true
    }
}
