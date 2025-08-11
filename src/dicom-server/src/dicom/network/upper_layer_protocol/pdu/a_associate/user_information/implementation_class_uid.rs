use crate::dicom::network::upper_layer_protocol::pdu::a_associate::{
    INVALID_ITEM_TYPE_ERROR_MESSAGE, Item,
};

pub(crate) const ITEM_TYPE: u8 = 0x52;

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

    pub fn new<T: Into<String>>(uid: T) -> Result<Self, &'static str> {
        let uid = uid.into();
        if uid.is_empty() {
            return Err("Implementation-class-uid が空です");
        }

        let mut length = uid.len() as u16;
        if uid.len() % 2 != 0 {
            length += 1;
        }

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

impl TryFrom<&[u8]> for ImplementationClassUid {
    type Error = &'static str;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let item = Item::try_from(bytes)?;
        if item.item_type != ITEM_TYPE {
            return Err(INVALID_ITEM_TYPE_ERROR_MESSAGE);
        }

        let uid = std::str::from_utf8(item.data)
            .map_err(
                |_| "Implementation-class-uid フィールドを UTF-8 の文字列として解釈できません",
            )?
            .trim_end_matches('\0')
            .to_string();

        Ok(ImplementationClassUid {
            length: item.length,
            uid,
        })
    }
}
