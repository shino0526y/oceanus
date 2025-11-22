use crate::core::value::{self, SpecificCharacterSet};
use std::fmt::{Display, Formatter};
use thiserror::Error;

/// 短い文字列（Short String）
#[derive(Debug, PartialEq, Clone)]
pub struct Sh {
    string: String,
}

impl Sh {
    const MAX_CHAR_COUNT: usize = 16;

    pub fn string(&self) -> &str {
        &self.string
    }

    /// 文字列から短い文字列（SH）の値をパースします。
    ///
    /// この関数は、DICOM規格に準拠したSH型の文字列をパースし、
    /// バックスラッシュ(`\`)で区切られた複数の値を処理します。
    /// 各値は最大16文字までで、制御文字（`\`を除く）以外の文字が許可されます。
    /// マルチバイト文字（日本語など）も使用可能です。
    ///
    /// # 引数
    ///
    /// * `str` - パースする文字列。複数の値はバックスラッシュ(`\`)で区切られます。
    ///
    /// # 戻り値
    ///
    /// * `Ok(Vec<Option<Self>>)` - パースに成功した場合、SH値のベクタを返します。
    ///   空白のみの値や空文字列は`None`として表現されます。
    /// * `Err(ShError)` - パースに失敗した場合、エラーを返します。
    ///   16文字を超える値がある場合にエラーとなります。
    ///
    /// # パディング処理
    ///
    /// DICOM規格に従い、文字列が偶数バイト長で末尾が空白の場合、
    /// その空白は自動的に除去されます。また、各値の前後の空白もトリミングされます。
    ///
    /// # 例
    ///
    /// ```
    /// use dicom_lib::core::value::values::Sh;
    ///
    /// // 単一の文字列をパース
    /// let result = Sh::from_string("SHORT").unwrap();
    /// assert_eq!(result.len(), 1);
    /// assert_eq!(result[0].as_ref().unwrap().string(), "SHORT");
    ///
    /// // 複数の値をパース（バックスラッシュ区切り）
    /// let result = Sh::from_string(r"VALUE1\VALUE2").unwrap();
    /// assert_eq!(result.len(), 2);
    /// assert_eq!(result[0].as_ref().unwrap().string(), "VALUE1");
    /// assert_eq!(result[1].as_ref().unwrap().string(), "VALUE2");
    ///
    /// // 空値を含むケース
    /// let result = Sh::from_string(r"\VALUE1").unwrap();
    /// assert_eq!(result.len(), 2);
    /// assert!(result[0].is_none());
    /// assert_eq!(result[1].as_ref().unwrap().string(), "VALUE1");
    ///
    /// // 前後の空白は自動的にトリミングされる
    /// let result = Sh::from_string(" TRIMMED  ").unwrap();
    /// assert_eq!(result[0].as_ref().unwrap().string(), "TRIMMED");
    ///
    /// // 最大長（16文字）
    /// let input = "1234567890ABCDEF";
    /// let result = Sh::from_string(input).unwrap();
    /// assert_eq!(result[0].as_ref().unwrap().string(), input);
    /// ```
    ///
    /// # エラー
    ///
    /// ```
    /// use dicom_lib::core::value::values::{Sh, sh::ShError};
    ///
    /// // 17文字を超える文字列
    /// let input = "1234567890ABCDEFG";
    /// let result = Sh::from_string(input);
    /// assert!(matches!(result, Err(ShError::InvalidLength { .. })));
    /// ```
    pub fn from_string(str: &str) -> Result<Vec<Option<Self>>, ShError> {
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

    pub fn from_buf_lossy(
        buf: &[u8],
        char_set: SpecificCharacterSet,
    ) -> Result<Vec<Option<Self>>, ShError> {
        let str = value::generate_string_lossy(buf, char_set);
        Self::from_string(&str)
    }

    fn from_string_single(str: &str) -> Result<Option<Self>, ShError> {
        if str.chars().count() > Self::MAX_CHAR_COUNT {
            return Err(ShError::InvalidLength {
                string: str.to_string(),
                char_count: str.chars().count(),
            });
        }

        let trimmed = str.trim_matches(' ');
        if trimmed.is_empty() {
            return Ok(None);
        }

        Ok(Some(Self {
            string: trimmed.to_string(),
        }))
    }
}

impl Display for Sh {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string)
    }
}

#[derive(Error, Debug)]
pub enum ShError {
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
            let input = "SHORT";
            let expected = vec![Some(Sh {
                string: "SHORT".to_string(),
            })];

            // Act
            let actual = Sh::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数の値（バックスラッシュ区切り）
        {
            // Arrange
            let input = r"VALUE1\VALUE2";
            let expected = vec![
                Some(Sh {
                    string: "VALUE1".to_string(),
                }),
                Some(Sh {
                    string: "VALUE2".to_string(),
                }),
            ];

            // Act
            let actual = Sh::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空の値を含むケース
        {
            // Arrange
            let input = r"\VALUE1";
            let expected = vec![
                None,
                Some(Sh {
                    string: "VALUE1".to_string(),
                }),
            ];

            // Act
            let actual = Sh::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 前後に空白を含む
        {
            // Arrange
            let input = " TRIMMED  ";
            let expected = vec![Some(Sh {
                string: "TRIMMED".to_string(),
            })];

            // Act
            let actual = Sh::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空文字列
        {
            // Arrange
            let input = "";
            let expected = vec![None];

            // Act
            let actual = Sh::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 最大長（16文字）
        {
            // Arrange
            let input = "1234567890ABCDEF";
            assert_eq!(input.chars().count(), 16);
            let expected = vec![Some(Sh {
                string: input.to_string(),
            })];

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
            let input = r"0123456789ABCDEF\0123456789ABCDEF ";
            let expected = vec![
                Some(Sh {
                    string: "0123456789ABCDEF".to_string(),
                }),
                Some(Sh {
                    string: "0123456789ABCDEF".to_string(),
                }),
            ];

            // Act
            let actual = Sh::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 17文字の文字列（長すぎる）
        {
            // Arrange
            let input = "1234567890ABCDEFG";
            assert_eq!(input.chars().count(), 17);

            // Act
            let result = Sh::from_string(input);

            // Assert
            match result.unwrap_err() {
                ShError::InvalidLength { string, char_count } => {
                    assert_eq!(string, input);
                    assert_eq!(char_count, 17);
                }
            }
        }
    }

    #[test]
    fn test_from_buf_lossy() {
        // 正常系: 通常
        {
            // Arrange
            let buf = b"SHORT";
            let expected = vec![Some(Sh {
                string: "SHORT".to_string(),
            })];

            // Act
            let actual = Sh::from_buf_lossy(buf, SpecificCharacterSet::None).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空白パディングを含む
        {
            // Arrange
            let buf = b" SHORT  ";
            let expected = vec![Some(Sh {
                string: "SHORT".to_string(),
            })];

            // Act
            let actual = Sh::from_buf_lossy(buf, SpecificCharacterSet::None).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 日本語を含む文字列（ISO 2022 IR 6＆ISO 2022 IR 87）
        {
            // Arrange
            let buf = [0x1b, 0x24, 0x42, 0x3b, 0x33, 0x45, 0x44, 0x1b, 0x28, 0x42];
            let expected = vec![Some(Sh {
                string: "山田".to_string(),
            })];

            // Act
            let actual =
                Sh::from_buf_lossy(&buf, SpecificCharacterSet::Iso2022Ir6AndIso2022Ir87).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: UTF-8の日本語文字列
        {
            // Arrange
            let buf = "山田".as_bytes();
            let expected = vec![Some(Sh {
                string: "山田".to_string(),
            })];

            // Act
            let actual = Sh::from_buf_lossy(buf, SpecificCharacterSet::IsoIr192).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数の値
        {
            // Arrange
            let buf = b"VALUE1\\VALUE2";
            let expected = vec![
                Some(Sh {
                    string: "VALUE1".to_string(),
                }),
                Some(Sh {
                    string: "VALUE2".to_string(),
                }),
            ];

            // Act
            let actual = Sh::from_buf_lossy(buf, SpecificCharacterSet::None).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空の値を含むケース
        {
            // Arrange
            let buf = b"\\VALUE1";
            let expected = vec![
                None,
                Some(Sh {
                    string: "VALUE1".to_string(),
                }),
            ];

            // Act
            let actual = Sh::from_buf_lossy(buf, SpecificCharacterSet::None).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn test_to_string() {
        // Arrange
        let sh = Sh {
            string: "SHORT".to_string(),
        };
        let expected = "SHORT";

        // Act
        let actual = sh.to_string();

        // Assert
        assert_eq!(expected, actual);
    }
}
