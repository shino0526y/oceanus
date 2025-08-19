pub mod abstract_syntax;

pub use crate::network::upper_layer_protocol::pdu::a_associate::presentation_context::transfer_syntax::{self, TransferSyntax};
use crate::network::upper_layer_protocol::pdu::{
    PduReadError,
    a_associate::{INVALID_ITEM_LENGTH_ERROR_MESSAGE, INVALID_ITEM_TYPE_ERROR_MESSAGE},
};
pub use abstract_syntax::AbstractSyntax;

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

    pub async fn read_from_stream(
        buf_reader: &mut tokio::io::BufReader<impl tokio::io::AsyncRead + Unpin>,
        length: u16,
    ) -> Result<Self, PduReadError> {
        use tokio::io::AsyncReadExt;

        if length < 4 + 4 {
            // Abstract Syntax Sub-Itemまでのフィールドの長さ + Abstract Syntax Sub-Itemのヘッダ（Item-type, Reserved, Item-length）の長さ が全体の長さを超えている場合
            return Err(PduReadError::InvalidFormat {
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
            let sub_item_type = buf_reader.read_u8().await?;
            if sub_item_type != abstract_syntax::ITEM_TYPE {
                return Err(PduReadError::InvalidFormat {
                    message: INVALID_ITEM_TYPE_ERROR_MESSAGE.to_string(),
                });
            }
            offset += 1;
            buf_reader.read_u8().await?; // Reserved
            offset += 1;
            let sub_item_length = buf_reader.read_u16().await?;
            offset += 2;

            let abstract_syntax = AbstractSyntax::read_from_stream(buf_reader, sub_item_length)
                .await
                .map_err(|e| PduReadError::InvalidFormat {
                    message: format!("Abstract Syntax Sub-Itemのパースに失敗しました: {e}"),
                })?;
            offset += abstract_syntax.length() as usize;

            abstract_syntax
        };
        let mut transfer_syntaxes = vec![];
        while offset < length as usize {
            if offset + 4 > length as usize {
                return Err(PduReadError::InvalidFormat {
                    message: INVALID_ITEM_LENGTH_ERROR_MESSAGE.to_string(),
                });
            }

            let sub_item_type = buf_reader.read_u8().await?;
            if sub_item_type != transfer_syntax::ITEM_TYPE {
                return Err(PduReadError::InvalidFormat {
                    message: INVALID_ITEM_TYPE_ERROR_MESSAGE.to_string(),
                });
            }
            offset += 1;
            buf_reader.read_u8().await?; // Reserved
            offset += 1;
            let sub_item_length = buf_reader.read_u16().await?;
            offset += 2;

            let transfer_syntax = TransferSyntax::read_from_stream(buf_reader, sub_item_length)
                .await
                .map_err(|e| PduReadError::InvalidFormat {
                    message: format!("Transfer Syntax Sub-Itemのパースに失敗しました: {e}"),
                })?;
            offset += transfer_syntax.length() as usize;

            transfer_syntaxes.push(transfer_syntax);
        }

        if offset != length as usize {
            return Err(PduReadError::InvalidFormat {
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
