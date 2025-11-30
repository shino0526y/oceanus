mod pn_component_group;
mod pn_value;

pub use pn_component_group::PnComponentGroup;
pub use pn_value::{PnValue, PnValueError};

use super::MultiStringValueError;
use crate::core::value::{self, SpecificCharacterSet};
use std::fmt::{Display, Formatter};

/// 人名(Person Name)
#[derive(Debug, PartialEq)]
pub struct Pn {
    values: Vec<Option<PnValue>>,
}

impl Pn {
    pub fn values(&self) -> &Vec<Option<PnValue>> {
        &self.values
    }

    pub fn take_values(self) -> Vec<Option<PnValue>> {
        self.values
    }

    pub fn from_bytes_lossy(
        bytes: &[u8],
        char_set: SpecificCharacterSet,
    ) -> Result<Self, MultiStringValueError> {
        let strings = value::generate_person_name_strings_lossy(bytes, char_set);

        let mut values = Vec::with_capacity(strings.len());
        for (i, str) in strings.iter().enumerate() {
            if str.is_empty() {
                values.push(None);
                continue;
            }

            let value = PnValue::from_string(str).map_err(|error| {
                MultiStringValueError::FailedToParse {
                    string: strings.join("\\"),
                    index: i,
                    error: Box::new(error),
                }
            })?;
            values.push(Some(value));
        }

        Ok(Self { values })
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

            let value = PnValue::from_string(str).map_err(|error| {
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

impl Display for Pn {
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
            let expected = Pn { values: vec![None] };

            // Act
            let actual = Pn::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 単一値
        {
            // Arrange
            let input = "Doe^John";
            let expected = Pn {
                values: vec![Some(PnValue::from_string("Doe^John").unwrap())],
            };

            // Act
            let actual = Pn::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数値
        {
            // Arrange
            let input = r"Doe^John\Doe^Jane";
            let expected = Pn {
                values: vec![
                    Some(PnValue::from_string("Doe^John").unwrap()),
                    Some(PnValue::from_string("Doe^Jane").unwrap()),
                ],
            };

            // Act
            let actual = Pn::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数値で空の値を含むケース
        {
            // Arrange
            let input = r"\Doe^Jane";
            let expected = Pn {
                values: vec![None, Some(PnValue::from_string("Doe^Jane").unwrap())],
            };

            // Act
            let actual = Pn::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn test_to_string() {
        // Arrange
        let source = Pn {
            values: vec![
                Some(PnValue::from_string("Doe^John").unwrap()),
                Some(PnValue::from_string("Doe^Jane").unwrap()),
            ],
        };
        let expected = r"Doe^John\Doe^Jane";

        // Act
        let actual = source.to_string();

        // Assert
        assert_eq!(expected, actual);
    }
}
