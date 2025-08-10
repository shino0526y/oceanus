pub mod sub_items;

pub(crate) const ITEM_TYPE: u8 = 0x21;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ResultReason {
    Acceptance = 0,
    UserRejection = 1,
    NoReason = 2,
    AbstractSyntaxNotSupported = 3,
    TransferSyntaxesNotSupported = 4,
}

pub struct PresentationContext {
    length: u16,
    context_id: u8,
    result_reason: ResultReason,
    transfer_syntax: sub_items::TransferSyntax,
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

    pub fn result_reason(&self) -> ResultReason {
        self.result_reason
    }

    pub fn transfer_syntax(&self) -> &sub_items::TransferSyntax {
        &self.transfer_syntax
    }

    pub fn new(
        context_id: u8,
        result_reason: ResultReason,
        transfer_syntax: sub_items::TransferSyntax,
    ) -> Self {
        let length = 1 // Presentation-context-ID
            + 1 // Reserved
            + 1 // Result/Reason
            + 1 // Reserved
            + transfer_syntax.size() as u16;

        PresentationContext {
            length,
            context_id,
            result_reason,
            transfer_syntax,
        }
    }
}

impl From<PresentationContext> for Vec<u8> {
    fn from(val: PresentationContext) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(val.size());

        bytes.push(ITEM_TYPE);
        bytes.push(0); // Reserved
        bytes.extend(val.length.to_be_bytes());
        bytes.push(val.context_id);
        bytes.push(0); // Reserved
        bytes.push(val.result_reason as u8);
        bytes.push(0); // Reserved
        bytes.append(&mut val.transfer_syntax.into());

        bytes
    }
}
