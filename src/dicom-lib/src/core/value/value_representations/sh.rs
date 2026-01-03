mod sh_value;

pub use sh_value::{ShValue, ShValueError};

use super::MultiStringValueError;
use crate::core::value::{self, SpecificCharacterSet};
use std::fmt::{Display, Formatter};

/// 短い文字列(Short String)
#[derive(Debug, PartialEq)]
pub struct Sh {
    values: Vec<Option<ShValue>>,
}

impl Sh {
    pub fn values(&self) -> &Vec<Option<ShValue>> {
        &self.values
    }

    pub fn from_bytes_lossy(
        bytes: &[u8],
        char_set: SpecificCharacterSet,
    ) -> Result<Self, MultiStringValueError> {
        let str = value::generate_string_lossy(bytes, char_set);
        Self::from_string(&str)
    }

    pub fn from_string(str: &str) -> Result<Self, MultiStringValueError> {
        let source_str = str;

        let str = if str.len().is_multiple_of(2) && str.ends_with(' ') {
            &str[..str.len() - 1]
        } else {
            str
        };
        let strings = str.split('\\').collect::<Vec<_>>();

        let mut values = Vec::with_capacity(strings.len());
        for (i, str) in strings.iter().enumerate() {
            if str.is_empty() {
                values.push(None);
                continue;
            }

            let value = ShValue::from_string(str).map_err(|error| {
                MultiStringValueError::FailedToParse {
                    string: source_str.to_string(),
                    index: i,
                    error: Box::new(error),
                }
            })?;
            values.push(Some(value));
        }

        Ok(Self { values })
    }
}

impl Display for Sh {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.values
                .iter()
                .map(|v| match v {
                    Some(v) => v.to_string(),
                    None => String::new(),
                })
                .collect::<Vec<_>>()
                .join("\\")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string() {
        // 正常系: 空の文字列
        {
            // Arrange
            let input = "";
            let expected = Sh { values: vec![None] };

            // Act
            let actual = Sh::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 単一値
        {
            // Arrange
            let input = "Short String";
            let expected = Sh {
                values: vec![Some(ShValue::from_string("Short String").unwrap())],
            };

            // Act
            let actual = Sh::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数値
        {
            // Arrange
            let input = r"Short String 1\Short String 2";
            let expected = Sh {
                values: vec![
                    Some(ShValue::from_string("Short String 1").unwrap()),
                    Some(ShValue::from_string("Short String 2").unwrap()),
                ],
            };

            // Act
            let actual = Sh::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数値で空の値を含むケース
        {
            // Arrange
            let input = r"\Short String 2";
            let expected = Sh {
                values: vec![None, Some(ShValue::from_string("Short String 2").unwrap())],
            };

            // Act
            let actual = Sh::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数値の最後の値が空白パディングされ、その値の長さが17になるケース
        //        詳しくは以下を参照。
        //        https://dicom.nema.org/medical/dicom/2025c/output/chtml/part05/sect_6.4.html
        {
            // Arrange
            let input = r"1234567890123456\1234567890123456 ";
            let expected = Sh {
                values: vec![
                    Some(ShValue::from_string("1234567890123456").unwrap()),
                    Some(ShValue::from_string("1234567890123456").unwrap()),
                ],
            };

            // Act
            let actual = Sh::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn test_to_string() {
        // Arrange
        let source = Sh {
            values: vec![
                Some(ShValue::from_string("Short String 1").unwrap()),
                Some(ShValue::from_string("Short String 2").unwrap()),
            ],
        };
        let expected = r"Short String 1\Short String 2";

        // Act
        let actual = source.to_string();

        // Assert
        assert_eq!(expected, actual);
    }
}
