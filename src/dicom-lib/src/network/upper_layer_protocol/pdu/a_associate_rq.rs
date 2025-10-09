pub mod presentation_context;

pub use crate::network::upper_layer_protocol::pdu::a_associate::*;
pub use presentation_context::PresentationContext;

use crate::network::upper_layer_protocol::pdu::{
    INVALID_PDU_LENGTH_ERROR_MESSAGE, ItemType, PduReadError,
};
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};

pub(crate) const PDU_TYPE: u8 = 0x01;

#[derive(Debug, PartialEq)]
pub struct AAssociateRq {
    length: u32,
    version: u16,
    called_ae_title: String,
    calling_ae_title: String,
    application_context: ApplicationContext,
    presentation_contexts: Vec<PresentationContext>,
    user_information: UserInformation,
}

impl AAssociateRq {
    pub fn size(&self) -> usize {
        6 + self.length as usize
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn version(&self) -> u16 {
        self.version
    }

    pub fn called_ae_title(&self) -> &str {
        &self.called_ae_title
    }

    pub fn calling_ae_title(&self) -> &str {
        &self.calling_ae_title
    }

    pub fn application_context(&self) -> &ApplicationContext {
        &self.application_context
    }

    pub fn presentation_contexts(&self) -> &[PresentationContext] {
        &self.presentation_contexts
    }

    pub fn user_information(&self) -> &UserInformation {
        &self.user_information
    }

    pub fn new(
        version: u16,
        called_ae_title: impl Into<String>,
        calling_ae_title: impl Into<String>,
        application_context: ApplicationContext,
        presentation_contexts: impl Into<Vec<PresentationContext>>,
        user_information: UserInformation,
    ) -> Result<Self, &'static str> {
        let called_ae_title = called_ae_title.into();
        if called_ae_title.is_empty() {
            return Err("Called-AE-titleが空です");
        } else if called_ae_title.len() > 16 {
            return Err("Called-AE-titleの長さが16バイトを超えています");
        }
        let calling_ae_title = calling_ae_title.into();
        if calling_ae_title.is_empty() {
            return Err("Calling-AE-titleが空です");
        } else if calling_ae_title.len() > 16 {
            return Err("Calling-AE-titleの長さが16バイトを超えています");
        }

        let presentation_contexts = presentation_contexts.into();
        let length = 2 // Protocol-version
            + 2 // Reserved
            + 16 // Called-AE-title
            + 16 // Calling-AE-title
            + 32 // Reserved
            + application_context.length() as u32
            + presentation_contexts
                .iter()
                .map(|pc| pc.length() as u32)
                .sum::<u32>()
            + user_information.length() as u32;

        Ok(Self {
            length,
            version,
            called_ae_title,
            calling_ae_title,
            application_context,
            presentation_contexts,
            user_information,
        })
    }

    pub async fn read_from_stream(
        buf_reader: &mut BufReader<impl AsyncRead + Unpin>,
        length: u32,
    ) -> Result<Self, PduReadError> {
        if length < 68 + 4 {
            // Application Context Itemまでのフィールドの長さ + Application Context Itemのヘッダ（Item-type, Reserved, Item-length）の長さ が全体の長さを超えている場合
            return Err(PduReadError::InvalidPduParameterValue {
                message: INVALID_PDU_LENGTH_ERROR_MESSAGE.to_string(),
            });
        }

        let mut offset = 0;

        let version = buf_reader.read_u16().await?;
        offset += 2;
        buf_reader.read_u16().await?; // Reserved
        offset += 2;
        let called_ae_title = {
            let mut buf = [0u8; 16];
            buf_reader.read_exact(&mut buf).await?;
            std::str::from_utf8(&buf)
                .map_err(|_| PduReadError::InvalidPduParameterValue {
                    message: "Called-AE-titleフィールドをUTF-8の文字列として解釈できません"
                        .to_string(),
                })?
                .trim_end_matches(' ')
                .trim_start_matches(' ')
                .to_string()
        };
        offset += 16;
        let calling_ae_title = {
            let mut buf = [0u8; 16];
            buf_reader.read_exact(&mut buf).await?;
            std::str::from_utf8(&buf)
                .map_err(|_| PduReadError::InvalidPduParameterValue {
                    message: "Calling-AE-titleフィールドをUTF-8の文字列として解釈できません"
                        .to_string(),
                })?
                .trim_end_matches(' ')
                .trim_start_matches(' ')
                .to_string()
        };
        offset += 16;
        {
            let mut buf = [0u8; 32];
            buf_reader.read_exact(&mut buf).await?; // Reserved
        };
        offset += 32;

        let application_context = {
            let item_type = ItemType::read_from_stream(buf_reader).await?;
            if item_type != ItemType::ApplicationContextItem {
                return Err(PduReadError::UnexpectedPduParameter(item_type));
            }
            offset += 1;
            buf_reader.read_u8().await?; // Reserved
            offset += 1;
            let item_length = buf_reader.read_u16().await?;
            offset += 2;

            if offset + item_length as usize > length as usize {
                return Err(PduReadError::InvalidPduParameterValue {
                    message: INVALID_PDU_LENGTH_ERROR_MESSAGE.to_string(),
                });
            }

            let application_context = ApplicationContext::read_from_stream(buf_reader, item_length)
                .await
                .map_err(|e| match e {
                    PduReadError::IoError(_) => e,
                    PduReadError::InvalidPduParameterValue { message } => {
                        PduReadError::InvalidPduParameterValue {
                            message: format!(
                                "Application Context Itemのパースに失敗しました: {message}"
                            ),
                        }
                    }
                    _ => panic!(),
                })?;
            offset += application_context.length() as usize;

            application_context
        };
        let mut presentation_contexts = vec![];
        let mut user_information = None;
        while offset + 4 < length as usize {
            let item_type = ItemType::read_from_stream(buf_reader).await?;
            offset += 1;
            buf_reader.read_u8().await?; // Reserved
            offset += 1;
            let item_length = buf_reader.read_u16().await?;
            offset += 2;

            if offset + item_length as usize > length as usize {
                return Err(PduReadError::InvalidPduParameterValue {
                    message: INVALID_PDU_LENGTH_ERROR_MESSAGE.to_string(),
                });
            }

            match item_type {
                ItemType::PresentationContextItemInAAssociateRq => {
                    let presentation_context = PresentationContext::read_from_stream(
                        buf_reader,
                        item_length,
                    )
                    .await
                    .map_err(|e| match e {
                        PduReadError::IoError(_) => e,
                        PduReadError::InvalidPduParameterValue { message } => {
                            PduReadError::InvalidPduParameterValue {
                                message: format!(
                                    "Presentation Context Itemのパースに失敗しました: {message}"
                                ),
                            }
                        }
                        _ => panic!(),
                    })?;
                    offset += presentation_context.length() as usize;

                    presentation_contexts.push(presentation_context);
                }
                ItemType::UserInformationItem => {
                    if presentation_contexts.is_empty() {
                        // Presentation Context Itemが存在しないのにUser Information Itemが出現した場合
                        return Err(PduReadError::UnexpectedPduParameter(item_type));
                    }

                    let temp_user_information =
                        UserInformation::read_from_stream(buf_reader, item_length)
                            .await
                            .map_err(|e| match e {
                                PduReadError::IoError(_) => e,
                                PduReadError::InvalidPduParameterValue { message } => {
                                    PduReadError::InvalidPduParameterValue {
                                        message: format!(
                                            "User Information Itemのパースに失敗しました: {message}"
                                        ),
                                    }
                                }
                                _ => panic!(),
                            })?;
                    offset += temp_user_information.length() as usize;

                    user_information = Some(temp_user_information);
                    break;
                }
                _ => return Err(PduReadError::UnexpectedPduParameter(item_type)),
            }
        }

        if offset != length as usize {
            return Err(PduReadError::InvalidPduParameterValue {
                message: format!(
                    "PDU-lengthと実際の読み取りバイト数が一致しません (PDU-length={length} 読み取りバイト数={offset})"
                ),
            });
        }

        let user_information =
            user_information.ok_or_else(|| PduReadError::InvalidPduParameterValue {
                message: "User Information Itemが存在しません".to_string(),
            })?;

        Ok(Self {
            length,
            version,
            called_ae_title,
            calling_ae_title,
            application_context,
            presentation_contexts,
            user_information,
        })
    }
}
