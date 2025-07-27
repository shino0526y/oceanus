use crate::dicom::network::pdu::a_associate::items::{INVALID_ITEM_TYPE_ERROR_MESSAGE, Item};

pub const ITEM_TYPE: u8 = 0x40;

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
}

impl TryFrom<&[u8]> for TransferSyntax {
    type Error = &'static str;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let item = Item::try_from(bytes)?;
        if item.item_type != ITEM_TYPE {
            return Err(INVALID_ITEM_TYPE_ERROR_MESSAGE);
        }

        let name = std::str::from_utf8(&item.data)
            .map_err(|_| "Transfer-syntax-name(s) フィールドを UTF-8 の文字列として解釈できません")?
            .trim_end_matches('\0')
            .to_string();

        Ok(TransferSyntax {
            length: item.length,
            name,
        })
    }
}
