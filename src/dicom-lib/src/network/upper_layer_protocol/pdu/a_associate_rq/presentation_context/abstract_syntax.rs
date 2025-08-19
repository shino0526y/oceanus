use crate::network::upper_layer_protocol::pdu::PduReadError;

pub(crate) const ITEM_TYPE: u8 = 0x30;

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

    pub async fn read_from_stream(
        buf_reader: &mut tokio::io::BufReader<impl tokio::io::AsyncRead + Unpin>,
        length: u16,
    ) -> Result<Self, PduReadError> {
        use tokio::io::AsyncReadExt;

        let name = {
            let mut buf = vec![0u8; length as usize];
            buf_reader.read_exact(&mut buf).await?;
            std::str::from_utf8(&buf)
                .map_err(|_| PduReadError::InvalidFormat {
                    message: "Abstract-syntax-nameフィールドをUTF-8の文字列として解釈できません"
                        .to_string(),
                })?
                .trim_end_matches('\0')
                .to_string()
        };

        Ok(Self { length, name })
    }
}
