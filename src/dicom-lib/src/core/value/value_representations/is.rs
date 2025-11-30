mod is_value;

pub use is_value::{IsValue, IsValueError};

use super::MultiStringValueError;
use std::fmt::{Display, Formatter};

/// 整数文字列(Integer String)
#[derive(Debug, PartialEq)]
pub struct Is {
    values: Vec<Option<IsValue>>,
}

impl Is {
    pub fn values(&self) -> &Vec<Option<IsValue>> {
        &self.values
    }

    pub fn take_values(self) -> Vec<Option<IsValue>> {
        self.values
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

            let value = IsValue::from_string(str).map_err(|error| {
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

impl Display for Is {
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
            let expected = Is { values: vec![None] };

            // Act
            let actual = Is::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 単一値
        {
            // Arrange
            let input = "12345";
            let expected = Is {
                values: vec![Some(IsValue::from_string("12345").unwrap())],
            };

            // Act
            let actual = Is::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数値
        {
            // Arrange
            let input = r"100\200\300";
            let expected = Is {
                values: vec![
                    Some(IsValue::from_string("100").unwrap()),
                    Some(IsValue::from_string("200").unwrap()),
                    Some(IsValue::from_string("300").unwrap()),
                ],
            };

            // Act
            let actual = Is::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数値で空の値を含むケース
        {
            // Arrange
            let input = r"\123";
            let expected = Is {
                values: vec![None, Some(IsValue::from_string("123").unwrap())],
            };

            // Act
            let actual = Is::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn test_to_string() {
        // Arrange
        let source = Is {
            values: vec![
                Some(IsValue::from_string("100").unwrap()),
                Some(IsValue::from_string("200").unwrap()),
            ],
        };
        let expected = r"100\200";

        // Act
        let actual = source.to_string();

        // Assert
        assert_eq!(expected, actual);
    }
}
