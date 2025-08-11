use crate::dicom::network::upper_layer_protocol::pdu::a_associate::{
    INVALID_ITEM_TYPE_ERROR_MESSAGE, Item,
};

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
            return Err("Transfer-syntax-name が空です");
        }

        let mut length = name.len() as u16;
        if name.len() % 2 != 0 {
            length += 1;
        }

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

impl TryFrom<&[u8]> for TransferSyntax {
    type Error = &'static str;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let item = Item::try_from(bytes)?;
        if item.item_type != ITEM_TYPE {
            return Err(INVALID_ITEM_TYPE_ERROR_MESSAGE);
        }

        let name = std::str::from_utf8(item.data)
            .map_err(|_| "Transfer-syntax-name(s) フィールドを UTF-8 の文字列として解釈できません")?
            .trim_end_matches('\0')
            .to_string();

        Ok(TransferSyntax {
            length: item.length,
            name,
        })
    }
}
