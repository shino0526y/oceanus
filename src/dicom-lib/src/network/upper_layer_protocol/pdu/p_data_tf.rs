pub mod presentation_data_value;

pub use presentation_data_value::PresentationDataValue;

use crate::network::upper_layer_protocol::pdu::{INVALID_PDU_LENGTH_ERROR_MESSAGE, PduReadError};
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};

pub(crate) const PDU_TYPE: u8 = 0x04;

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

        Self {
            length,
            presentation_data_values,
        }
    }

    pub async fn read_from_stream(
        buf_reader: &mut BufReader<impl AsyncRead + Unpin>,
        length: u32,
    ) -> Result<Self, PduReadError> {
        let mut offset = 0;
        let mut presentation_data_values = vec![];
        while offset + 4 < length as usize {
            // オフセット + PDVの最小サイズ が全体の長さを超えない範囲でループ

            let pdv_length = buf_reader.read_u32().await?;
            offset += 4;

            if offset + pdv_length as usize > length as usize {
                // オフセット + PDVの長さ が全体の長さを超える場合
                return Err(PduReadError::InvalidPduParameterValue {
                    message: INVALID_PDU_LENGTH_ERROR_MESSAGE.to_string(),
                });
            }

            let pdv = PresentationDataValue::read_from_stream(buf_reader, pdv_length)
                .await
                .map_err(|e| match e {
                    PduReadError::IoError(_) => e,
                    PduReadError::InvalidPduParameterValue { message } => {
                        PduReadError::InvalidPduParameterValue {
                            message: format!(
                                "Presentation Data Value Itemのパースに失敗しました: {message}"
                            ),
                        }
                    }
                    _ => panic!(),
                })?;
            offset += pdv.length() as usize;

            presentation_data_values.push(pdv);
        }

        if offset != length as usize {
            return Err(PduReadError::InvalidPduParameterValue {
                message: format!(
                    "PDU-lengthと実際の読み取りバイト数が一致しません (PDU-length={length} 読み取りバイト数={offset})"
                ),
            });
        }

        Ok(Self {
            length,
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
