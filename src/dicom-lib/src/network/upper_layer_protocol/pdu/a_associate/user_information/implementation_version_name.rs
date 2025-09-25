use crate::network::upper_layer_protocol::pdu::PduReadError;
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};

pub(crate) const ITEM_TYPE: u8 = 0x55;

pub struct ImplementationVersionName {
    length: u16,
    name: String,
}

impl ImplementationVersionName {
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
        if name.is_empty() || name.len() > 16 {
            return Err("Implementation-version-nameは1文字以上16文字以下でなければなりません");
        }
        if !name.is_ascii() {
            return Err(
                "Implementation-version-nameはISO 646:1990 (basic G0 set)でエンコーディングされている必要があります",
            );
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

            std::str::from_utf8(&buf)
                .map_err(|_| PduReadError::InvalidPduParameterValue {
                    message:
                        "Implementation-class-nameフィールドをUTF-8の文字列として解釈できません"
                            .to_string(),
                })?
                .to_string()
        };

        Ok(Self { length, name })
    }
}

impl From<ImplementationVersionName> for Vec<u8> {
    fn from(val: ImplementationVersionName) -> Self {
        let mut bytes = Vec::with_capacity(val.size());

        bytes.push(ITEM_TYPE);
        bytes.push(0); // Reserved
        bytes.extend(val.length.to_be_bytes());
        bytes.extend(val.name.as_bytes());

        bytes
    }
}
