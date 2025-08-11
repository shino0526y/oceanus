pub mod presentation_data_value;

use crate::dicom::network::upper_layer_protocol::pdu::{self, INVALID_PDU_TYPE_ERROR_MESSAGE};
pub use presentation_data_value::PresentationDataValue;

const PDU_TYPE: u8 = 0x04;

pub struct PDataTf {
    length: u32,
    presentation_data_values: Vec<PresentationDataValue>,
}

impl PDataTf {
    pub fn size(&self) -> usize {
        6 + self.length as usize
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn presentation_data_values(&self) -> &[PresentationDataValue] {
        &self.presentation_data_values
    }

    pub fn new(presentation_data_values: Vec<PresentationDataValue>) -> Self {
        let length = presentation_data_values
            .iter()
            .map(|pdv| pdv.size() as u32)
            .sum();

        PDataTf {
            length,
            presentation_data_values,
        }
    }
}

impl TryFrom<&[u8]> for PDataTf {
    type Error = String;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let pdu = pdu::Pdu::try_from(bytes)?;
        if pdu.pdu_type != PDU_TYPE {
            return Err(INVALID_PDU_TYPE_ERROR_MESSAGE.to_string());
        }

        let mut presentation_data_values = Vec::new();
        let mut offset = 0;
        while offset < pdu.data.len() {
            let pdv = PresentationDataValue::try_from(&pdu.data[offset..])
                .map_err(|e| format!("Presentation Data Value Item のパースに失敗しました: {e}"))?;
            offset += pdv.size();
            presentation_data_values.push(pdv);
        }

        Ok(PDataTf {
            length: pdu.length,
            presentation_data_values,
        })
    }
}

impl From<&PDataTf> for Vec<u8> {
    fn from(val: &PDataTf) -> Self {
        let mut bytes = Vec::with_capacity(val.size());

        bytes.push(PDU_TYPE);
        bytes.push(0); // Reserved
        bytes.extend(val.length().to_be_bytes());
        val.presentation_data_values().iter().for_each(|pdv| {
            bytes.append(&mut pdv.into());
        });

        bytes
    }
}
