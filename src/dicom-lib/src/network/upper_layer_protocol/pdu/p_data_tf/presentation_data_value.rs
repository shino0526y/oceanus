use crate::network::upper_layer_protocol::pdu::{
    PduReadError, a_associate::INVALID_ITEM_LENGTH_ERROR_MESSAGE,
};

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

    pub fn new(presentation_context_id: u8, is_command: bool, is_last: bool, data: &[u8]) -> Self {
        let length = data.len() as u32
            + 1 // Presentation Context ID
            + 1; // Message Control Header
        let mut message_control_header = 0;
        if is_command {
            message_control_header |= 0b00000001;
        }
        if is_last {
            message_control_header |= 0b00000010;
        }
        let data = data.to_vec();

        Self {
            length,
            presentation_context_id,
            message_control_header,
            data,
        }
    }

    pub async fn read_from_stream(
        buf_reader: &mut tokio::io::BufReader<impl tokio::io::AsyncRead + Unpin>,
        length: u32,
    ) -> Result<Self, PduReadError> {
        use tokio::io::AsyncReadExt;

        const SIZE_OF_PRESENTATION_CONTEXT_ID: usize = 1;
        const SIZE_OF_MESSAGE_CONTROL_HEADER: usize = 1;
        if (length as usize) < SIZE_OF_PRESENTATION_CONTEXT_ID + SIZE_OF_MESSAGE_CONTROL_HEADER {
            return Err(PduReadError::InvalidPduParameterValue {
                message: INVALID_ITEM_LENGTH_ERROR_MESSAGE.to_string(),
            });
        }

        let presentation_context_id = buf_reader.read_u8().await?;
        let message_control_header = buf_reader.read_u8().await?;
        let mut data = vec![0; (length - 2) as usize];
        buf_reader.read_exact(&mut data).await?;
        Ok(Self {
            length,
            presentation_context_id,
            message_control_header,
            data,
        })
    }
}

impl From<&PresentationDataValue> for Vec<u8> {
    fn from(val: &PresentationDataValue) -> Self {
        let mut bytes = Vec::with_capacity(val.size());

        bytes.extend(val.length().to_be_bytes());
        bytes.push(val.presentation_context_id());
        bytes.push(val.message_control_header());
        bytes.extend(val.data());

        bytes
    }
}
