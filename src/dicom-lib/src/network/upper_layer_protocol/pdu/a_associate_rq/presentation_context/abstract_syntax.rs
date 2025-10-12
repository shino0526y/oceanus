use crate::network::upper_layer_protocol::pdu::PduReadError;
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};

pub(crate) const ITEM_TYPE: u8 = 0x30;

#[derive(Debug, PartialEq)]
pub struct AbstractSyntax {
    length: u16,
    name: String,
}

impl AbstractSyntax {
    pub fn size(&self) -> usize {
        4 + self.length as usize
    }

    pub fn length(&self) -> u16 {
        self.length
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn new(name: impl Into<String>) -> Result<Self, &'static str> {
        let name = name.into();
        if name.is_empty() {
            return Err("Abstract-syntax-nameが空です");
        }
        let length = name.len() as u16;

        Ok(Self { length, name })
    }

    pub async fn read_from_stream(
        buf_reader: &mut BufReader<impl AsyncRead + Unpin>,
        length: u16,
    ) -> Result<Self, PduReadError> {
        let name = {
            let mut buf = vec![0u8; length as usize];
            buf_reader.read_exact(&mut buf).await?;
            String::from_utf8(buf).map_err(|_| PduReadError::InvalidPduParameterValue {
                message: "Abstract-syntax-nameフィールドをUTF-8の文字列として解釈できません"
                    .to_string(),
            })?
        };

        Ok(Self { length, name })
    }
}
