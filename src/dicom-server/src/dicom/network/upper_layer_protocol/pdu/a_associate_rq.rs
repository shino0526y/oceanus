pub mod presentation_context;

pub use crate::dicom::network::upper_layer_protocol::pdu::a_associate::*;
use crate::dicom::{
    errors::StreamParseError,
    network::upper_layer_protocol::pdu::{INVALID_PDU_LENGTH_ERROR_MESSAGE, a_associate},
};
pub use presentation_context::PresentationContext;

pub const PDU_TYPE: u8 = 0x01;

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

    pub async fn read_from_stream(
        buf_reader: &mut tokio::io::BufReader<impl tokio::io::AsyncRead + Unpin>,
        length: u32,
    ) -> Result<Self, StreamParseError> {
        use tokio::io::AsyncReadExt;

        if length < 68 + 4
        // Application Context Itemまでのフィールドの長さ + Application Context Itemのヘッダ（Item-type, Reserved, Item-length）の長さ
        {
            return Err(StreamParseError::InvalidFormat {
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
                .map_err(|_| StreamParseError::InvalidFormat {
                    message: "Called-AE-title フィールドを UTF-8 の文字列として解釈できません"
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
                .map_err(|_| StreamParseError::InvalidFormat {
                    message: "Calling-AE-title フィールドを UTF-8 の文字列として解釈できません"
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
            let item_type = buf_reader.read_u8().await?;
            if item_type != a_associate::application_context::ITEM_TYPE {
                return Err(StreamParseError::InvalidFormat {
                    message: "Application Context Item が存在しません".to_string(),
                });
            }
            offset += 1;
            buf_reader.read_u8().await?; // Reserved
            offset += 1;
            let item_length = buf_reader.read_u16().await?;
            offset += 2;

            if offset + item_length as usize > length as usize {
                return Err(StreamParseError::InvalidFormat {
                    message: INVALID_PDU_LENGTH_ERROR_MESSAGE.to_string(),
                });
            }

            let application_context = ApplicationContext::read_from_stream(buf_reader, item_length)
                .await
                .map_err(|e| StreamParseError::InvalidFormat {
                    message: format!("Application Context Item のパースに失敗しました: {e}"),
                })?;
            offset += application_context.length() as usize;

            application_context
        };
        let mut presentation_contexts = vec![];
        let mut user_information = None;
        while offset + 4 < length as usize {
            let item_type = buf_reader.read_u8().await?;
            offset += 1;
            buf_reader.read_u8().await?; // Reserved
            offset += 1;
            let item_length = buf_reader.read_u16().await?;
            offset += 2;

            if offset + item_length as usize > length as usize {
                return Err(StreamParseError::InvalidFormat {
                    message: INVALID_PDU_LENGTH_ERROR_MESSAGE.to_string(),
                });
            }

            match item_type {
                presentation_context::ITEM_TYPE => {
                    let presentation_context =
                        PresentationContext::read_from_stream(buf_reader, item_length)
                            .await
                            .map_err(|e| StreamParseError::InvalidFormat {
                                message: format!(
                                    "Presentation Context Item のパースに失敗しました: {e}"
                                ),
                            })?;
                    offset += presentation_context.length() as usize;

                    presentation_contexts.push(presentation_context);
                }
                user_information::ITEM_TYPE => {
                    if presentation_contexts.is_empty() {
                        return Err(StreamParseError::InvalidFormat {
                            message: "Presentation Context Item が存在しません".to_string(),
                        });
                    }

                    let temp_user_information =
                        UserInformation::read_from_stream(buf_reader, item_length)
                            .await
                            .map_err(|e| StreamParseError::InvalidFormat {
                                message: format!(
                                    "User Information Item のパースに失敗しました: {e}"
                                ),
                            })?;
                    offset += temp_user_information.length() as usize;

                    user_information = Some(temp_user_information);
                    break;
                }
                _ => {
                    return Err(StreamParseError::InvalidFormat {
                        message: format!(
                            "Presentation Context Item もしくは User Information Item のパースを試みようとした際に予期しない Item-type (0x{item_type:02X}) を持つ Item が出現しました"
                        ),
                    });
                }
            }
        }

        if offset != length as usize {
            return Err(StreamParseError::InvalidFormat {
                message: format!(
                    "PDU-length ({length}) と実際の読み取りバイト数 ({offset}) が一致しません"
                ),
            });
        }

        let user_information = user_information.ok_or_else(|| StreamParseError::InvalidFormat {
            message: "User Information Item が存在しません".to_string(),
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
