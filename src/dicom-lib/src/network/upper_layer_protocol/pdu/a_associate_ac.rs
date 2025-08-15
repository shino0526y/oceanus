pub mod presentation_context;

pub use crate::network::upper_layer_protocol::pdu::a_associate::*;
pub use presentation_context::PresentationContext;

pub(crate) const PDU_TYPE: u8 = 0x02;

pub struct AAssociateAc {
    length: u32,
    version: u16,
    called_ae_title: String,
    calling_ae_title: String,
    application_context: ApplicationContext,
    presentation_contexts: Vec<PresentationContext>,
    user_information: UserInformation,
}

impl AAssociateAc {
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

    pub fn new<T: Into<String>>(
        version: u16,
        called_ae_title: T,
        calling_ae_title: T,
        application_context: ApplicationContext,
        presentation_contexts: Vec<PresentationContext>,
        user_information: UserInformation,
    ) -> Result<Self, &'static str> {
        if version != 1 {
            return Err("Protocol-version は 1 でなければなりません");
        }
        let called_ae_title = called_ae_title.into();
        if called_ae_title.is_empty() || called_ae_title.len() > 16 {
            return Err("Called-AE-title は 1 文字以上 16 文字以下でなければなりません");
        }
        if !called_ae_title.is_ascii() {
            return Err(
                "Called-AE-title は ISO 646:1990 (basic G0 set) でエンコーディングされている必要があります",
            );
        }
        let calling_ae_title = calling_ae_title.into();
        if calling_ae_title.is_empty() || calling_ae_title.len() > 16 {
            return Err("Calling-AE-title は 1 文字以上 16 文字以下でなければなりません");
        }
        if !calling_ae_title.is_ascii() {
            return Err(
                "Calling-AE-title は ISO 646:1990 (basic G0 set) でエンコーディングされている必要があります",
            );
        }

        let length = 2 // Protocol-version
            + 2 // Reserved
            + 16 // Reserved (Called-AE-title)
            + 16 // Reserved (Calling-AE-title)
            + 32 // Reserved
            + application_context.size() as u32
            + presentation_contexts
                .iter()
                .map(|presentation_context| presentation_context.size() as u32)
                .sum::<u32>()
            + user_information.size() as u32;

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

impl From<AAssociateAc> for Vec<u8> {
    fn from(val: AAssociateAc) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(val.size());

        bytes.push(PDU_TYPE);
        bytes.push(0); // Reserved
        bytes.extend(val.length.to_be_bytes());
        bytes.extend(val.version.to_be_bytes());
        bytes.extend([0; 2]); // Reserved
        bytes.extend(val.called_ae_title.as_bytes()); // Reserved (Called-AE-title)
        bytes.extend(vec![0; 16 - val.called_ae_title.len()]);
        bytes.extend(val.calling_ae_title.as_bytes()); // Reserved (Calling-AE-title)
        bytes.extend(vec![0; 16 - val.calling_ae_title.len()]);
        bytes.extend(vec![0; 32]); // Reserved
        bytes.append(&mut val.application_context.into());
        for presentation_context in val.presentation_contexts {
            bytes.append(&mut presentation_context.into());
        }
        bytes.append(&mut val.user_information.into());

        bytes
    }
}
