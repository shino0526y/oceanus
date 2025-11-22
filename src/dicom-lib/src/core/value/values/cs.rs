use std::{
    fmt::{Display, Formatter},
    str::Utf8Error,
};
use thiserror::Error;

/// コード文字列（Code String）
#[derive(Debug, PartialEq, Clone)]
pub struct Cs {
    code: String,
}

impl Cs {
    const MAX_BYTE_LENGTH: usize = 16;

    pub fn code(&self) -> &str {
        &self.code
    }

    /// 文字列からコード文字列（CS）の値をパースします。
    ///
    /// この関数は、DICOM規格に準拠したCS型の文字列をパースし、
    /// バックスラッシュ(`\`)で区切られた複数の値を処理します。
    /// 各値は最大16バイトまでで、大文字、数字、空白、アンダースコアのみが許可されます。
    ///
    /// # 引数
    ///
    /// * `str` - パースする文字列。複数の値はバックスラッシュ(`\`)で区切られます。
    ///
    /// # 戻り値
    ///
    /// * `Ok(Vec<Option<Self>>)` - パースに成功した場合、CS値のベクタを返します。
    ///   空白のみの値や空文字列は`None`として表現されます。
    /// * `Err(CsError)` - パースに失敗した場合、エラーを返します。
    ///   不正な文字が含まれている場合や、16バイトを超える値がある場合にエラーとなります。
    ///
    /// # パディング処理
    ///
    /// DICOM規格に従い、文字列が偶数バイト長で末尾が空白の場合、
    /// その空白は自動的に除去されます。
    ///
    /// # 例
    ///
    /// ```
    /// use dicom_lib::core::value::values::Cs;
    ///
    /// // 単一の値をパース
    /// let result = Cs::from_string("ORIGINAL").unwrap();
    /// assert_eq!(result.len(), 1);
    /// assert_eq!(result[0].as_ref().unwrap().code(), "ORIGINAL");
    ///
    /// // 複数の値をパース（バックスラッシュ区切り）
    /// let result = Cs::from_string(r"PRIMARY\SECONDARY").unwrap();
    /// assert_eq!(result.len(), 2);
    /// assert_eq!(result[0].as_ref().unwrap().code(), "PRIMARY");
    /// assert_eq!(result[1].as_ref().unwrap().code(), "SECONDARY");
    ///
    /// // 空値を含むケース
    /// let result = Cs::from_string(r"\ISO_IR_192").unwrap();
    /// assert_eq!(result.len(), 2);
    /// assert!(result[0].is_none());
    /// assert_eq!(result[1].as_ref().unwrap().code(), "ISO_IR_192");
    ///
    /// // 前後の空白は自動的にトリミングされる
    /// let result = Cs::from_string(" DERIVED  ").unwrap();
    /// assert_eq!(result[0].as_ref().unwrap().code(), "DERIVED");
    ///
    /// // 数字、アンダースコア、空白を含む値
    /// let result = Cs::from_string("ISO 2022 IR 87").unwrap();
    /// assert_eq!(result[0].as_ref().unwrap().code(), "ISO 2022 IR 87");
    /// ```
    ///
    /// # エラー
    ///
    /// ```
    /// use dicom_lib::core::value::values::{Cs, cs::CsError};
    ///
    /// // 17バイトを超える文字列
    /// let result = Cs::from_string("1234567890ABCDEFG");
    /// assert!(matches!(result, Err(CsError::InvalidLength { .. })));
    ///
    /// // 小文字を含む（不正な文字）
    /// let result = Cs::from_string("primary");
    /// assert!(matches!(result, Err(CsError::InvalidCharacter { .. })));
    ///
    /// // 記号を含む（不正な文字）
    /// let result = Cs::from_string("CODE-STRING");
    /// assert!(matches!(result, Err(CsError::InvalidCharacter { .. })));
    /// ```
    pub fn from_string(str: &str) -> Result<Vec<Option<Self>>, CsError> {
        let str = if str.len().is_multiple_of(2) && str.ends_with(' ') {
            &str[..str.len() - 1]
        } else {
            str
        };
        let strings = str.split('\\').collect::<Vec<_>>();

        let mut values = Vec::with_capacity(strings.len());
        for str in strings {
            values.push(Self::from_string_single(str)?);
        }

        Ok(values)
    }

    pub fn from_buf(buf: &[u8]) -> Result<Vec<Option<Self>>, CsError> {
        let str = str::from_utf8(buf).map_err(CsError::InvalidUtf8)?;
        Self::from_string(str)
    }

    fn from_string_single(str: &str) -> Result<Option<Self>, CsError> {
        if str.len() > Self::MAX_BYTE_LENGTH {
            return Err(CsError::InvalidLength {
                string: str.to_string(),
                byte_length: str.len(),
            });
        }

        let trimmed = str.trim_matches(' ');
        if trimmed.is_empty() {
            return Ok(None);
        }

        // 各文字が許可された文字（大文字、数字、スペース、アンダースコア）であることを確認
        for (i, c) in trimmed.chars().enumerate() {
            if !matches!(c, 'A'..='Z' | '0'..='9' | ' ' | '_') {
                return Err(CsError::InvalidCharacter {
                    string: trimmed.to_string(),
                    character: c,
                    position: i,
                });
            }
        }

        Ok(Some(Self {
            code: trimmed.to_string(),
        }))
    }
}

impl Display for Cs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code)
    }
}

#[derive(Error, Debug)]
pub enum CsError {
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
    fn test_from_string() {
        // 正常系: 通常
        {
            // Arrange
            let input = "ORIGINAL";
            let expected = vec![Some(Cs {
                code: "ORIGINAL".to_string(),
            })];

            // Act
            let actual = Cs::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数の値（バックスラッシュ区切り）
        {
            // Arrange
            let input = r"PRIMARY\SECONDARY";
            let expected = vec![
                Some(Cs {
                    code: "PRIMARY".to_string(),
                }),
                Some(Cs {
                    code: "SECONDARY".to_string(),
                }),
            ];

            // Act
            let actual = Cs::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空の値を含むケース
        {
            // Arrange
            let input = r"\ISO 2022 IR 87";
            let expected = vec![
                None,
                Some(Cs {
                    code: "ISO 2022 IR 87".to_string(),
                }),
            ];

            // Act
            let actual = Cs::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 前後に空白を含む
        {
            // Arrange
            let input = " DERIVED  ";
            let expected = vec![Some(Cs {
                code: "DERIVED".to_string(),
            })];

            // Act
            let actual = Cs::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空白のみ
        {
            // Arrange
            let input = "  ";
            let expected = vec![None];

            // Act
            let actual = Cs::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 数字を含む
        {
            // Arrange
            let input = "ISO_IR_192";
            let expected = vec![Some(Cs {
                code: "ISO_IR_192".to_string(),
            })];

            // Act
            let actual = Cs::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: アンダースコアを含む
        {
            // Arrange
            let input = "PATIENT_ID";
            let expected = vec![Some(Cs {
                code: "PATIENT_ID".to_string(),
            })];

            // Act
            let actual = Cs::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: スペースを含む
        {
            // Arrange
            let input = "STUDY DATE";
            let expected = vec![Some(Cs {
                code: "STUDY DATE".to_string(),
            })];

            // Act
            let actual = Cs::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空文字列
        {
            // Arrange
            let input = "";
            let expected = vec![None];

            // Act
            let actual = Cs::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 最大長（16文字）
        {
            // Arrange
            let input = "1234567890ABCDEF";
            assert_eq!(input.len(), 16);
            let expected = vec![Some(Cs {
                code: input.to_string(),
            })];

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
            let expected = vec![
                Some(Cs {
                    code: "0123456789ABCDEF".to_string(),
                }),
                Some(Cs {
                    code: "0123456789ABCDEF".to_string(),
                }),
            ];

            // Act
            let actual = Cs::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 17文字の文字列（長すぎる）
        {
            // Arrange
            let input = "1234567890ABCDEFG";
            assert_eq!(input.len(), 17);

            // Act
            let result = Cs::from_string(input);

            // Assert
            match result.unwrap_err() {
                CsError::InvalidLength {
                    string,
                    byte_length,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(byte_length, 17);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 小文字を含む（不正な文字）
        {
            // Arrange
            let input = "primary";

            // Act
            let result = Cs::from_string(input);

            // Assert
            match result.unwrap_err() {
                CsError::InvalidCharacter {
                    string,
                    character,
                    position,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(character, 'p');
                    assert_eq!(position, 0);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 記号を含む（不正な文字）
        {
            // Arrange
            let input = "CODE-STRING";

            // Act
            let result = Cs::from_string(input);

            // Assert
            match result.unwrap_err() {
                CsError::InvalidCharacter {
                    string,
                    character,
                    position,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(character, '-');
                    assert_eq!(position, 4);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 日本語を含む（不正な文字）
        {
            // Arrange
            let input = "コード";

            // Act
            let result = Cs::from_string(input);

            // Assert
            match result.unwrap_err() {
                CsError::InvalidCharacter {
                    string,
                    character,
                    position,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(character, 'コ');
                    assert_eq!(position, 0);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_from_buf() {
        // 正常系: 通常
        {
            // Arrange
            let buf = b"ORIGINAL";
            let expected = vec![Some(Cs {
                code: "ORIGINAL".to_string(),
            })];

            // Act
            let actual = Cs::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空白パディングを含む
        {
            // Arrange
            let buf = b" DERIVED  ";
            let expected = vec![Some(Cs {
                code: "DERIVED".to_string(),
            })];

            // Act
            let actual = Cs::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数の値
        {
            // Arrange
            let buf = b"PRIMARY\\SECONDARY";
            let expected = vec![
                Some(Cs {
                    code: "PRIMARY".to_string(),
                }),
                Some(Cs {
                    code: "SECONDARY".to_string(),
                }),
            ];

            // Act
            let actual = Cs::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空の値を含むケース
        {
            // Arrange
            let buf = b"\\ISO 2022 IR 87";
            let expected = vec![
                None,
                Some(Cs {
                    code: "ISO 2022 IR 87".to_string(),
                }),
            ];

            // Act
            let actual = Cs::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 不正なUTF-8バイト列
        {
            // Arrange
            let buf = b"\xff\xfe";

            // Act
            let result = Cs::from_buf(buf);

            // Assert
            match result.unwrap_err() {
                CsError::InvalidUtf8(_) => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_to_string() {
        // Arrange
        let cs = Cs {
            code: "ORIGINAL".to_string(),
        };
        let expected = "ORIGINAL";

        // Act
        let actual = cs.to_string();

        // Assert
        assert_eq!(expected, actual);
    }
}
