use std::sync::Arc;

use chrono::NaiveDate;
use tonic::{Request, Response, Status};

use proto::oceanus::v1::{
    SearchStudiesRequest, SearchStudiesResponse, StudyRecord as ProtoStudyRecord,
    study_search_service_server::StudySearchService,
};

use crate::internal::{
    application::study_search::{SearchStudiesError, SearchStudiesUseCase},
    domain::value_object::search_criteria::SearchCriteria,
};

/// gRPC StudySearchService のサーバー実装
pub struct StudySearchServiceImpl {
    use_case: Arc<SearchStudiesUseCase>,
}

impl StudySearchServiceImpl {
    pub fn new(use_case: Arc<SearchStudiesUseCase>) -> Self {
        Self { use_case }
    }
}

#[tonic::async_trait]
impl StudySearchService for StudySearchServiceImpl {
    async fn search_studies(
        &self,
        request: Request<SearchStudiesRequest>,
    ) -> Result<Response<SearchStudiesResponse>, Status> {
        let req = request.into_inner();

        // Proto型 → ドメイン型への変換
        let criteria = convert_to_criteria(&req)?;

        let (studies, total_count) = self
            .use_case
            .execute(criteria, req.limit, req.offset)
            .await
            .map_err(to_status)?;

        // ドメイン型 → Proto型への変換
        let proto_studies = studies
            .into_iter()
            .map(|s| ProtoStudyRecord {
                patient_id: s.patient_id,
                patient_name_alphabet: s.patient_name_alphabet,
                patient_name_kanji: s.patient_name_kanji,
                patient_name_hiragana: s.patient_name_hiragana,
                patient_sex: s.patient_sex as i32,
                patient_birth_date: s
                    .patient_birth_date
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_default(),
                study_instance_uid: s.study_instance_uid,
                study_id: s.study_id,
                study_date: s
                    .study_date
                    .map(|d| d.format("%Y-%m-%d").to_string())
                    .unwrap_or_default(),
                study_time: s
                    .study_time
                    .map(|t| t.format("%H:%M:%S").to_string())
                    .unwrap_or_default(),
                accession_number: s.accession_number,
                modalities: s.modalities,
            })
            .collect();

        Ok(Response::new(SearchStudiesResponse {
            studies: proto_studies,
            total_count,
        }))
    }
}

/// Proto のリクエストを SearchCriteria に変換する
#[allow(clippy::result_large_err)]
fn convert_to_criteria(req: &SearchStudiesRequest) -> Result<SearchCriteria, Status> {
    let study_date_from = parse_optional_date(&req.study_date_from, "study_date_from")?;
    let study_date_to = parse_optional_date(&req.study_date_to, "study_date_to")?;

    Ok(SearchCriteria {
        patient_id: non_empty(&req.patient_id),
        patient_name: non_empty(&req.patient_name),
        study_date_from,
        study_date_to,
        accession_number: non_empty(&req.accession_number),
        modality: non_empty(&req.modality),
        study_id: non_empty(&req.study_id),
    })
}

/// 空文字列を None に変換する
fn non_empty(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

/// 文字列をオプショナルな日付に変換する
#[allow(clippy::result_large_err)]
fn parse_optional_date(s: &str, field_name: &str) -> Result<Option<NaiveDate>, Status> {
    if s.is_empty() {
        return Ok(None);
    }
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map(Some)
        .map_err(|e| {
            Status::invalid_argument(format!(
                "文字列から日付へのパースに失敗しました (フィールド=\"{field_name}\", 文字列=\"{s}\"): {e}"
            ))
        })
}

/// SearchStudiesError を tonic::Status に変換する
fn to_status(err: SearchStudiesError) -> Status {
    match err {
        SearchStudiesError::InvalidCriteria(e) => Status::invalid_argument(e.to_string()),
        SearchStudiesError::Repository(e) => Status::internal(e.to_string()),
    }
}
