use chrono::NaiveDate;

/// 検査検索結果の1レコード（患者情報 + 検査情報 + モダリティ集約）
#[derive(Debug, Clone)]
pub struct StudyRecord {
    /// 患者ID
    pub patient_id: String,
    /// 患者名（アルファベット）
    pub patient_name_alphabet: String,
    /// 患者名（漢字）
    pub patient_name_kanji: String,
    /// 患者名（ひらがな）
    pub patient_name_hiragana: String,
    /// 患者の性別（ISO 5218: 0=不明, 1=男性, 2=女性, 9=適用不能）
    pub patient_sex: i16,
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
