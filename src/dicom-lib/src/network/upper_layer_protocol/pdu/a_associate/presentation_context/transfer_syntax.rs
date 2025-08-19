use crate::network::upper_layer_protocol::pdu::PduReadError;

pub(crate) const ITEM_TYPE: u8 = 0x40;

pub struct TransferSyntax {
    length: u16,
    name: String,
}

impl TransferSyntax {
    pub fn size(&self) -> usize {
        4 + self.length as usize
    }

    pub fn length(&self) -> u16 {
        self.length
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn new<T: Into<String>>(name: T) -> Result<Self, &'static str> {
        let name = name.into();
        if name.is_empty() {
            return Err("Transfer-syntax-nameが空です");
        }

        let mut length = name.len() as u16;
        if name.len() % 2 != 0 {
            length += 1;
        }

        Ok(Self { length, name })
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
                    message: "Transfer-syntax-nameフィールドをUTF-8の文字列として解釈できません"
                        .to_string(),
                })?
                .trim_end_matches('\0')
                .to_string()
        };

        Ok(Self { length, name })
    }
}

impl From<TransferSyntax> for Vec<u8> {
    fn from(val: TransferSyntax) -> Self {
        let mut bytes = Vec::with_capacity(val.size());

        bytes.push(ITEM_TYPE);
        bytes.push(0); // Reserved
        bytes.extend(val.length.to_be_bytes());
        bytes.extend(val.name.as_bytes());
        if val.name.len() % 2 != 0 {
            bytes.push(b'\0');
        }

        bytes
    }
}
