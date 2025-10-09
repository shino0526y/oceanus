pub mod abstract_syntax;

pub use crate::network::upper_layer_protocol::pdu::a_associate::presentation_context::transfer_syntax::{self, TransferSyntax};
pub use abstract_syntax::AbstractSyntax;

use crate::network::upper_layer_protocol::pdu::{
    ItemType, PduReadError, a_associate::INVALID_ITEM_LENGTH_ERROR_MESSAGE,
};
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};

pub(crate) const ITEM_TYPE: u8 = 0x20;

#[derive(Debug, PartialEq)]
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

    pub fn new(
        context_id: u8,
        abstract_syntax: AbstractSyntax,
        transfer_syntaxes: impl Into<Vec<TransferSyntax>>,
    ) -> Self {
        let transfer_syntaxes = transfer_syntaxes.into();
        let length = 1 // Presentation-context-ID
            + 1 // Reserved
            + 1 // Reserved
            + 1 // Reserved
            + abstract_syntax.size() as u16
            + transfer_syntaxes.iter().map(|ts| ts.size() as u16).sum::<u16>();

        Self {
            length,
            context_id,
            abstract_syntax,
            transfer_syntaxes,
        }
    }

    pub async fn read_from_stream(
        buf_reader: &mut BufReader<impl AsyncRead + Unpin>,
        length: u16,
    ) -> Result<Self, PduReadError> {
        if length < 4 + 4 {
            // Abstract Syntax Sub-Itemまでのフィールドの長さ + Abstract Syntax Sub-Itemのヘッダ（Item-type, Reserved, Item-length）の長さ が全体の長さを超えている場合
            return Err(PduReadError::InvalidPduParameterValue {
                message: INVALID_ITEM_LENGTH_ERROR_MESSAGE.to_string(),
            });
        }

        let mut offset = 0;

        let context_id = buf_reader.read_u8().await?;
        offset += 1;
        buf_reader.read_u8().await?; // Reserved
        offset += 1;
        buf_reader.read_u8().await?; // Reserved
        offset += 1;
        buf_reader.read_u8().await?; // Reserved
        offset += 1;

        let abstract_syntax = {
            let sub_item_type = ItemType::read_from_stream(buf_reader).await?;
            if sub_item_type != ItemType::AbstractSyntaxSubItem {
                return Err(PduReadError::UnexpectedPduParameter(sub_item_type));
            }
            offset += 1;
            buf_reader.read_u8().await?; // Reserved
            offset += 1;
            let sub_item_length = buf_reader.read_u16().await?;
            offset += 2;

            let abstract_syntax = AbstractSyntax::read_from_stream(buf_reader, sub_item_length)
                .await
                .map_err(|e| match e {
                    PduReadError::IoError(_) => e,
                    PduReadError::InvalidPduParameterValue { message } => {
                        PduReadError::InvalidPduParameterValue {
                            message: format!(
                                "Abstract Syntax Sub-Itemのパースに失敗しました: {message}"
                            ),
                        }
                    }
                    _ => panic!(),
                })?;
            offset += abstract_syntax.length() as usize;

            abstract_syntax
        };
        let mut transfer_syntaxes = vec![];
        while offset < length as usize {
            if offset + 4 > length as usize {
                return Err(PduReadError::InvalidPduParameterValue {
                    message: INVALID_ITEM_LENGTH_ERROR_MESSAGE.to_string(),
                });
            }

            let sub_item_type = ItemType::read_from_stream(buf_reader).await?;
            if sub_item_type != ItemType::TransferSyntaxSubItem {
                return Err(PduReadError::UnexpectedPduParameter(sub_item_type));
            }
            offset += 1;
            buf_reader.read_u8().await?; // Reserved
            offset += 1;
            let sub_item_length = buf_reader.read_u16().await?;
            offset += 2;

            let transfer_syntax = TransferSyntax::read_from_stream(buf_reader, sub_item_length)
                .await
                .map_err(|e| match e {
                    PduReadError::IoError(_) => e,
                    PduReadError::InvalidPduParameterValue { message } => {
                        PduReadError::InvalidPduParameterValue {
                            message: format!(
                                "Transfer Syntax Sub-Itemのパースに失敗しました: {message}"
                            ),
                        }
                    }
                    _ => panic!(),
                })?;
            offset += transfer_syntax.length() as usize;

            transfer_syntaxes.push(transfer_syntax);
        }

        if offset != length as usize {
            return Err(PduReadError::InvalidPduParameterValue {
                message: format!(
                    "Item-lengthと実際の読み取りバイト数が一致しません (Item-length={length} 読み取りバイト数={offset})"
                ),
            });
        }

        Ok(Self {
            length,
            context_id,
            abstract_syntax,
            transfer_syntaxes,
        })
    }
}
