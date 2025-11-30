mod ur_value;

pub use ur_value::{UrValue, UrValueError};

use super::SingleStringValueError;
use std::fmt::{Display, Formatter};

/// URI/URL(Universal Resource Identifier or Universal Resource Locator)
///
/// URはRFC3986で定義されたURIまたはURLを識別する文字列です。
/// このVRは複数値にはなりません。
#[derive(Debug, PartialEq)]
pub struct Ur {
    value: Option<UrValue>,
}

impl Ur {
    pub fn value(&self) -> Option<&UrValue> {
        self.value.as_ref()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SingleStringValueError> {
        let str = str::from_utf8(bytes).map_err(SingleStringValueError::InvalidUtf8)?;
        Self::from_string(str)
    }

    pub fn from_string(str: &str) -> Result<Self, SingleStringValueError> {
        let trimmed = str.trim_matches(' ');
        if trimmed.is_empty() {
            return Ok(Self { value: None });
        }

        let value = UrValue::from_string(trimmed).map_err(|error| {
            SingleStringValueError::FailedToParse {
                string: str.to_string(),
                error: Box::new(error),
            }
        })?;
        Ok(Self { value: Some(value) })
    }
}

impl Display for Ur {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self.value.as_ref() {
                Some(v) => v.to_string(),
                None => String::new(),
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string() {
        // 正常系: 標準的なURL
        {
            // Arrange
            let input = "https://example.com/path";
            let expected = Ur {
                value: Some(UrValue::from_string("https://example.com/path").unwrap()),
            };

            // Act
            let actual = Ur::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空文字列
        {
            // Arrange
            let input = "";
            let expected = Ur { value: None };

            // Act
            let actual = Ur::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空白のみ
        {
            // Arrange
            let input = "  ";
            let expected = Ur { value: None };

            // Act
            let actual = Ur::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn test_to_string() {
        // 正常系: 標準的なURL
        {
            // Arrange
            let ur = Ur {
                value: Some(UrValue::from_string("https://example.com/path").unwrap()),
            };
            let expected = "https://example.com/path";

            // Act
            let actual = ur.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空値
        {
            // Arrange
            let ur = Ur { value: None };
            let expected = "";

            // Act
            let actual = ur.to_string();

            // Assert
            assert_eq!(expected, actual);
        }
    }
}
