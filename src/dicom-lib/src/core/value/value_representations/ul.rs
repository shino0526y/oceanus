mod ul_value;

pub use ul_value::{UlValue, UlValueError};

use super::MultiNumberValueError;
use std::fmt::{Display, Formatter};

/// 符号なし32ビット整数(Unsigned Long)
#[derive(Debug, PartialEq)]
pub struct Ul {
    values: Vec<UlValue>,
}

impl Ul {
    const BYTES_PER_VALUE: usize = 4;

    pub fn values(&self) -> &Vec<UlValue> {
        &self.values
    }

    pub fn take_values(self) -> Vec<UlValue> {
        self.values
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
            let bytes: [u8; 4] = chunk.try_into().unwrap(); // chunks_exactなのでunwrapしても安全
            let value = UlValue(u32::from_le_bytes(bytes));
            values.push(value);
        }

        Ok(Self { values })
    }
}

impl Display for Ul {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bytes() {
        // 正常系: 単一値
        {
            // Arrange
            let bytes = [0x01, 0x00, 0x00, 0x00];
            let expected = Ul {
                values: vec![UlValue(1)],
            };

            // Act
            let actual = Ul::from_bytes(&bytes).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数値
        {
            // Arrange
            let bytes = [
                0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00,
            ];
            let expected = Ul {
                values: vec![UlValue(1), UlValue(2), UlValue(3)],
            };

            // Act
            let actual = Ul::from_bytes(&bytes).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空のバッファ
        {
            // Arrange
            let bytes = [];
            let expected = Ul { values: Vec::new() };

            // Act
            let actual = Ul::from_bytes(&bytes).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 1バイト(InvalidLength)
        {
            // Arrange
            let bytes = [0x01];

            // Act
            let result = Ul::from_bytes(&bytes);

            // Assert
            match result.unwrap_err() {
                MultiNumberValueError::InvalidLength { byte_length, .. } => {
                    assert_eq!(byte_length, 1);
                }
            }
        }

        // 準正常系: 5バイト(InvalidLength)
        {
            // Arrange
            let bytes = [0x01, 0x00, 0x00, 0x00, 0x02];

            // Act
            let result = Ul::from_bytes(&bytes);

            // Assert
            match result.unwrap_err() {
                MultiNumberValueError::InvalidLength { byte_length, .. } => {
                    assert_eq!(byte_length, 5);
                }
            }
        }
    }

    #[test]
    fn test_to_string() {
        // 正常系: 空値
        {
            // Arrange
            let ul = Ul { values: vec![] };
            let expected = "[]";

            // Act
            let actual = ul.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 単一値
        {
            // Arrange
            let ul = Ul {
                values: vec![UlValue(0)],
            };
            let expected = "[0]";

            // Act
            let actual = ul.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数値
        {
            // Arrange
            let ul = Ul {
                values: vec![UlValue(1), UlValue(2), UlValue(3)],
            };
            let expected = "[1, 2, 3]";

            // Act
            let actual = ul.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 多数の値
        {
            // Arrange
            let ul = Ul {
                values: (0..20).map(UlValue).collect(),
            };
            let expected = "[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, ...]";

            // Act
            let actual = ul.to_string();

            // Assert
            assert_eq!(expected, actual);
        }
    }
}
