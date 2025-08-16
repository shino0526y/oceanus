use crate::errors::StreamParseError;

pub(crate) const ITEM_TYPE: u8 = 0x10;

pub struct ApplicationContext {
    length: u16,
    name: String,
}

impl ApplicationContext {
    pub fn size(&self) -> usize {
        4 + self.length as usize
    }

    pub fn length(&self) -> u16 {
        self.length
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn new<T: Into<String>>(name: T) -> Self {
        let name = name.into();
        let mut length = name.len() as u16;
        if name.len() % 2 != 0 {
            length += 1;
        }

        Self { length, name }
    }

    pub async fn read_from_stream(
        buf_reader: &mut tokio::io::BufReader<impl tokio::io::AsyncRead + Unpin>,
        length: u16,
    ) -> Result<Self, StreamParseError> {
        use tokio::io::AsyncReadExt;

        let name = {
            let mut buf = vec![0u8; length as usize];
            buf_reader.read_exact(&mut buf).await?;
            std::str::from_utf8(&buf)
                .map_err(|_| StreamParseError::InvalidFormat {
                    message:
                        "Application-context-nameフィールドをUTF-8の文字列として解釈できません"
                            .to_string(),
                })?
                .trim_end_matches('\0')
                .to_string()
        };

        Ok(Self { length, name })
    }
}

impl From<ApplicationContext> for Vec<u8> {
    fn from(val: ApplicationContext) -> Self {
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
