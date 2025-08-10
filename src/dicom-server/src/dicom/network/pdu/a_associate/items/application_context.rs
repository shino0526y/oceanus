use crate::dicom::network::pdu::a_associate::items::{INVALID_ITEM_TYPE_ERROR_MESSAGE, Item};

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

        ApplicationContext { length, name }
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

impl TryFrom<&[u8]> for ApplicationContext {
    type Error = &'static str;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let item = Item::try_from(bytes)?;
        if item.item_type != ITEM_TYPE {
            return Err(INVALID_ITEM_TYPE_ERROR_MESSAGE);
        }

        let name = std::str::from_utf8(item.data)
            .map_err(
                |_| "Application-context-name フィールドを UTF-8 の文字列として解釈できません",
            )?
            .trim_end_matches('\0')
            .to_string();

        Ok(ApplicationContext {
            length: item.length,
            name,
        })
    }
}
