use crate::core::{
    DataElement, Tag,
    data_element::Vr,
    value::value_representations::{
        Ob, ae::AeValue, fd::FdValue, sh::ShValue, ui::UiValue, ul::UlValue, ur::UrValue,
    },
};
use std::slice::Iter;

// https://dicom.nema.org/medical/dicom/2025c/output/chtml/part10/chapter_7.html
pub struct FileMetaInformation {
    meta_data_elements: Vec<DataElement>,

    file_meta_information_group_length: UlValue,
    file_meta_information_version: Ob,
    media_storage_sop_class_uid: UiValue,
    media_storage_sop_instance_uid: UiValue,
    transfer_syntax_uid: UiValue,
    implementation_class_uid: UiValue,
    implementation_version_name: Option<ShValue>,
    source_application_entity_title: Option<AeValue>,
    sending_application_entity_title: Option<AeValue>,
    receiving_application_entity_title: Option<AeValue>,
    source_presentation_address: Option<UrValue>,
    sending_presentation_address: Option<UrValue>,
    receiving_presentation_address: Option<UrValue>,
    rtv_meta_information_version: Option<Ob>,
    rtv_communication_sop_class_uid: Option<UiValue>,
    rtv_communication_sop_instance_uid: Option<UiValue>,
    rtv_source_identifier: Option<Ob>,
    rtv_flow_identifier: Option<Ob>,
    rtv_flow_rtp_sampling_rate: Option<UlValue>,
    rtv_flow_actual_frame_duration: Option<FdValue>,
    private_information_creator_uid: Option<UiValue>,
    private_information: Option<Ob>,
}

impl FileMetaInformation {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        media_storage_sop_class_uid: UiValue,
        media_storage_sop_instance_uid: UiValue,
        transfer_syntax_uid: UiValue,
        implementation_class_uid: UiValue,
        implementation_version_name: Option<ShValue>,
        source_application_entity_title: Option<AeValue>,
        sending_application_entity_title: Option<AeValue>,
        receiving_application_entity_title: Option<AeValue>,
        source_presentation_address: Option<UrValue>,
        sending_presentation_address: Option<UrValue>,
        receiving_presentation_address: Option<UrValue>,
        rtv_meta_information_version: Option<Ob>,
        rtv_communication_sop_class_uid: Option<UiValue>,
        rtv_communication_sop_instance_uid: Option<UiValue>,
        rtv_source_identifier: Option<Ob>,
        rtv_flow_identifier: Option<Ob>,
        rtv_flow_rtp_sampling_rate: Option<UlValue>,
        rtv_flow_actual_frame_duration: Option<FdValue>,
        private_information_creator_uid: Option<UiValue>,
        private_information: Option<Ob>,
    ) -> FileMetaInformation {
        let mut meta_data_elements = Vec::new();

        // File Meta Information Group Length
        meta_data_elements.push(DataElement::new(
            Tag(0x0002, 0x0000),
            Some(Vr::Ul),
            4,
            Vec::new(),
        ));

        // File Meta Information Version
        meta_data_elements.push(DataElement::new(
            Tag(0x0002, 0x0001),
            Some(Vr::Ob),
            2u32,
            vec![0x00, 0x01],
        ));
        // Media Storage SOP Class UID
        {
            let value_field = media_storage_sop_class_uid.to_bytes();
            meta_data_elements.push(DataElement::new(
                Tag(0x0002, 0x0002),
                Some(Vr::Ui),
                value_field.len() as u32,
                value_field,
            ));
        }

        // Media Storage SOP Instance UID
        {
            let value_field = media_storage_sop_instance_uid.to_bytes();
            meta_data_elements.push(DataElement::new(
                Tag(0x0002, 0x0003),
                Some(Vr::Ui),
                value_field.len() as u32,
                value_field,
            ));
        }

        // Transfer Syntax UID
        {
            let value_field = transfer_syntax_uid.to_bytes();
            meta_data_elements.push(DataElement::new(
                Tag(0x0002, 0x0010),
                Some(Vr::Ui),
                value_field.len() as u32,
                value_field,
            ));
        }

        // Implementation Class UID
        {
            let value_field = implementation_class_uid.to_bytes();
            meta_data_elements.push(DataElement::new(
                Tag(0x0002, 0x0012),
                Some(Vr::Ui),
                value_field.len() as u32,
                value_field,
            ));
        }

        // Implementation Version Name
        if let Some(v) = implementation_version_name.as_ref() {
            let mut value_field = v.string().as_bytes().to_vec();
            if !value_field.len().is_multiple_of(2) {
                value_field.push(b' ');
            }
            meta_data_elements.push(DataElement::new(
                Tag(0x0002, 0x0013),
                Some(Vr::Sh),
                value_field.len() as u32,
                value_field,
            ));
        }

        // Source Application Entity Title
        if let Some(v) = source_application_entity_title.as_ref() {
            let value_field = v.to_bytes();
            meta_data_elements.push(DataElement::new(
                Tag(0x0002, 0x0016),
                Some(Vr::Ae),
                value_field.len() as u32,
                value_field,
            ));
        }

        // Sending Application Entity Title
        if let Some(v) = sending_application_entity_title.as_ref() {
            let value_field = v.to_bytes();
            meta_data_elements.push(DataElement::new(
                Tag(0x0002, 0x0017),
                Some(Vr::Ae),
                value_field.len() as u32,
                value_field,
            ));
        }

        // Receiving Application Entity Title
        if let Some(v) = receiving_application_entity_title.as_ref() {
            let value_field = v.to_bytes();
            meta_data_elements.push(DataElement::new(
                Tag(0x0002, 0x0018),
                Some(Vr::Ae),
                value_field.len() as u32,
                value_field,
            ));
        }

        // Source Presentation Address
        if let Some(v) = source_presentation_address.as_ref() {
            let value_field = v.to_bytes();
            meta_data_elements.push(DataElement::new(
                Tag(0x0002, 0x0026),
                Some(Vr::Ur),
                value_field.len() as u32,
                value_field,
            ));
        }

        // Sending Presentation Address
        if let Some(v) = sending_presentation_address.as_ref() {
            let value_field = v.to_bytes();
            meta_data_elements.push(DataElement::new(
                Tag(0x0002, 0x0027),
                Some(Vr::Ur),
                value_field.len() as u32,
                value_field,
            ));
        }

        // Receiving Presentation Address
        if let Some(v) = receiving_presentation_address.as_ref() {
            let value_field = v.to_bytes();
            meta_data_elements.push(DataElement::new(
                Tag(0x0002, 0x0028),
                Some(Vr::Ur),
                value_field.len() as u32,
                value_field,
            ));
        }

        // RTV Meta Information Version
        if let Some(v) = rtv_meta_information_version.as_ref() {
            let value_field = v.to_bytes();
            meta_data_elements.push(DataElement::new(
                Tag(0x0002, 0x0031),
                Some(Vr::Ob),
                value_field.len() as u32,
                value_field,
            ));
        }

        // RTV Communication SOP Class UID
        if let Some(v) = rtv_communication_sop_class_uid.as_ref() {
            let value_field = v.to_bytes();
            meta_data_elements.push(DataElement::new(
                Tag(0x0002, 0x0032),
                Some(Vr::Ui),
                value_field.len() as u32,
                value_field,
            ));
        }

        // RTV Communication SOP Instance UID
        if let Some(v) = rtv_communication_sop_instance_uid.as_ref() {
            let value_field = v.to_bytes();
            meta_data_elements.push(DataElement::new(
                Tag(0x0002, 0x0033),
                Some(Vr::Ui),
                value_field.len() as u32,
                value_field,
            ));
        }

        // RTV Source Identifier
        if let Some(v) = rtv_source_identifier.as_ref() {
            let value_field = v.to_bytes();
            meta_data_elements.push(DataElement::new(
                Tag(0x0002, 0x0035),
                Some(Vr::Ob),
                value_field.len() as u32,
                value_field,
            ));
        }

        // RTV Flow Identifier
        if let Some(v) = rtv_flow_identifier.as_ref() {
            let value_field = v.to_bytes();
            meta_data_elements.push(DataElement::new(
                Tag(0x0002, 0x0036),
                Some(Vr::Ob),
                value_field.len() as u32,
                value_field,
            ));
        }

        // RTV Flow RTP Sampling Rate
        if let Some(v) = rtv_flow_rtp_sampling_rate.as_ref() {
            let value_field = v.to_bytes().to_vec();
            meta_data_elements.push(DataElement::new(
                Tag(0x0002, 0x0037),
                Some(Vr::Ul),
                value_field.len() as u32,
                value_field,
            ));
        }

        // RTV Flow Actual Frame Duration
        if let Some(v) = rtv_flow_actual_frame_duration.as_ref() {
            let value_field = v.to_bytes().to_vec();
            meta_data_elements.push(DataElement::new(
                Tag(0x0002, 0x0038),
                Some(Vr::Fd),
                value_field.len() as u32,
                value_field,
            ));
        }

        // Private Information Creator UID
        if let Some(v) = private_information_creator_uid.as_ref() {
            let value_field = v.to_bytes();
            meta_data_elements.push(DataElement::new(
                Tag(0x0002, 0x0100),
                Some(Vr::Ui),
                value_field.len() as u32,
                value_field,
            ));
        }

        // Private Information
        if let Some(v) = private_information.as_ref() {
            let value_field = v.to_bytes();
            meta_data_elements.push(DataElement::new(
                Tag(0x0002, 0x0102),
                Some(Vr::Ob),
                value_field.len() as u32,
                value_field,
            ));
        }

        // File Meta Information Group Lengthをセットする
        let file_meta_information_group_length: u32 = meta_data_elements
            .iter()
            .skip(1) // 最初の要素はFile Meta Information Group Length自身なのでスキップ
            .map(|e| e.size() as u32)
            .sum();
        meta_data_elements[0].value_field =
            file_meta_information_group_length.to_le_bytes().to_vec();

        FileMetaInformation {
            meta_data_elements,
            file_meta_information_group_length: UlValue(file_meta_information_group_length),
            file_meta_information_version: Ob(vec![0x00, 0x01]),
            media_storage_sop_class_uid,
            media_storage_sop_instance_uid,
            transfer_syntax_uid,
            implementation_class_uid,
            implementation_version_name,
            source_application_entity_title,
            sending_application_entity_title,
            receiving_application_entity_title,
            source_presentation_address,
            sending_presentation_address,
            receiving_presentation_address,
            rtv_meta_information_version,
            rtv_communication_sop_class_uid,
            rtv_communication_sop_instance_uid,
            rtv_source_identifier,
            rtv_flow_identifier,
            rtv_flow_rtp_sampling_rate,
            rtv_flow_actual_frame_duration,
            private_information_creator_uid,
            private_information,
        }
    }

    pub fn iter(&self) -> Iter<'_, DataElement> {
        self.meta_data_elements.iter()
    }

    pub fn file_meta_information_group_length(&self) -> &UlValue {
        &self.file_meta_information_group_length
    }

    pub fn file_meta_information_version(&self) -> &Ob {
        &self.file_meta_information_version
    }

    pub fn media_storage_sop_class_uid(&self) -> &UiValue {
        &self.media_storage_sop_class_uid
    }

    pub fn media_storage_sop_instance_uid(&self) -> &UiValue {
        &self.media_storage_sop_instance_uid
    }

    pub fn transfer_syntax_uid(&self) -> &UiValue {
        &self.transfer_syntax_uid
    }

    pub fn implementation_class_uid(&self) -> &UiValue {
        &self.implementation_class_uid
    }

    pub fn implementation_version_name(&self) -> Option<&ShValue> {
        self.implementation_version_name.as_ref()
    }

    pub fn source_application_entity_title(&self) -> Option<&AeValue> {
        self.source_application_entity_title.as_ref()
    }

    pub fn sending_application_entity_title(&self) -> Option<&AeValue> {
        self.sending_application_entity_title.as_ref()
    }

    pub fn receiving_application_entity_title(&self) -> Option<&AeValue> {
        self.receiving_application_entity_title.as_ref()
    }

    pub fn source_presentation_address(&self) -> Option<&UrValue> {
        self.source_presentation_address.as_ref()
    }

    pub fn sending_presentation_address(&self) -> Option<&UrValue> {
        self.sending_presentation_address.as_ref()
    }

    pub fn receiving_presentation_address(&self) -> Option<&UrValue> {
        self.receiving_presentation_address.as_ref()
    }

    pub fn rtv_meta_information_version(&self) -> Option<&Ob> {
        self.rtv_meta_information_version.as_ref()
    }

    pub fn rtv_communication_sop_class_uid(&self) -> Option<&UiValue> {
        self.rtv_communication_sop_class_uid.as_ref()
    }

    pub fn rtv_communication_sop_instance_uid(&self) -> Option<&UiValue> {
        self.rtv_communication_sop_instance_uid.as_ref()
    }

    pub fn rtv_source_identifier(&self) -> Option<&Ob> {
        self.rtv_source_identifier.as_ref()
    }

    pub fn rtv_flow_identifier(&self) -> Option<&Ob> {
        self.rtv_flow_identifier.as_ref()
    }

    pub fn rtv_flow_rtp_sampling_rate(&self) -> Option<&UlValue> {
        self.rtv_flow_rtp_sampling_rate.as_ref()
    }

    pub fn rtv_flow_actual_frame_duration(&self) -> Option<&FdValue> {
        self.rtv_flow_actual_frame_duration.as_ref()
    }

    pub fn private_information_creator_uid(&self) -> Option<&UiValue> {
        self.private_information_creator_uid.as_ref()
    }

    pub fn private_information(&self) -> Option<&Ob> {
        self.private_information.as_ref()
    }

    pub fn size(&self) -> usize {
        debug_assert!(!self.meta_data_elements.is_empty());
        debug_assert!(self.meta_data_elements[0].tag == Tag(0x0002, 0x0000));

        self.file_meta_information_group_length.value() as usize
            + self.meta_data_elements[0].size() as usize
    }
}

impl<'a> IntoIterator for &'a FileMetaInformation {
    type Item = &'a DataElement;
    type IntoIter = Iter<'a, DataElement>;

    fn into_iter(self) -> Self::IntoIter {
        self.meta_data_elements.iter()
    }
}

impl From<FileMetaInformation> for Vec<u8> {
    fn from(val: FileMetaInformation) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(val.size());
        for element in val.meta_data_elements {
            bytes.append(&mut element.into());
        }
        bytes
    }
}
