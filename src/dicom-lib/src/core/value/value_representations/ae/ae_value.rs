use std::{
    fmt::{Display, Formatter},
    str::Utf8Error,
};
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AeValue(String);

impl AeValue {
    const MAX_BYTE_LENGTH: usize = 16;

    pub fn value(&self) -> &str {
        &self.0
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, AeValueError> {
        let str = str::from_utf8(bytes).map_err(AeValueError::InvalidUtf8)?;
        Self::from_string(str)
    }

    pub fn from_string(str: &str) -> Result<Self, AeValueError> {
        if str.len() > Self::MAX_BYTE_LENGTH {
            return Err(AeValueError::InvalidLength {
                string: str.to_string(),
                byte_length: str.len(),
            });
        }

        let trimmed = str.trim_matches(' ');
        if trimmed.is_empty() {
            return Err(AeValueError::Empty);
        }

        // 各文字が許可された文字であることを確認
        for (i, c) in trimmed.chars().enumerate() {
            if c == '\\' || c.is_control() {
                return Err(AeValueError::InvalidCharacter {
                    string: trimmed.to_string(),
                    character: c,
                    position: i,
                });
            }
        }

        Ok(Self(trimmed.to_string()))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.to_bytes_without_padding();
        if !bytes.len().is_multiple_of(2) {
            bytes.push(b' ');
        }
        bytes
    }

    fn to_bytes_without_padding(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }
}

impl Display for AeValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum AeValueError {
    #[error("空値は許容されません")]
    Empty,

    #[error("文字列の長さが16バイトを超えています (文字列=\"{string}\", バイト数={byte_length})")]
    InvalidLength { string: String, byte_length: usize },

    #[error(
        "文字列に不正な文字が含まれています (文字列=\"{string}\", 文字='{character}', 位置={position})"
    )]
    InvalidCharacter {
        string: String,
        character: char,
        position: usize,
    },

    #[error("バイト列をUTF-8として解釈できません: {0}")]
    InvalidUtf8(#[from] Utf8Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bytes() {
        // 準正常系: 不正なUTF-8バイト列(InvalidUtf8)
        {
            // Arrange
            let bytes = b"\xff\xfe";

            // Act
            let result = AeValue::from_bytes(bytes);

            // Assert
            match result.unwrap_err() {
                AeValueError::InvalidUtf8(_) => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_from_string() {
        // 正常系: 通常
        {
            // Arrange
            let input = "STORE_SCP";
            let expected = AeValue("STORE_SCP".to_string());

            // Act
            let actual = AeValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 前後に空白を含む
        {
            // Arrange
            let input = " STORE_SCP  ";
            let expected = AeValue("STORE_SCP".to_string());

            // Act
            let actual = AeValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 最大長
        {
            // Arrange
            let input = "1234567890ABCDEF";
            assert_eq!(input.len(), 16);
            let expected = AeValue(input.to_string());

            // Act
            let actual = AeValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 空文字列(Empty)
        {
            // Arrange
            let input = "";

            // Act
            let result = AeValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                AeValueError::Empty => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: スペースのみ(Empty)
        {
            // Arrange
            let input = "  ";

            // Act
            let result = AeValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                AeValueError::Empty => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 17バイトの文字列(InvalidLength)
        {
            // Arrange
            let input = "1234567890ABCDEFG";
            assert_eq!(input.len(), 17);

            // Act
            let result = AeValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                AeValueError::InvalidLength {
                    string,
                    byte_length,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(byte_length, 17);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: バックスラッシュを含む(InvalidCharacter)
        {
            // Arrange
            let input = r"STORE\SCP";

            // Act
            let result = AeValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                AeValueError::InvalidCharacter {
                    string,
                    character,
                    position,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(character, '\\');
                    assert_eq!(position, 5);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 改行を含む(InvalidCharacter)
        {
            // Arrange
            let input = "STORE\nSCP";

            // Act
            let result = AeValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                AeValueError::InvalidCharacter {
                    string,
                    character,
                    position,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(character, '\n');
                    assert_eq!(position, 5);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: タブを含む(InvalidCharacter)
        {
            // Arrange
            let input = "STORE\tSCP";

            // Act
            let result = AeValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                AeValueError::InvalidCharacter {
                    string,
                    character,
                    position,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(character, '\t');
                    assert_eq!(position, 5);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: NULL文字を含む(InvalidCharacter)
        {
            // Arrange
            let input = "STORE\0SCP";

            // Act
            let result = AeValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                AeValueError::InvalidCharacter {
                    string,
                    character,
                    position,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(character, '\0');
                    assert_eq!(position, 5);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_to_string() {
        // Arrange
        let ae = AeValue("STORE_SCP".to_string());
        let expected = "STORE_SCP";

        // Act
        let actual = ae.to_string();

        // Assert
        assert_eq!(expected, actual);
    }
}
