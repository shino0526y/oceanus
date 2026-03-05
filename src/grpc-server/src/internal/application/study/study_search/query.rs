use async_trait::async_trait;
use chrono::NaiveDate;

/// 検査検索結果の1レコード（患者情報 + 検査情報 + モダリティ集約）
#[derive(Debug, Clone)]
pub struct SearchStudiesRecord {
    /// 患者ID
    pub patient_id: String,
    /// 患者名（アルファベット）
    pub patient_name_alphabet: String,
    /// 患者名（漢字）
    pub patient_name_kanji: String,
    /// 患者名（ひらがな）
    pub patient_name_hiragana: String,
    /// 患者の性別（ISO 5218: 0=不明, 1=男性, 2=女性, 9=適用不能）
    pub patient_sex: i32,
    /// 患者の生年月日
    pub patient_birth_date: Option<NaiveDate>,
    /// Study Instance UID
    pub study_instance_uid: String,
    /// 検査ID
    pub study_id: String,
    /// 検査日
    pub study_date: Option<NaiveDate>,
    /// 検査時刻
    pub study_time: Option<chrono::NaiveTime>,
    /// Accession Number
    pub accession_number: String,
    /// この検査に含まれるモダリティのリスト（重複なし）
    pub modalities: Vec<String>,
}

/// 検査検索クエリエラー
#[derive(Debug, thiserror::Error)]
#[error("検査検索クエリの実行に失敗しました: {message}")]
pub struct QueryError {
    pub message: String,
}

/// 検査検索クエリトレイト
#[async_trait]
pub trait SearchStudiesQuery: Send + Sync {
    /// 検索条件に一致する検査を取得する
    ///
    /// # 戻り値
    /// `(検査レコードのリスト, 総件数)` のタプル
    #[allow(clippy::too_many_arguments)]
    async fn search(
        &self,
        patient_id: &Option<String>,
        patient_name: &Option<String>,
        study_date_from: &Option<NaiveDate>,
        study_date_to: &Option<NaiveDate>,
        accession_number: &Option<String>,
        modality: &Option<String>,
        study_id: &Option<String>,
        limit: i32,
        offset: i32,
    ) -> Result<(Vec<SearchStudiesRecord>, i32), QueryError>;
}
