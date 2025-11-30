use std::{
    fmt::{Display, Formatter},
    str::Utf8Error,
};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct UrValue(String);

impl UrValue {
    pub fn uri(&self) -> &str {
        &self.0
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, UrValueError> {
        let str = str::from_utf8(bytes).map_err(UrValueError::InvalidUtf8)?;
        Self::from_string(str)
    }

    pub fn from_string(str: &str) -> Result<Self, UrValueError> {
        let trimmed = str.trim_end_matches(' ');
        if trimmed.is_empty() {
            return Err(UrValueError::Empty);
        }

        // 先頭の空白チェック(先頭に空白がある場合はエラー)
        if trimmed.starts_with(' ') {
            return Err(UrValueError::LeadingSpaceNotAllowed {
                string: trimmed.to_string(),
            });
        }

        Ok(Self(trimmed.to_string()))
    }
}

impl Display for UrValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum UrValueError {
    #[error("空値は許容されません")]
    Empty,

    #[error("先頭の空白は許可されていません (文字列=\"{string}\")")]
    LeadingSpaceNotAllowed { string: String },

    #[error("バイト列をUTF-8として解釈できません: {0}")]
    InvalidUtf8(#[from] Utf8Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_bytes() {
        // 準正常系: 不正なUTF-8バイト列
        {
            // Arrange
            let bytes = b"\xff\xfe";

            // Act
            let result = UrValue::from_bytes(bytes);

            // Assert
            match result.unwrap_err() {
                UrValueError::InvalidUtf8(_) => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_from_string() {
        // 正常系: 標準的なURL
        {
            // Arrange
            let input = "https://example.com/path";
            let expected = UrValue("https://example.com/path".to_string());

            // Act
            let actual = UrValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 相対URI
        {
            // Arrange
            let input = "/path/to/resource";
            let expected = UrValue("/path/to/resource".to_string());

            // Act
            let actual = UrValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: クエリパラメータを含む
        {
            // Arrange
            let input = "https://example.com/path?key=value&foo=bar";
            let expected = UrValue("https://example.com/path?key=value&foo=bar".to_string());

            // Act
            let actual = UrValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: フラグメントを含む
        {
            // Arrange
            let input = "https://example.com/page#section";
            let expected = UrValue("https://example.com/page#section".to_string());

            // Act
            let actual = UrValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: ポート番号を含む
        {
            // Arrange
            let input = "https://example.com:8080/path";
            let expected = UrValue("https://example.com:8080/path".to_string());

            // Act
            let actual = UrValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: パーセントエンコーディングを含む
        {
            // Arrange
            let input = "https://example.com/path%20with%20spaces";
            let expected = UrValue("https://example.com/path%20with%20spaces".to_string());

            // Act
            let actual = UrValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 末尾に空白を含む(トリミングされる)
        {
            // Arrange
            let input = "https://example.com ";
            let expected = UrValue("https://example.com".to_string());

            // Act
            let actual = UrValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: file:// スキーム
        {
            // Arrange
            let input = "file:///path/to/file.dcm";
            let expected = UrValue("file:///path/to/file.dcm".to_string());

            // Act
            let actual = UrValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: ftp:// スキーム
        {
            // Arrange
            let input = "ftp://ftp.example.com/file.txt";
            let expected = UrValue("ftp://ftp.example.com/file.txt".to_string());

            // Act
            let actual = UrValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: バックスラッシュを含む(許可される)
        {
            // Arrange
            let input = "file:///C:\\path\\to\\file.dcm";
            let expected = UrValue("file:///C:\\path\\to\\file.dcm".to_string());

            // Act
            let actual = UrValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 空文字列(Empty)
        {
            // Arrange
            let input = "";

            // Act
            let result = UrValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                UrValueError::Empty => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 空白のみ(Empty)
        {
            // Arrange
            let input = "  ";

            // Act
            let result = UrValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                UrValueError::Empty => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 先頭に空白がある(LeadingSpaceNotAllowed)
        {
            // Arrange
            let input = " https://example.com";

            // Act
            let result = UrValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                UrValueError::LeadingSpaceNotAllowed { string } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_to_string() {
        // 正常系: 標準的なURL
        {
            // Arrange
            let ur = UrValue("https://example.com/path".to_string());
            let expected = "https://example.com/path";

            // Act
            let actual = ur.to_string();

            // Assert
            assert_eq!(expected, actual);
        }
    }
}
