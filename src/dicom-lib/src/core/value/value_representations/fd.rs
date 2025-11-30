mod fd_value;

pub use fd_value::{FdValue, FdValueError};

use super::MultiNumberValueError;
use std::fmt::{Display, Formatter};
use thiserror::Error;

/// 倍精度浮動小数点数(Floating Point Double)
#[derive(Debug, PartialEq)]
pub struct Fd {
    values: Vec<FdValue>,
}

impl Fd {
    const BYTES_PER_VALUE: usize = 8;

    pub fn values(&self) -> &Vec<FdValue> {
        &self.values
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MultiNumberValueError> {
        if !bytes.len().is_multiple_of(Self::BYTES_PER_VALUE) {
            return Err(MultiNumberValueError::InvalidLength {
                bytes_per_value: Self::BYTES_PER_VALUE,
                byte_length: bytes.len(),
            });
        }

        if bytes.is_empty() {
            return Ok(Self { values: Vec::new() });
        }

        let mut values = Vec::with_capacity(bytes.len() / Self::BYTES_PER_VALUE);
        for chunk in bytes.chunks_exact(Self::BYTES_PER_VALUE) {
            let bytes: [u8; 8] = chunk.try_into().unwrap(); // chunks_exactなのでunwrapしても安全
            let value = FdValue(f64::from_le_bytes(bytes));
            values.push(value);
        }

        Ok(Self { values })
    }
}

impl Display for Fd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        const MAX_VALUES_TO_DISPLAY: usize = 16;

        write!(f, "[")?;
        for (i, value) in self.values.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            if i >= MAX_VALUES_TO_DISPLAY {
                write!(f, "...")?;
                break;
            }
            write!(f, "{}", value.0)?;
        }
        write!(f, "]")?;

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum FdError {
    #[error("バイト列の長さが8の倍数ではありません (バイト数={byte_length})")]
    InvalidLength { byte_length: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bytes() {
        // 正常系: 単一値
        {
            // Arrange
            let bytes = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x3f]; // 1.0
            let expected = Fd {
                values: vec![FdValue(1.0)],
            };

            // Act
            let actual = Fd::from_bytes(&bytes).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数値
        {
            // Arrange
            let bytes = [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x3f, // 1.0
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, // 2.0
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x40, // 3.0
            ];
            let expected = Fd {
                values: vec![FdValue(1.0), FdValue(2.0), FdValue(3.0)],
            };

            // Act
            let actual = Fd::from_bytes(&bytes).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空のバッファ
        {
            // Arrange
            let bytes = [];
            let expected = Fd { values: Vec::new() };

            // Act
            let actual = Fd::from_bytes(&bytes).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 1バイト(InvalidLength)
        {
            // Arrange
            let bytes = [0x01];

            // Act
            let result = Fd::from_bytes(&bytes);

            // Assert
            match result.unwrap_err() {
                MultiNumberValueError::InvalidLength { byte_length, .. } => {
                    assert_eq!(byte_length, 1);
                }
            }
        }

        // 準正常系: 9バイト(InvalidLength)
        {
            // Arrange
            let bytes = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

            // Act
            let result = Fd::from_bytes(&bytes);

            // Assert
            match result.unwrap_err() {
                MultiNumberValueError::InvalidLength { byte_length, .. } => {
                    assert_eq!(byte_length, 9);
                }
            }
        }
    }

    #[test]
    fn test_to_string() {
        // 正常系: 空値
        {
            // Arrange
            let fd = Fd { values: vec![] };
            let expected = "[]";

            // Act
            let actual = fd.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 単一値
        {
            // Arrange
            let fd = Fd {
                values: vec![FdValue(0.0)],
            };
            let expected = "[0]";

            // Act
            let actual = fd.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数値
        {
            // Arrange
            let fd = Fd {
                values: vec![FdValue(1.0), FdValue(2.0), FdValue(3.0)],
            };
            let expected = "[1, 2, 3]";

            // Act
            let actual = fd.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 多数の値
        {
            // Arrange
            let fd = Fd {
                values: (0..20).map(|i| FdValue(i as f64)).collect(),
            };
            let expected = "[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, ...]";

            // Act
            let actual = fd.to_string();

            // Assert
            assert_eq!(expected, actual);
        }
    }
}
