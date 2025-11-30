mod ui_value;

pub use ui_value::{UiValue, UiValueError};

use super::MultiStringValueError;
use std::fmt::{Display, Formatter};

/// UID(Unique Identifier)
#[derive(Debug, PartialEq)]
pub struct Ui {
    values: Vec<Option<UiValue>>,
}

impl Ui {
    pub fn values(&self) -> &Vec<Option<UiValue>> {
        &self.values
    }

    pub fn take_values(self) -> Vec<Option<UiValue>> {
        self.values
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MultiStringValueError> {
        let str = str::from_utf8(bytes).map_err(MultiStringValueError::InvalidUtf8)?;
        Self::from_string(str)
    }

    pub fn from_string(str: &str) -> Result<Self, MultiStringValueError> {
        let source_str = str;

        let str = if str.len().is_multiple_of(2) && str.ends_with('\0') {
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

            let value = UiValue::from_string(str).map_err(|error| {
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

impl Display for Ui {
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
            let expected = Ui { values: vec![None] };

            // Act
            let actual = Ui::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 単一値
        {
            // Arrange
            let input = "1.2.840.10008.1.1";
            let expected = Ui {
                values: vec![Some(UiValue::from_string("1.2.840.10008.1.1").unwrap())],
            };

            // Act
            let actual = Ui::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数値
        {
            // Arrange
            let input = r"1.2.840.10008.1.1\1.2.840.10008.1.2";
            let expected = Ui {
                values: vec![
                    Some(UiValue::from_string("1.2.840.10008.1.1").unwrap()),
                    Some(UiValue::from_string("1.2.840.10008.1.2").unwrap()),
                ],
            };

            // Act
            let actual = Ui::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数値で空の値を含むケース
        {
            // Arrange
            let input = r"\1.2.840.10008.1.2";
            let expected = Ui {
                values: vec![
                    None,
                    Some(UiValue::from_string("1.2.840.10008.1.2").unwrap()),
                ],
            };

            // Act
            let actual = Ui::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数値の最後の値がNULLパディングされ、その値の長さが65になるケース
        //        詳しくは以下を参照。
        //        https://dicom.nema.org/medical/dicom/2025c/output/chtml/part05/sect_6.4.html
        {
            // Arrange
            let input = "1234567890.1234567890.1234567890.1234567890.1234567890.123456789\\1234567890.1234567890.1234567890.1234567890.1234567890.123456789\0";
            let expected = Ui {
                values: vec![
                    Some(
                        UiValue::from_string(
                            "1234567890.1234567890.1234567890.1234567890.1234567890.123456789",
                        )
                        .unwrap(),
                    ),
                    Some(
                        UiValue::from_string(
                            "1234567890.1234567890.1234567890.1234567890.1234567890.123456789",
                        )
                        .unwrap(),
                    ),
                ],
            };

            // Act
            let actual = Ui::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn test_to_string() {
        // Arrange
        let source = Ui {
            values: vec![
                Some(UiValue::from_string("1.2.840.10008.1.1").unwrap()),
                Some(UiValue::from_string("1.2.840.10008.1.2").unwrap()),
            ],
        };
        let expected = r"1.2.840.10008.1.1\1.2.840.10008.1.2";

        // Act
        let actual = source.to_string();

        // Assert
        assert_eq!(expected, actual);
    }
}
