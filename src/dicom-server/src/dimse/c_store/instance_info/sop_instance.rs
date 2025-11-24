use dicom_lib::core::value::values::{Is, Ui};

pub struct SopInstance {
    pub instance_uid: String,
    pub class_uid: String,
    pub number: Option<i32>,
}

impl SopInstance {
    pub fn new(
        class_uid: Option<Ui>,
        instance_uid: Option<Ui>,
        instance_number: Option<Is>,
    ) -> Result<Self, String> {
        let instance_uid = instance_uid
            .ok_or("SOP Instance UIDが見つかりませんでした".to_string())?
            .uid()
            .to_string();

        let class_uid = class_uid
            .ok_or("SOP Class UIDが見つかりませんでした".to_string())?
            .uid()
            .to_string();

        let number = if let Some(instance_number) = instance_number {
            Some(instance_number.value())
        } else {
            None
        };

        Ok(SopInstance {
            instance_uid,
            class_uid,
            number,
        })
    }
}
