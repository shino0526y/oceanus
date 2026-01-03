use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct UlValue(pub u32);

impl UlValue {
    const BYTE_LENGTH: usize = 4;

    pub fn value(&self) -> u32 {
        self.0
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, UlValueError> {
        if bytes.is_empty() {
            return Err(UlValueError::Empty);
        }

        if bytes.len() != Self::BYTE_LENGTH {
            return Err(UlValueError::InvalidLength {
                byte_length: bytes.len(),
            });
        }

        let bytes: [u8; 4] = bytes.try_into().unwrap(); // 長さが4バイトであることは上で確認済み
        let value = u32::from_le_bytes(bytes);

        Ok(Self(value))
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        self.0.to_le_bytes()
    }
}

impl Display for UlValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum UlValueError {
    #[error("空値は許容されません")]
    Empty,

    #[error("バイト列の長さが4ではありません (バイト数={byte_length})")]
    InvalidLength { byte_length: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bytes() {
        // 正常系: ゼロ
        {
            // Arrange
            let bytes = [0x00, 0x00, 0x00, 0x00];

            // Act
            let actual = UlValue::from_bytes(&bytes).unwrap();

            // Assert
            assert_eq!(actual.value(), 0);
        }

        // 正常系: 最大値 (2^32 - 1)
        {
            // Arrange
            let bytes = [0xff, 0xff, 0xff, 0xff];

            // Act
            let actual = UlValue::from_bytes(&bytes).unwrap();

            // Assert
            assert_eq!(actual.value(), 4294967295);
        }

        // 準正常系: 空のバッファ(Empty)
        {
            // Arrange
            let bytes = [];

            // Act
            let result = UlValue::from_bytes(&bytes);

            // Assert
            match result.unwrap_err() {
                UlValueError::Empty => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 1バイト(InvalidLength)
        {
            // Arrange
            let bytes = [0x01];

            // Act
            let result = UlValue::from_bytes(&bytes);

            // Assert
            match result.unwrap_err() {
                UlValueError::InvalidLength { byte_length } => {
                    assert_eq!(byte_length, 1);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 5バイト(InvalidLength)
        {
            // Arrange
            let bytes = [0x01, 0x00, 0x00, 0x00, 0x02];

            // Act
            let result = UlValue::from_bytes(&bytes);

            // Assert
            match result.unwrap_err() {
                UlValueError::InvalidLength { byte_length } => {
                    assert_eq!(byte_length, 5);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_to_string() {
        // 正常系: 通常の値
        {
            // Arrange
            let ul = UlValue(123);
            let expected = "123";

            // Act
            let actual = ul.to_string();

            // Assert
            assert_eq!(expected, actual);
        }
    }
}
