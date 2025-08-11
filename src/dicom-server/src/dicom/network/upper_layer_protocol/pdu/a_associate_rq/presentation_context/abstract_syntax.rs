use crate::dicom::network::upper_layer_protocol::pdu::a_associate::{
    INVALID_ITEM_TYPE_ERROR_MESSAGE, Item,
};

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
}

impl TryFrom<&[u8]> for AbstractSyntax {
    type Error = &'static str;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let item = Item::try_from(bytes)?;
        if item.item_type != ITEM_TYPE {
            return Err(INVALID_ITEM_TYPE_ERROR_MESSAGE);
        }

        let name = std::str::from_utf8(item.data)
            .map_err(|_| "Abstract-syntax-name フィールドを UTF-8 の文字列として解釈できません")?
            .trim_end_matches('\0')
            .to_string();

        Ok(AbstractSyntax {
            length: item.length,
            name,
        })
    }
}
