use dicom_lib::core::value::value_representations::{is::IsValue, ui::UiValue};

pub struct SopInstance {
    pub instance_uid: String,
    pub class_uid: String,
    pub number: Option<i32>,
}

impl SopInstance {
    pub fn new(
        class_uid: Option<UiValue>,
        instance_uid: Option<UiValue>,
        instance_number: Option<IsValue>,
    ) -> Result<Self, String> {
        let instance_uid = instance_uid
            .ok_or("SOP Instance UIDが見つかりませんでした".to_string())?
            .uid()
            .to_string();

        let class_uid = class_uid
            .ok_or("SOP Class UIDが見つかりませんでした".to_string())?
            .uid()
            .to_string();

        let number = instance_number.map(|instance_number| instance_number.value());

        Ok(SopInstance {
            instance_uid,
            class_uid,
            number,
        })
    }
}
