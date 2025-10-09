use crate::network::upper_layer_protocol::pdu::{INVALID_PDU_LENGTH_ERROR_MESSAGE, PduReadError};
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};

pub(crate) const PDU_TYPE: u8 = 0x05;

#[derive(Debug, PartialEq)]
pub struct AReleaseRq();

impl AReleaseRq {
    pub fn size(&self) -> usize {
        10
    }

    pub fn length(&self) -> u32 {
        4
    }

    pub fn new() -> Self {
        Self()
    }

    pub async fn read_from_stream(
        buf_reader: &mut BufReader<impl AsyncRead + Unpin>,
        length: u32,
    ) -> Result<Self, PduReadError> {
        if length != 4 {
            return Err(PduReadError::InvalidPduParameterValue {
                message: INVALID_PDU_LENGTH_ERROR_MESSAGE.to_string(),
            });
        }

        let mut buf = [0u8; 4];
        buf_reader.read_exact(&mut buf).await?; // Reserved

        Ok(Self())
    }
}
