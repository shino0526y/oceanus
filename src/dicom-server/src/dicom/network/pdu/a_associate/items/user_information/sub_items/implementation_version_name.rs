use crate::dicom::network::pdu::a_associate::items::{INVALID_ITEM_TYPE_ERROR_MESSAGE, Item};

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
            return Err(
                "Implementation-version-name は 1 文字以上 16 文字以下でなければなりません",
            );
        }
        if !name.is_ascii() {
            return Err(
                "Implementation-version-name は ISO 646:1990 (basic G0 set) でエンコーディングされている必要があります",
            );
        }

        let length = name.len() as u16;

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

        if val.name.len() % 2 != 0 {
            bytes.push(b'\0');
        }

        bytes
    }
}

impl TryFrom<&[u8]> for ImplementationVersionName {
    type Error = &'static str;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let item = Item::try_from(bytes)?;
        if item.item_type != ITEM_TYPE {
            return Err(INVALID_ITEM_TYPE_ERROR_MESSAGE);
        }

        let name = std::str::from_utf8(item.data)
            .map_err(
                |_| "Implementation-version-name フィールドを UTF-8 の文字列として解釈できません",
            )?
            .to_string();

        Ok(ImplementationVersionName {
            length: item.length,
            name,
        })
    }
}
