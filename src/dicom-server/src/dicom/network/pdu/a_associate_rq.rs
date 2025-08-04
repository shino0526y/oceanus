use crate::dicom::network::pdu::{
    self, INVALID_PDU_TYPE_ERROR_MESSAGE,
    a_associate::{
        self,
        items::{ApplicationContext, PresentationContext, UserInformation},
    },
};
use std::vec;

const PDU_TYPE: u8 = 0x01;

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
}

impl TryFrom<&[u8]> for AAssociateRq {
    type Error = String;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let pdu = pdu::Pdu::try_from(bytes)?;
        if pdu.pdu_type != PDU_TYPE {
            return Err(INVALID_PDU_TYPE_ERROR_MESSAGE.to_string());
        }

        let version = u16::from_be_bytes([pdu.data[0], pdu.data[1]]);
        let called_ae_title = std::str::from_utf8(&pdu.data[4..19])
            .map_err(|_| {
                "Called-AE-title フィールドを UTF-8 の文字列として解釈できません".to_string()
            })?
            .trim_end_matches(' ')
            .to_string();
        let calling_ae_title = std::str::from_utf8(&pdu.data[20..35])
            .map_err(|_| {
                "Calling-AE-title フィールドを UTF-8 の文字列として解釈できません".to_string()
            })?
            .trim_end_matches(' ')
            .to_string();

        let mut offset = 68;
        let application_context =
            ApplicationContext::try_from(&pdu.data[offset..]).map_err(|message| {
                format!("Application Context Item のパースに失敗しました: {message}")
            })?;
        offset += application_context.size();

        let mut presentation_contexts = vec![];
        let mut user_information = Option::None;
        while offset < pdu.data.len() {
            let item_type = pdu.data[offset];
            match item_type {
                a_associate::items::presentation_context::ITEM_TYPE => {
                    let presentation_context = PresentationContext::try_from(&pdu.data[offset..])
                        .map_err(|message| {
                        format!("Presentation Context Item のパースに失敗しました: {message}")
                    })?;
                    offset += presentation_context.size();
                    presentation_contexts.push(presentation_context);
                }
                a_associate::items::user_information::ITEM_TYPE => {
                    user_information = Some(
                        UserInformation::try_from(&pdu.data[offset..]).map_err(|message| {
                            format!("User Information Item のパースに失敗しました: {message}")
                        })?,
                    );
                    break;
                }
                _ => {
                    return Err(format!(
                        "Presentation Context Item もしくは User Information Item のパースを試みようとした際に予期しない Item-type (0x{item_type:02X}) を持つ Item が出現しました"
                    ));
                }
            }
        }

        if presentation_contexts.is_empty() {
            return Err("Presentation Context Item が存在しません".to_string());
        }
        if user_information.is_none() {
            return Err("User Information Item が存在しません".to_string());
        }

        Ok(AAssociateRq {
            length: pdu.length,
            version,
            called_ae_title,
            calling_ae_title,
            application_context,
            presentation_contexts,
            user_information: user_information.unwrap(),
        })
    }
}
