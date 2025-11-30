use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct FdValue(pub f64);

impl FdValue {
    const BYTE_LENGTH: usize = 8;

    pub fn value(&self) -> f64 {
        self.0
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, FdValueError> {
        if bytes.is_empty() {
            return Err(FdValueError::Empty);
        }

        if bytes.len() != Self::BYTE_LENGTH {
            return Err(FdValueError::InvalidLength {
                byte_length: bytes.len(),
            });
        }

        let bytes: [u8; 8] = bytes.try_into().unwrap(); // 長さが8バイトであることは上で確認済み
        let value = f64::from_le_bytes(bytes);

        Ok(Self(value))
    }
}

impl Display for FdValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum FdValueError {
    #[error("空値は許容されません")]
    Empty,

    #[error("バイト列の長さが8ではありません (バイト数={byte_length})")]
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
            let bytes = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

            // Act
            let actual = FdValue::from_bytes(&bytes).unwrap();

            // Assert
            assert_eq!(actual.value(), 0.0);
        }

        // 正常系: 正の値
        {
            // Arrange
            let bytes = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x3f]; // 1.0

            // Act
            let actual = FdValue::from_bytes(&bytes).unwrap();

            // Assert
            assert_eq!(actual.value(), 1.0);
        }

        // 正常系: 負の値
        {
            // Arrange
            let bytes = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0xbf]; // -1.0

            // Act
            let actual = FdValue::from_bytes(&bytes).unwrap();

            // Assert
            assert_eq!(actual.value(), -1.0);
        }

        // 正常系: 正の無限大
        {
            // Arrange
            let bytes = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x7f];

            // Act
            let actual = FdValue::from_bytes(&bytes).unwrap();

            // Assert
            assert_eq!(actual.value(), f64::INFINITY);
        }

        // 正常系: 負の無限大
        {
            // Arrange
            let bytes = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0xff];

            // Act
            let actual = FdValue::from_bytes(&bytes).unwrap();

            // Assert
            assert_eq!(actual.value(), f64::NEG_INFINITY);
        }

        // 正常系: NaN
        {
            // Arrange
            let bytes = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf8, 0x7f];

            // Act
            let actual = FdValue::from_bytes(&bytes).unwrap();

            // Assert
            assert!(actual.value().is_nan());
        }

        // 準正常系: 空のバッファ(Empty)
        {
            // Arrange
            let bytes = [];

            // Act
            let result = FdValue::from_bytes(&bytes);

            // Assert
            match result.unwrap_err() {
                FdValueError::Empty => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 1バイト(InvalidLength)
        {
            // Arrange
            let bytes = [0x01];

            // Act
            let result = FdValue::from_bytes(&bytes);

            // Assert
            match result.unwrap_err() {
                FdValueError::InvalidLength { byte_length } => {
                    assert_eq!(byte_length, 1);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 9バイト(InvalidLength)
        {
            // Arrange
            let bytes = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

            // Act
            let result = FdValue::from_bytes(&bytes);

            // Assert
            match result.unwrap_err() {
                FdValueError::InvalidLength { byte_length } => {
                    assert_eq!(byte_length, 9);
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
            let fd = FdValue(1.5);
            let expected = "1.5";

            // Act
            let actual = fd.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: ゼロ
        {
            // Arrange
            let fd = FdValue(0.0);
            let expected = "0";

            // Act
            let actual = fd.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 負の値
        {
            // Arrange
            let fd = FdValue(-1.5);
            let expected = "-1.5";

            // Act
            let actual = fd.to_string();

            // Assert
            assert_eq!(expected, actual);
        }
    }
}
