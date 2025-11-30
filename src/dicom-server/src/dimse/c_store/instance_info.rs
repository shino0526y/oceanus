mod patient;
mod series;
mod sop_instance;
mod study;

pub use patient::Patient;
pub use series::Series;
pub use sop_instance::SopInstance;
pub use study::Study;

use dicom_lib::core::{
    DataSet, Tag,
    value::{
        SpecificCharacterSet,
        value_representations::{
            cs::CsValue, da::DaValue, is::IsValue, lo::LoValue, pn::PnValue, sh::ShValue,
            tm::TmValue, ui::UiValue,
        },
    },
};

pub struct InstanceInfo {
    pub patient: Patient,
    pub study: Study,
    pub series: Series,
    pub sop_instance: SopInstance,
}

impl InstanceInfo {
    pub fn from_data_set(data_set: &DataSet) -> Result<Self, String> {
        let mut char_set = SpecificCharacterSet::None;

        // Patient Module
        // https://dicom.nema.org/medical/dicom/2025c/output/chtml/part03/sect_C.7.html#sect_C.7.1.1
        let mut patients_name = Option::None;
        let mut patient_id = Option::None;
        let mut patients_birth_date = Option::None;
        let mut patients_sex = Option::None;

        // General Study Module
        // https://dicom.nema.org/medical/dicom/2025c/output/chtml/part03/sect_C.7.2.html#sect_C.7.2.1
        let mut study_instance_uid = Option::None;
        let mut study_date = Option::None;
        let mut study_time = Option::None;
        let mut study_id = Option::None;
        let mut accession_number = Option::None;

        // General Series Module
        // https://dicom.nema.org/medical/dicom/2025c/output/chtml/part03/sect_C.7.3.html#sect_C.7.3.1
        let mut series_instance_uid = Option::None;
        let mut modality = Option::None;
        let mut series_number = Option::None;

        // SOP Common Module
        // https://dicom.nema.org/medical/dicom/2025c/output/chtml/part03/sect_C.12.html#sect_C.12.1
        let mut sop_class_uid = Option::None;
        let mut sop_instance_uid = Option::None;

        // General Image Module
        // https://dicom.nema.org/medical/dicom/2025c/output/chtml/part03/sect_C.7.6.html#sect_C.7.6.1
        let mut instance_number = Option::None;

        let mut i = 0;
        while i < data_set.len() {
            let descendants_count = data_set.get_descendants_count(i);
            if descendants_count > 0 {
                i += descendants_count + 1;
                continue;
            }

            let element = &data_set[i];
            let value_field = element.value_field();
            if value_field.is_empty() {
                i += 1;
                continue;
            }

            match element.tag() {
                Tag(0x0008, 0x0005) => {
                    char_set = SpecificCharacterSet::try_from(value_field)
                        .map_err(|e| format!("Specific Character Setのパースに失敗しました: {e}"))?
                }

                Tag(0x0008, 0x0016) => {
                    sop_class_uid = Some(
                        UiValue::from_bytes(value_field)
                            .map_err(|e| format!("SOP Class UIDのパースに失敗しました: {e}"))?,
                    )
                }

                Tag(0x0008, 0x0018) => {
                    sop_instance_uid = Some(
                        UiValue::from_bytes(value_field)
                            .map_err(|e| format!("SOP Instance UIDのパースに失敗しました: {e}"))?,
                    )
                }

                Tag(0x0008, 0x0020) => {
                    study_date = Some(
                        DaValue::from_bytes(value_field)
                            .map_err(|e| format!("Study Dateのパースに失敗しました: {e}"))?,
                    )
                }

                Tag(0x0008, 0x0030) => {
                    study_time = Some(
                        TmValue::from_bytes(value_field)
                            .map_err(|e| format!("Study Timeのパースに失敗しました: {e}"))?,
                    )
                }

                Tag(0x0008, 0x0050) => {
                    accession_number = Some(
                        ShValue::from_bytes_lossy(value_field, char_set)
                            .map_err(|e| format!("Accession Numberのパースに失敗しました: {e}"))?,
                    )
                }

                Tag(0x0008, 0x0060) => {
                    modality = Some(
                        CsValue::from_bytes(value_field)
                            .map_err(|e| format!("Modalityのパースに失敗しました: {e}"))?,
                    )
                }

                Tag(0x0010, 0x0010) => {
                    patients_name = Some(
                        PnValue::from_bytes_lossy(value_field, char_set)
                            .map_err(|e| format!("Patient's Nameのパースに失敗しました: {e}"))?,
                    )
                }

                Tag(0x0010, 0x0020) => {
                    patient_id = Some(
                        LoValue::from_bytes_lossy(value_field, char_set)
                            .map_err(|e| format!("Patient IDのパースに失敗しました: {e}"))?,
                    )
                }

                Tag(0x0010, 0x0030) => {
                    patients_birth_date =
                        Some(DaValue::from_bytes(value_field).map_err(|e| {
                            format!("Patient's Birth Dateのパースに失敗しました: {e}")
                        })?)
                }

                Tag(0x0010, 0x0040) => {
                    patients_sex = Some(
                        CsValue::from_bytes(value_field)
                            .map_err(|e| format!("Patient's Sexのパースに失敗しました: {e}"))?,
                    )
                }

                Tag(0x0020, 0x000d) => {
                    study_instance_uid =
                        Some(UiValue::from_bytes(value_field).map_err(|e| {
                            format!("Study Instance UIDのパースに失敗しました: {e}")
                        })?)
                }

                Tag(0x0020, 0x000e) => {
                    series_instance_uid =
                        Some(UiValue::from_bytes(value_field).map_err(|e| {
                            format!("Series Instance UIDのパースに失敗しました: {e}")
                        })?)
                }

                Tag(0x0020, 0x0010) => {
                    study_id = Some(
                        ShValue::from_bytes_lossy(value_field, char_set)
                            .map_err(|e| format!("Study IDのパースに失敗しました: {e}"))?,
                    )
                }

                Tag(0x0020, 0x0011) => {
                    series_number = Some(
                        IsValue::from_bytes(value_field)
                            .map_err(|e| format!("Series Numberのパースに失敗しました: {e}"))?,
                    )
                }

                Tag(0x0020, 0x0013) => {
                    instance_number = Some(
                        IsValue::from_bytes(value_field)
                            .map_err(|e| format!("Instance Numberのパースに失敗しました: {e}"))?,
                    );

                    break;
                }

                _ => {}
            }

            i += 1;
        }

        let patient = Patient::new(
            char_set,
            patients_name,
            patient_id,
            patients_birth_date,
            patients_sex,
        )?;
        let study = Study::new(
            study_instance_uid,
            study_date,
            study_time,
            study_id,
            accession_number,
        )?;
        let series = Series::new(modality, series_instance_uid, series_number)?;
        let sop_instance = SopInstance::new(sop_class_uid, sop_instance_uid, instance_number)?;

        Ok(InstanceInfo {
            patient,
            study,
            series,
            sop_instance,
        })
    }
}
