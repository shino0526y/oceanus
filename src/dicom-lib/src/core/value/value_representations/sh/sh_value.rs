use crate::core::value::{self, SpecificCharacterSet};
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct ShValue(String);

impl ShValue {
    const MAX_CHAR_COUNT: usize = 16;

    pub fn string(&self) -> &str {
        &self.0
    }

    pub fn from_bytes_lossy(
        bytes: &[u8],
        char_set: SpecificCharacterSet,
    ) -> Result<Self, ShValueError> {
        let str = value::generate_string_lossy(bytes, char_set);
        Self::from_string(&str)
    }

    pub fn from_string(str: &str) -> Result<Self, ShValueError> {
        if str.chars().count() > Self::MAX_CHAR_COUNT {
            return Err(ShValueError::InvalidLength {
                string: str.to_string(),
                char_count: str.chars().count(),
            });
        }

        let trimmed = str.trim_matches(' ');
        if trimmed.is_empty() {
            return Err(ShValueError::Empty);
        }

        Ok(Self(trimmed.to_string()))
    }
}

impl Display for ShValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum ShValueError {
    #[error("空値は許容されません")]
    Empty,

    #[error("文字列の長さが16文字を超えています (文字列=\"{string}\", 文字数={char_count})")]
    InvalidLength { string: String, char_count: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string() {
        // 正常系: 通常
        {
            // Arrange
            let input = "Short String";
            let expected = ShValue("Short String".to_string());

            // Act
            let actual = ShValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 前後に空白を含む
        {
            // Arrange
            let input = " Short String ";
            let expected = ShValue("Short String".to_string());

            // Act
            let actual = ShValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 最大長
        {
            // Arrange
            let input = "1234567890123456";
            assert_eq!(input.chars().count(), 16);
            let expected = ShValue(input.to_string());

            // Act
            let actual = ShValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 日本語を含む文字列
        {
            // Arrange
            let input = "日本語の文字列";
            let expected = ShValue("日本語の文字列".to_string());

            // Act
            let actual = ShValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 空文字列(空値)
        {
            // Arrange
            let input = "";

            // Act
            let result = ShValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                ShValueError::Empty => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: スペースのみ(空値)
        {
            // Arrange
            let input = "  ";

            // Act
            let result = ShValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                ShValueError::Empty => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 17文字の文字列(InvalidLength)
        {
            // Arrange
            let input = "12345678901234567";
            assert_eq!(input.chars().count(), 17);
            // Act
            let result = ShValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                ShValueError::InvalidLength { string, char_count } => {
                    assert_eq!(string, input);
                    assert_eq!(char_count, 17);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_to_string() {
        // Arrange
        let lo = ShValue("Short String".to_string());
        let expected = "Short String";

        // Act
        let actual = lo.to_string();

        // Assert
        assert_eq!(expected, actual);
    }
}
