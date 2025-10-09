use crate::network::upper_layer_protocol::pdu::PduReadError;
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};

pub(crate) const ITEM_TYPE: u8 = 0x52;

#[derive(Debug, PartialEq)]
pub struct ImplementationClassUid {
    length: u16,
    uid: String,
}

impl ImplementationClassUid {
    pub fn size(&self) -> usize {
        4 + self.length as usize
    }

    pub fn length(&self) -> u16 {
        self.length
    }

    pub fn uid(&self) -> &str {
        &self.uid
    }

    pub fn new(uid: impl Into<String>) -> Result<Self, &'static str> {
        let uid = uid.into();
        if uid.is_empty() {
            return Err("Implementation-class-uidが空です");
        }

        let mut length = uid.len() as u16;
        if uid.len() % 2 != 0 {
            length += 1;
        }

        Ok(Self { length, uid })
    }

    pub async fn read_from_stream(
        buf_reader: &mut BufReader<impl AsyncRead + Unpin>,
        length: u16,
    ) -> Result<Self, PduReadError> {
        let uid = {
            let mut buf = vec![0u8; length as usize];
            buf_reader.read_exact(&mut buf).await?;

            std::str::from_utf8(&buf)
                .map_err(|_| PduReadError::InvalidPduParameterValue {
                    message:
                        "Implementation-class-uidフィールドをUTF-8の文字列として解釈できません"
                            .to_string(),
                })?
                .trim_end_matches('\0')
                .to_string()
        };

        Ok(Self { length, uid })
    }
}

impl From<ImplementationClassUid> for Vec<u8> {
    fn from(val: ImplementationClassUid) -> Self {
        let mut bytes = Vec::with_capacity(val.size());

        bytes.push(ITEM_TYPE);
        bytes.push(0); // Reserved
        bytes.extend(val.length.to_be_bytes());
        bytes.extend(val.uid.as_bytes());
        if val.uid.len() % 2 != 0 {
            bytes.push(b'\0');
        }

        bytes
    }
}
