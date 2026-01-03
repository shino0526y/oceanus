mod cs_value;

pub use cs_value::{CsValue, CsValueError};

use super::MultiStringValueError;
use std::fmt::{Display, Formatter};

/// コード文字列(Code String)
#[derive(Debug, PartialEq)]
pub struct Cs {
    values: Vec<Option<CsValue>>,
}

impl Cs {
    pub fn values(&self) -> &Vec<Option<CsValue>> {
        &self.values
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, MultiStringValueError> {
        let str = str::from_utf8(bytes).map_err(MultiStringValueError::InvalidUtf8)?;
        Self::from_string(str)
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

            let value = CsValue::from_string(str).map_err(|error| {
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

impl Display for Cs {
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
            let expected = Cs { values: vec![None] };

            // Act
            let actual = Cs::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 単一値
        {
            // Arrange
            let input = "PRIMARY";
            let expected = Cs {
                values: vec![Some(CsValue::from_string("PRIMARY").unwrap())],
            };

            // Act
            let actual = Cs::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数値
        {
            // Arrange
            let input = r"PRIMARY\SECONDARY";
            let expected = Cs {
                values: vec![
                    Some(CsValue::from_string("PRIMARY").unwrap()),
                    Some(CsValue::from_string("SECONDARY").unwrap()),
                ],
            };

            // Act
            let actual = Cs::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数値で空の値を含むケース
        {
            // Arrange
            let input = r"\SECONDARY";
            let expected = Cs {
                values: vec![None, Some(CsValue::from_string("SECONDARY").unwrap())],
            };

            // Act
            let actual = Cs::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数値の最後の値が空白パディングされ、その値の長さが17になるケース
        //        詳しくは以下を参照。
        //        https://dicom.nema.org/medical/dicom/2025c/output/chtml/part05/sect_6.4.html
        {
            // Arrange
            let input = r"0123456789ABCDEF\0123456789ABCDEF ";
            let expected = Cs {
                values: vec![
                    Some(CsValue::from_string("0123456789ABCDEF").unwrap()),
                    Some(CsValue::from_string("0123456789ABCDEF").unwrap()),
                ],
            };

            // Act
            let actual = Cs::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn test_to_string() {
        // Arrange
        let source = Cs {
            values: vec![
                Some(CsValue::from_string("PRIMARY").unwrap()),
                Some(CsValue::from_string("SECONDARY").unwrap()),
            ],
        };
        let expected = r"PRIMARY\SECONDARY";

        // Act
        let actual = source.to_string();

        // Assert
        assert_eq!(expected, actual);
    }
}
