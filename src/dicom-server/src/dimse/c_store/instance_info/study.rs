use chrono::{NaiveDate, NaiveTime};
use dicom_lib::core::value::value_representations::{
    da::DaValue, sh::ShValue, tm::TmValue, ui::UiValue,
};

pub struct Study {
    pub instance_uid: String,
    pub id: String,
    pub date: Option<NaiveDate>,
    pub time: Option<NaiveTime>,
    pub accession_number: String,
}

impl Study {
    pub fn new(
        study_instance_uid: Option<UiValue>,
        study_date: Option<DaValue>,
        study_time: Option<TmValue>,
        study_id: Option<ShValue>,
        accession_number: Option<ShValue>,
    ) -> Result<Self, String> {
        let instance_uid = study_instance_uid
            .ok_or("Study Instance UIDが見つかりませんでした".to_string())?
            .uid()
            .to_string();

        let id = if let Some(study_id) = &study_id {
            study_id.string()
        } else {
            ""
        }
        .to_string();

        let date = study_date.map(|study_date| *study_date.date());
        let time = study_time.map(|study_time| *study_time.time());

        let accession_number = if let Some(accession_number) = &accession_number {
            accession_number.string()
        } else {
            ""
        }
        .to_string();

        Ok(Study {
            instance_uid,
            id,
            date,
            time,
            accession_number,
        })
    }
}
