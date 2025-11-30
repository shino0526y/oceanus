use std::fmt::{Display, Formatter};

/// その他バイト(Other Byte)
#[derive(Debug, PartialEq)]
pub struct Ob {
    values: Vec<u8>,
}

impl Ob {
    pub fn values(&self) -> &[u8] {
        &self.values
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            values: bytes.to_vec(),
        }
    }
}

impl Display for Ob {
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
            write!(f, "{}", value)?;
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
        // 正常系: 通常のバイト列
        {
            // Arrange
            let bytes = [0x01, 0x02, 0x03, 0x04];
            let expected = Ob {
                values: vec![1, 2, 3, 4],
            };

            // Act
            let actual = Ob::from_bytes(&bytes);

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空のバイト列
        {
            // Arrange
            let bytes: [u8; 0] = [];
            let expected = Ob { values: vec![] };

            // Act
            let actual = Ob::from_bytes(&bytes);

            // Assert
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn test_to_string() {
        // 正常系: 空のバイト列
        {
            // Arrange
            let ob = Ob { values: vec![] };
            let expected = "[]";

            // Act
            let actual = ob.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 短いバイト列
        {
            // Arrange
            let ob = Ob {
                values: vec![1, 2, 3, 4],
            };
            let expected = "[1, 2, 3, 4]";

            // Act
            let actual = ob.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 単一バイト
        {
            // Arrange
            let ob = Ob { values: vec![255] };
            let expected = "[255]";

            // Act
            let actual = ob.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数バイト
        {
            // Arrange
            let ob = Ob {
                values: vec![10, 20, 30, 40, 50],
            };
            let expected = "[10, 20, 30, 40, 50]";

            // Act
            let actual = ob.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 多数のバイト
        {
            // Arrange
            let ob = Ob {
                values: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            };
            let expected = "[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, ...]";

            // Act
            let actual = ob.to_string();

            // Assert
            assert_eq!(expected, actual);
        }
    }
}
