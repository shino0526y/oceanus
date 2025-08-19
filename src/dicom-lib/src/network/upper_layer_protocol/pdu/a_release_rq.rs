use crate::network::upper_layer_protocol::pdu::{INVALID_PDU_LENGTH_ERROR_MESSAGE, PduReadError};

pub(crate) const PDU_TYPE: u8 = 0x05;

pub struct AReleaseRq();

impl AReleaseRq {
    pub fn size(&self) -> usize {
        10
    }

    pub fn length(&self) -> u32 {
        4
    }

    pub async fn read_from_stream(
        buf_reader: &mut tokio::io::BufReader<impl tokio::io::AsyncRead + Unpin>,
        length: u32,
    ) -> Result<Self, PduReadError> {
        use tokio::io::AsyncReadExt;

        if length != 4 {
            return Err(PduReadError::InvalidFormat {
                message: INVALID_PDU_LENGTH_ERROR_MESSAGE.to_string(),
            });
        }

        let mut buf = [0u8; 4];
        buf_reader.read_exact(&mut buf).await?; // Reserved

        Ok(Self())
    }
}
