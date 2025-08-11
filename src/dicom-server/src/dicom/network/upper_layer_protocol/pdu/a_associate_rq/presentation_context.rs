pub mod abstract_syntax;

pub use crate::dicom::network::upper_layer_protocol::pdu::a_associate::presentation_context::transfer_syntax::{self, TransferSyntax};
pub use abstract_syntax::AbstractSyntax;
use crate::dicom::network::upper_layer_protocol::pdu::a_associate::{
    INVALID_ITEM_TYPE_ERROR_MESSAGE, Item,
};

pub(crate) const ITEM_TYPE: u8 = 0x20;

pub struct PresentationContext {
    length: u16,
    context_id: u8,
    abstract_syntax: AbstractSyntax,
    transfer_syntaxes: Vec<TransferSyntax>,
}

impl PresentationContext {
    pub fn size(&self) -> usize {
        4 + self.length as usize
    }

    pub fn length(&self) -> u16 {
        self.length
    }

    pub fn context_id(&self) -> u8 {
        self.context_id
    }

    pub fn abstract_syntax(&self) -> &AbstractSyntax {
        &self.abstract_syntax
    }

    pub fn transfer_syntaxes(&self) -> &[TransferSyntax] {
        &self.transfer_syntaxes
    }
}

impl TryFrom<&[u8]> for PresentationContext {
    type Error = String;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let item = Item::try_from(bytes)?;
        if item.item_type != ITEM_TYPE {
            return Err(INVALID_ITEM_TYPE_ERROR_MESSAGE.to_string());
        }

        let context_id = item.data[0];

        let mut offset = 4;
        let abstract_syntax =
            AbstractSyntax::try_from(&item.data[offset..]).map_err(|message| {
                format!("Abstract Syntax Sub-Item のパースに失敗しました: {message}")
            })?;
        offset += abstract_syntax.size();

        let mut transfer_syntaxes = vec![];
        while offset < item.data.len() {
            let transfer_syntax =
                TransferSyntax::try_from(&item.data[offset..]).map_err(|message| {
                    format!("Transfer Syntax Sub-Item のパースに失敗しました: {message}")
                })?;
            offset += transfer_syntax.size();
            transfer_syntaxes.push(transfer_syntax);
        }

        Ok(PresentationContext {
            length: item.length,
            context_id,
            abstract_syntax,
            transfer_syntaxes,
        })
    }
}
