pub(crate) const PDU_TYPE: u8 = 0x03;

pub struct AAssociateRj {
    result: Result,
    source_and_reason: SourceAndReason,
}

impl AAssociateRj {
    pub fn size(&self) -> usize {
        10
    }

    pub fn length(&self) -> u32 {
        4
    }

    pub fn result(&self) -> Result {
        self.result
    }

    pub fn source_and_reason(&self) -> &SourceAndReason {
        &self.source_and_reason
    }

    pub fn new(result: Result, source_and_reason: SourceAndReason) -> Self {
        Self {
            result,
            source_and_reason,
        }
    }
}

impl From<AAssociateRj> for Vec<u8> {
    fn from(val: AAssociateRj) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(val.size());

        bytes.push(PDU_TYPE);
        bytes.push(0); // Reserved
        bytes.extend(val.length().to_be_bytes());
        bytes.push(0); // Reserved
        bytes.push(val.result as u8);
        match val.source_and_reason {
            SourceAndReason::ServiceUser(reason) => {
                bytes.push(1);
                bytes.push(reason as u8);
            }
            SourceAndReason::ServiceProviderAcse(reason) => {
                bytes.push(2);
                bytes.push(reason as u8);
            }
            SourceAndReason::ServiceProviderPresentation(reason) => {
                bytes.push(3);
                bytes.push(reason as u8);
            }
        }

        bytes
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Result {
    RejectedPermanent = 1,
    RejectedTransient = 2,
}

pub enum SourceAndReason {
    ServiceUser(source::service_user::Reason),
    ServiceProviderAcse(source::service_provider_acse::Reason),
    ServiceProviderPresentation(source::service_provider_presentation::Reason),
}

pub mod source {

    pub mod service_user {
        pub enum Reason {
            NoReasonGiven = 1,
            ApplicationContextNameNotSupported = 2,
            CallingAeTitleNotRecognized = 3,
            CalledAeTitleNotRecognized = 7,
        }
    }

    pub mod service_provider_acse {
        pub enum Reason {
            NoReasonGiven = 1,
            ProtocolVersionNotSupported = 2,
        }
    }

    pub mod service_provider_presentation {
        pub enum Reason {
            NoReasonGiven = 1,
            TemporaryCongestion = 2,
            LocalLimitExceeded = 3,
        }
    }
}
