use std::{
    fmt::{Display, Formatter},
    str::Utf8Error,
};
use thiserror::Error;

/// URI/URL（Universal Resource Identifier or Universal Resource Locator）
///
/// URはRFC3986で定義されたURIまたはURLを識別する文字列です。
/// このVRは複数値にはなりません。
#[derive(Debug, PartialEq, Clone)]
pub struct Ur {
    uri: String,
}

impl Ur {
    pub fn uri(&self) -> &str {
        &self.uri
    }

    /// 文字列からURI/URL（UR）の値をパースします。
    ///
    /// この関数は、DICOM規格に準拠したUR型の文字列をパースします。
    /// URはRFC3986で定義されたURIまたはURLを識別する文字列で、
    /// **複数値にはなりません**（単一値のみ）。
    /// 先頭の空白は許可されず、末尾の空白は無視されます。
    ///
    /// # 引数
    ///
    /// * `str` - パースする文字列。複数値にはならないため、バックスラッシュ区切りは使用されません。
    ///
    /// # 戻り値
    ///
    /// * `Ok(Some(Self))` - パースに成功した場合、UR値を返します。
    /// * `Ok(None)` - 空文字列または空白のみの場合は、Noneを返します。
    /// * `Err(UrError)` - パースに失敗した場合、エラーを返します。
    ///   先頭に空白がある場合にエラーとなります。
    ///
    /// # パディング処理
    ///
    /// DICOM規格に従い、文字列が偶数バイト長で末尾が空白の場合、
    /// その空白は自動的に除去されます。また、末尾の空白もトリミングされます。
    ///
    /// # URI形式
    ///
    /// - RFC3986で要求されるDefault Character Repertoireのサブセット
    /// - 空白文字（20H）は末尾パディングのみ許可
    /// - 先頭の空白は許可されない
    /// - 絶対URIと相対URIの両方が許可される
    /// - バックスラッシュ（5CH）文字は使用可能
    ///
    /// # 例
    ///
    /// ```
    /// use dicom_lib::core::value::values::Ur;
    ///
    /// // 標準的なHTTP URL
    /// let result = Ur::from_string("https://example.com/path").unwrap();
    /// assert_eq!(result.as_ref().unwrap().uri(), "https://example.com/path");
    ///
    /// // 相対URI
    /// let result = Ur::from_string("/path/to/resource").unwrap();
    /// assert_eq!(result.as_ref().unwrap().uri(), "/path/to/resource");
    ///
    /// // クエリパラメータを含む
    /// let result = Ur::from_string("https://example.com/path?key=value&foo=bar").unwrap();
    /// assert_eq!(result.as_ref().unwrap().uri(), "https://example.com/path?key=value&foo=bar");
    ///
    /// // フラグメントを含む
    /// let result = Ur::from_string("https://example.com/page#section").unwrap();
    /// assert_eq!(result.as_ref().unwrap().uri(), "https://example.com/page#section");
    ///
    /// // 末尾の空白は自動的にトリミングされる
    /// let result = Ur::from_string("https://example.com  ").unwrap();
    /// assert_eq!(result.as_ref().unwrap().uri(), "https://example.com");
    ///
    /// // 空文字列
    /// let result = Ur::from_string("").unwrap();
    /// assert!(result.is_none());
    ///
    /// // 空白のみ
    /// let result = Ur::from_string("  ").unwrap();
    /// assert!(result.is_none());
    /// ```
    ///
    /// # エラー
    ///
    /// ```
    /// use dicom_lib::core::value::values::{Ur, ur::UrError};
    ///
    /// // 先頭に空白がある（パディングではない）
    /// let result = Ur::from_string(" https://example.com");
    /// assert!(matches!(result, Err(UrError::LeadingSpaceNotAllowed { .. })));
    /// ```
    pub fn from_string(str: &str) -> Result<Option<Self>, UrError> {
        // 偶数バイト長で末尾が空白の場合、パディングとして除去
        let str = if str.len().is_multiple_of(2) && str.ends_with(' ') {
            &str[..str.len() - 1]
        } else {
            str
        };

        // 末尾の空白を除去
        let trimmed = str.trim_end_matches(' ');

        // 空文字列の場合はNoneを返す
        if trimmed.is_empty() {
            return Ok(None);
        }

        // 先頭の空白チェック（先頭に空白がある場合はエラー）
        if trimmed.starts_with(' ') {
            return Err(UrError::LeadingSpaceNotAllowed {
                string: trimmed.to_string(),
            });
        }

        Ok(Some(Self {
            uri: trimmed.to_string(),
        }))
    }

    pub fn from_buf(buf: &[u8]) -> Result<Option<Self>, UrError> {
        let str = str::from_utf8(buf).map_err(UrError::InvalidUtf8)?;
        Self::from_string(str)
    }
}

impl Display for Ur {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.uri)
    }
}

#[derive(Error, Debug)]
pub enum UrError {
    #[error("先頭の空白は許可されていません (文字列=\"{string}\")")]
    LeadingSpaceNotAllowed { string: String },

    #[error("バイト列をUTF-8として解釈できません: {0}")]
    InvalidUtf8(#[from] Utf8Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string() {
        // 正常系: 標準的なHTTP URL
        {
            // Arrange
            let input = "https://example.com/path";
            let expected = Some(Ur {
                uri: "https://example.com/path".to_string(),
            });

            // Act
            let actual = Ur::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 相対URI
        {
            // Arrange
            let input = "/path/to/resource";
            let expected = Some(Ur {
                uri: "/path/to/resource".to_string(),
            });

            // Act
            let actual = Ur::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: クエリパラメータを含む
        {
            // Arrange
            let input = "https://example.com/path?key=value&foo=bar";
            let expected = Some(Ur {
                uri: "https://example.com/path?key=value&foo=bar".to_string(),
            });

            // Act
            let actual = Ur::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: フラグメントを含む
        {
            // Arrange
            let input = "https://example.com/page#section";
            let expected = Some(Ur {
                uri: "https://example.com/page#section".to_string(),
            });

            // Act
            let actual = Ur::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: ポート番号を含む
        {
            // Arrange
            let input = "https://example.com:8080/path";
            let expected = Some(Ur {
                uri: "https://example.com:8080/path".to_string(),
            });

            // Act
            let actual = Ur::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: パーセントエンコーディングを含む
        {
            // Arrange
            let input = "https://example.com/path%20with%20spaces";
            let expected = Some(Ur {
                uri: "https://example.com/path%20with%20spaces".to_string(),
            });

            // Act
            let actual = Ur::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 末尾に空白を含む（トリミングされる）
        {
            // Arrange
            let input = "https://example.com ";
            let expected = Some(Ur {
                uri: "https://example.com".to_string(),
            });

            // Act
            let actual = Ur::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空文字列
        {
            // Arrange
            let input = "";

            // Act
            let actual = Ur::from_string(input).unwrap();

            // Assert
            assert!(actual.is_none());
        }

        // 正常系: 空白のみ
        {
            // Arrange
            let input = "  ";

            // Act
            let actual = Ur::from_string(input).unwrap();

            // Assert
            assert!(actual.is_none());
        }

        // 正常系: file:// スキーム
        {
            // Arrange
            let input = "file:///path/to/file.dcm";
            let expected = Some(Ur {
                uri: "file:///path/to/file.dcm".to_string(),
            });

            // Act
            let actual = Ur::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: ftp:// スキーム
        {
            // Arrange
            let input = "ftp://ftp.example.com/file.txt";
            let expected = Some(Ur {
                uri: "ftp://ftp.example.com/file.txt".to_string(),
            });

            // Act
            let actual = Ur::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: バックスラッシュを含む（許可される）
        {
            // Arrange
            let input = "file:///C:\\path\\to\\file.dcm";
            let expected = Some(Ur {
                uri: "file:///C:\\path\\to\\file.dcm".to_string(),
            });

            // Act
            let actual = Ur::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 先頭に空白がある（パディングではない）
        {
            // Arrange
            let input = " https://example.com";

            // Act
            let result = Ur::from_string(input);

            // Assert
            match result.unwrap_err() {
                UrError::LeadingSpaceNotAllowed { string } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 前後に空白がある（先頭の空白はエラー）
        {
            // Arrange
            let input = " https://example.com ";

            // Act
            let result = Ur::from_string(input);

            // Assert
            match result.unwrap_err() {
                UrError::LeadingSpaceNotAllowed { string } => {
                    // 末尾の空白はトリミングされるため、先頭の空白のみが残る
                    assert_eq!(string, " https://example.com");
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_from_buf() {
        // 正常系: 標準的なHTTP URL
        {
            // Arrange
            let buf = b"https://example.com/path";
            let expected = Some(Ur {
                uri: "https://example.com/path".to_string(),
            });

            // Act
            let actual = Ur::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空白パディングを含む
        {
            // Arrange
            let buf = b"https://example.com ";
            let expected = Some(Ur {
                uri: "https://example.com".to_string(),
            });

            // Act
            let actual = Ur::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空のバッファ
        {
            // Arrange
            let buf = b"";

            // Act
            let actual = Ur::from_buf(buf).unwrap();

            // Assert
            assert!(actual.is_none());
        }

        // 準正常系: 不正なUTF-8バイト列
        {
            // Arrange
            let buf = b"\xff\xfe";

            // Act
            let result = Ur::from_buf(buf);

            // Assert
            match result.unwrap_err() {
                UrError::InvalidUtf8(_) => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 先頭に空白がある
        {
            // Arrange
            let buf = b" https://example.com";

            // Act
            let result = Ur::from_buf(buf);

            // Assert
            match result.unwrap_err() {
                UrError::LeadingSpaceNotAllowed { .. } => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_to_string() {
        // 正常系: URIを持つ場合
        {
            // Arrange
            let ur = Ur {
                uri: "https://example.com/path".to_string(),
            };
            let expected = "https://example.com/path";

            // Act
            let actual = ur.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空文字列の場合
        {
            // Arrange
            let ur = Ur { uri: String::new() };
            let expected = "";

            // Act
            let actual = ur.to_string();

            // Assert
            assert_eq!(expected, actual);
        }
    }
}
