use chrono::{NaiveDate, NaiveTime};
use dicom_lib::{
    core::value::value_representations::{da::DaValue, sh::ShValue, tm::TmValue, ui::UiValue},
    network::service_class::storage::{Status, status::code::DataSetMismatch},
};

pub struct Study {
    instance_uid: UiValue,
    id: String,
    date: Option<DaValue>,
    time: Option<TmValue>,
    accession_number: Option<ShValue>,
}

impl Study {
    pub fn instance_uid(&self) -> &str {
        self.instance_uid.uid()
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn date(&self) -> Option<&NaiveDate> {
        self.date.as_ref().map(|da_value| da_value.date())
    }

    pub fn time(&self) -> Option<&NaiveTime> {
        self.time.as_ref().map(|tm_value| tm_value.time())
    }

    pub fn accession_number(&self) -> &str {
        self.accession_number
            .as_ref()
            .map(|sh_value| sh_value.string())
            .unwrap_or("")
    }

    pub fn new(
        study_instance_uid: Option<UiValue>,
        study_date: Option<DaValue>,
        study_time: Option<TmValue>,
        study_id: Option<ShValue>,
        accession_number: Option<ShValue>,
    ) -> Result<Self, (String, Status)> {
        let instance_uid = study_instance_uid.ok_or((
            "Study Instance UIDが見つかりませんでした".to_string(),
            Status::DataSetDoesNotMatchSopClassError(DataSetMismatch::new(0xa900).unwrap()),
        ))?;
        let id = if let Some(study_id) = &study_id {
            study_id.string()
        } else {
            ""
        }
        .to_string();
        let date = study_date;
        let time = study_time;

        Ok(Study {
            instance_uid,
            id,
            date,
            time,
            accession_number,
        })
    }
}
