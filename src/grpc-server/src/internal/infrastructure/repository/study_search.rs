use async_trait::async_trait;
use chrono::{NaiveDate, NaiveTime};
use sqlx::{Pool, Postgres, QueryBuilder};

use crate::internal::domain::{
    entity::study_record::StudyRecord, error::RepositoryError,
    repository::study_search::StudySearchRepository, value_object::search_criteria::SearchCriteria,
};

pub struct PostgresStudySearchRepository {
    pool: Pool<Postgres>,
}

impl PostgresStudySearchRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

/// DBから取得する生の行データ
#[derive(sqlx::FromRow)]
struct StudyRow {
    patient_id: Option<String>,
    patient_name_alphabet: Option<String>,
    patient_name_kanji: Option<String>,
    patient_name_hiragana: Option<String>,
    patient_sex: Option<i16>,
    patient_birth_date: Option<NaiveDate>,
    study_instance_uid: String,
    study_id: String,
    study_date: Option<NaiveDate>,
    study_time: Option<NaiveTime>,
    accession_number: String,
    modalities: Option<Vec<String>>,
    total_count: Option<i64>,
}

#[async_trait]
impl StudySearchRepository for PostgresStudySearchRepository {
    async fn search(
        &self,
        criteria: &SearchCriteria,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<StudyRecord>, i32), RepositoryError> {
        // 動的SQLクエリの構築
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
SELECT
    p.id AS patient_id,
    p.name_alphabet AS patient_name_alphabet,
    p.name_kanji AS patient_name_kanji,
    p.name_hiragana AS patient_name_hiragana,
    p.sex AS patient_sex,
    p.birth_date AS patient_birth_date,
    s.instance_uid AS study_instance_uid,
    s.id AS study_id,
    s.study_date,
    s.study_time,
    s.accession_number,
    ARRAY(
        SELECT DISTINCT se.modality
        FROM series se
        WHERE se.study_instance_uid = s.instance_uid
        ORDER BY se.modality
    ) AS modalities,
    COUNT(*) OVER() AS total_count
FROM studies s
LEFT JOIN patients p ON p.id = s.patient_id
WHERE 1=1
"#,
        );

        // 患者ID（部分一致）
        if let Some(ref patient_id) = criteria.patient_id {
            qb.push(" AND p.id ILIKE '%' || ");
            qb.push_bind(patient_id.clone());
            qb.push(" || '%'");
        }

        // 患者名（部分一致、alphabet/kanji/hiraganaのいずれか）
        if let Some(ref patient_name) = criteria.patient_name {
            qb.push(" AND (p.name_alphabet ILIKE '%' || ");
            qb.push_bind(patient_name.clone());
            qb.push(" || '%' OR p.name_kanji ILIKE '%' || ");
            qb.push_bind(patient_name.clone());
            qb.push(" || '%' OR p.name_hiragana ILIKE '%' || ");
            qb.push_bind(patient_name.clone());
            qb.push(" || '%')");
        }

        // 検査日の開始日
        if let Some(ref from) = criteria.study_date_from {
            qb.push(" AND s.study_date >= ");
            qb.push_bind(*from);
        }

        // 検査日の終了日
        if let Some(ref to) = criteria.study_date_to {
            qb.push(" AND s.study_date <= ");
            qb.push_bind(*to);
        }

        // Accession Number（部分一致）
        if let Some(ref accession_number) = criteria.accession_number {
            qb.push(" AND s.accession_number ILIKE '%' || ");
            qb.push_bind(accession_number.clone());
            qb.push(" || '%'");
        }

        // モダリティ（部分一致）
        if let Some(ref modality) = criteria.modality {
            qb.push(
                " AND EXISTS (SELECT 1 FROM series se WHERE se.study_instance_uid = s.instance_uid AND se.modality ILIKE '%' || ",
            );
            qb.push_bind(modality.clone());
            qb.push(" || '%')");
        }

        // 検査ID（部分一致）
        if let Some(ref study_id) = criteria.study_id {
            qb.push(" AND s.id ILIKE '%' || ");
            qb.push_bind(study_id.clone());
            qb.push(" || '%'");
        }

        // ソートとページネーション
        qb.push(" ORDER BY s.study_date DESC NULLS LAST, s.study_time DESC NULLS LAST");
        qb.push(" LIMIT ");
        qb.push_bind(limit);
        qb.push(" OFFSET ");
        qb.push_bind(offset);

        let rows = qb
            .build_query_as::<StudyRow>()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepositoryError::Other {
                message: format!("検査の検索に失敗しました: {e}"),
            })?;

        let total_count = rows.first().map_or(0, |r| r.total_count.unwrap_or(0)) as i32;

        let studies = rows
            .into_iter()
            .map(|row| StudyRecord {
                patient_id: row.patient_id.unwrap_or_default(),
                patient_name_alphabet: row.patient_name_alphabet.unwrap_or_default(),
                patient_name_kanji: row.patient_name_kanji.unwrap_or_default(),
                patient_name_hiragana: row.patient_name_hiragana.unwrap_or_default(),
                patient_sex: row.patient_sex.unwrap_or(0),
                patient_birth_date: row.patient_birth_date,
                study_instance_uid: row.study_instance_uid,
                study_id: row.study_id,
                study_date: row.study_date,
                study_time: row.study_time,
                accession_number: row.accession_number,
                modalities: row.modalities.unwrap_or_default(),
            })
            .collect();

        Ok((studies, total_count))
    }
}
