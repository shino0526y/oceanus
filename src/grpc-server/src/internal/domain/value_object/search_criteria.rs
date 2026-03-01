use chrono::NaiveDate;

use super::super::error::DomainError;

/// 検査検索の条件
#[derive(Debug, Clone, Default)]
pub struct SearchCriteria {
    /// 患者ID（部分一致）
    pub patient_id: Option<String>,
    /// 患者名（部分一致、alphabet/kanji/hiraganaのいずれかにマッチ）
    pub patient_name: Option<String>,
    /// 検査日の開始日
    pub study_date_from: Option<NaiveDate>,
    /// 検査日の終了日
    pub study_date_to: Option<NaiveDate>,
    /// Accession Number（部分一致）
    pub accession_number: Option<String>,
    /// モダリティ（部分一致）
    pub modality: Option<String>,
    /// 検査ID（部分一致）
    pub study_id: Option<String>,
}

impl SearchCriteria {
    /// 検索条件をバリデーションする
    pub fn validate(&self) -> Result<(), DomainError> {
        if let (Some(from), Some(to)) = (&self.study_date_from, &self.study_date_to)
            && from > to
        {
            return Err(DomainError::InvalidSearchCriteria {
                message: format!(
                    "検査日の開始日が終了日より後になっています (開始日=\"{from}\", 終了日=\"{to}\")"
                ),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_date_range() {
        let criteria = SearchCriteria {
            study_date_from: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            study_date_to: Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
            ..Default::default()
        };
        assert!(criteria.validate().is_ok());
    }

    #[test]
    fn test_validate_same_date() {
        let criteria = SearchCriteria {
            study_date_from: Some(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()),
            study_date_to: Some(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()),
            ..Default::default()
        };
        assert!(criteria.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_date_range() {
        let criteria = SearchCriteria {
            study_date_from: Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
            study_date_to: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            ..Default::default()
        };
        assert!(criteria.validate().is_err());
    }

    #[test]
    fn test_validate_no_dates() {
        let criteria = SearchCriteria::default();
        assert!(criteria.validate().is_ok());
    }

    #[test]
    fn test_validate_only_from_date() {
        let criteria = SearchCriteria {
            study_date_from: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            ..Default::default()
        };
        assert!(criteria.validate().is_ok());
    }

    #[test]
    fn test_validate_only_to_date() {
        let criteria = SearchCriteria {
            study_date_to: Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
            ..Default::default()
        };
        assert!(criteria.validate().is_ok());
    }
}
