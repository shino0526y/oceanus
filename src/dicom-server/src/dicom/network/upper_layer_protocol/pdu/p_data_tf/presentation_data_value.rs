use crate::dicom::network::upper_layer_protocol::pdu::INVALID_FIELD_LENGTH_ERROR_MESSAGE;

pub struct PresentationDataValue {
    length: u32,
    presentation_context_id: u8,
    message_control_header: u8,
    data: Vec<u8>,
}

impl PresentationDataValue {
    pub fn size(&self) -> usize {
        4 + self.length as usize
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn presentation_context_id(&self) -> u8 {
        self.presentation_context_id
    }

    pub fn message_control_header(&self) -> u8 {
        self.message_control_header
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn is_command(&self) -> bool {
        self.message_control_header & 0b00000001 == 1
    }

    pub fn is_data(&self) -> bool {
        !self.is_command()
    }

    pub fn is_last(&self) -> bool {
        self.message_control_header & 0b00000010 == 2
    }
}

impl TryFrom<&[u8]> for PresentationDataValue {
    type Error = &'static str;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() <= 6 {
            return Err(INVALID_FIELD_LENGTH_ERROR_MESSAGE);
        }

        let length = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        if bytes.len() < (4 + length as usize) {
            return Err("Item-length が不正です");
        }

        let presentation_context_id = bytes[4];
        let message_control_header = bytes[5];
        let data = bytes[6..(4 + length as usize)].to_vec();

        Ok(PresentationDataValue {
            length,
            presentation_context_id,
            message_control_header,
            data,
        })
    }
}
