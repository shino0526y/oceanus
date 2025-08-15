pub mod reason;
pub mod source;

use crate::{
    errors::StreamParseError, network::upper_layer_protocol::pdu::INVALID_PDU_LENGTH_ERROR_MESSAGE,
};
pub use reason::Reason;
pub use source::Source;

pub(crate) const PDU_TYPE: u8 = 0x07;

pub struct AAbort {
    source: Source,
    reason: Reason,
}

impl AAbort {
    pub fn size(&self) -> usize {
        10
    }

    pub fn length(&self) -> u32 {
        4
    }

    pub fn source(&self) -> Source {
        self.source
    }

    pub fn reason(&self) -> Reason {
        self.reason
    }

    pub fn new(source: Source, reason: Reason) -> Self {
        Self { source, reason }
    }

    pub async fn read_from_stream(
        buf_reader: &mut tokio::io::BufReader<impl tokio::io::AsyncRead + Unpin>,
        length: u32,
    ) -> Result<Self, StreamParseError> {
        use tokio::io::AsyncReadExt;

        if length != 4 {
            return Err(StreamParseError::InvalidFormat {
                message: INVALID_PDU_LENGTH_ERROR_MESSAGE.to_string(),
            });
        }

        buf_reader.read_u8().await?; // Reserved
        buf_reader.read_u8().await?; // Reserved
        let source = Source::try_from(buf_reader.read_u8().await?).map_err(|e| {
            StreamParseError::InvalidFormat {
                message: format!("Source の変換に失敗しました: {e}"),
            }
        })?;
        let reason = Reason::try_from(buf_reader.read_u8().await?).map_err(|e| {
            StreamParseError::InvalidFormat {
                message: format!("Reason/Diag. の変換に失敗しました: {e}"),
            }
        })?;

        Ok(Self { source, reason })
    }
}

#[rustfmt::skip]
impl From<AAbort> for Vec<u8> {
    fn from(val: AAbort) -> Self {
        vec![
            PDU_TYPE,
            0,          // Reserved
            0, 0, 0, 4, // PDU-length
            0,          // Reserved
            0,          // Reserved
            val.source as u8,
            val.reason as u8,
        ]
    }
}
