use crate::network::upper_layer_protocol::pdu::{
    PduReadError, a_associate::INVALID_ITEM_LENGTH_ERROR_MESSAGE,
};

pub(crate) const ITEM_TYPE: u8 = 0x51;

pub struct MaximumLength {
    maximum_length: u32,
}

impl MaximumLength {
    pub fn size(&self) -> usize {
        8
    }

    pub fn length(&self) -> u16 {
        4
    }

    pub fn maximum_length(&self) -> u32 {
        self.maximum_length
    }

    pub fn new(maximum_length: u32) -> Self {
        Self { maximum_length }
    }

    pub async fn read_from_stream(
        buf_reader: &mut tokio::io::BufReader<impl tokio::io::AsyncRead + Unpin>,
        length: u16,
    ) -> Result<Self, PduReadError> {
        use tokio::io::AsyncReadExt;

        if length != 4 {
            return Err(PduReadError::InvalidFormat {
                message: INVALID_ITEM_LENGTH_ERROR_MESSAGE.to_string(),
            });
        }

        let maximum_length = buf_reader.read_u32().await?;

        Ok(Self { maximum_length })
    }
}

impl From<MaximumLength> for Vec<u8> {
    fn from(val: MaximumLength) -> Self {
        let mut bytes = Vec::with_capacity(val.size());

        bytes.push(ITEM_TYPE);
        bytes.push(0); // Reserved
        bytes.extend(4u16.to_be_bytes());
        bytes.extend(val.maximum_length.to_be_bytes());

        bytes
    }
}
