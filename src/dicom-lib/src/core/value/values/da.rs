use chrono::NaiveDate;
use std::{
    fmt::{Display, Formatter},
    str::Utf8Error,
};
use thiserror::Error;

/// 日付（Date）
#[derive(Debug, PartialEq, Clone)]
pub struct Da {
    date: NaiveDate,
}

impl Da {
    const BYTE_LENGTH: usize = 8;

    pub fn date(&self) -> &NaiveDate {
        &self.date
    }

    /// 文字列から日付（DA）の値をパースします。
    ///
    /// この関数は、DICOM規格に準拠したDA型の文字列をパースし、
    /// バックスラッシュ(`\`)で区切られた複数の値を処理します。
    /// 日付は`YYYYMMDD`形式（8バイト）で、0-9の数字のみが許可されます。
    ///
    /// # 引数
    ///
    /// * `str` - パースする文字列。複数の値はバックスラッシュ(`\`)で区切られます。
    ///
    /// # 戻り値
    ///
    /// * `Ok(Vec<Option<Self>>)` - パースに成功した場合、DA値のベクタを返します。
    ///   空文字列は`None`として表現されます。
    /// * `Err(DaError)` - パースに失敗した場合、エラーを返します。
    ///   不正な文字が含まれている場合、8バイトでない場合、
    ///   または無効な日付（13月、2月30日など）の場合にエラーとなります。
    ///
    /// # パディング処理
    ///
    /// DICOM規格に従い、文字列が偶数バイト長で末尾が空白の場合、
    /// その空白は自動的に除去されます。
    ///
    /// # 例
    ///
    /// ```
    /// use dicom_lib::core::value::values::Da;
    /// use chrono::NaiveDate;
    ///
    /// // 単一の日付をパース
    /// let result = Da::from_string("20231115").unwrap();
    /// assert_eq!(result.len(), 1);
    /// assert_eq!(result[0].as_ref().unwrap().date(), &NaiveDate::from_ymd_opt(2023, 11, 15).unwrap());
    ///
    /// // 複数の日付をパース（バックスラッシュ区切り）
    /// let result = Da::from_string(r"20231115\20240229").unwrap();
    /// assert_eq!(result.len(), 2);
    /// assert_eq!(result[0].as_ref().unwrap().date(), &NaiveDate::from_ymd_opt(2023, 11, 15).unwrap());
    /// assert_eq!(result[1].as_ref().unwrap().date(), &NaiveDate::from_ymd_opt(2024, 2, 29).unwrap());
    ///
    /// // 空値を含むケース
    /// let result = Da::from_string(r"\20231115").unwrap();
    /// assert_eq!(result.len(), 2);
    /// assert!(result[0].is_none());
    /// assert_eq!(result[1].as_ref().unwrap().date(), &NaiveDate::from_ymd_opt(2023, 11, 15).unwrap());
    ///
    /// // 閏年の日付
    /// let result = Da::from_string("20240229").unwrap();
    /// assert_eq!(result[0].as_ref().unwrap().date(), &NaiveDate::from_ymd_opt(2024, 2, 29).unwrap());
    ///
    /// // 年初・年末の日付
    /// let result = Da::from_string("20230101").unwrap();
    /// assert_eq!(result[0].as_ref().unwrap().date(), &NaiveDate::from_ymd_opt(2023, 1, 1).unwrap());
    ///
    /// let result = Da::from_string("20231231").unwrap();
    /// assert_eq!(result[0].as_ref().unwrap().date(), &NaiveDate::from_ymd_opt(2023, 12, 31).unwrap());
    /// ```
    ///
    /// # エラー
    ///
    /// ```
    /// use dicom_lib::core::value::values::{Da, da::DaError};
    ///
    /// // 8バイトでない文字列
    /// let result = Da::from_string("2023111");
    /// assert!(matches!(result, Err(DaError::InvalidLength { .. })));
    ///
    /// // 数字以外の文字を含む
    /// let result = Da::from_string("2023A115");
    /// assert!(matches!(result, Err(DaError::InvalidCharacter { .. })));
    ///
    /// // 存在しない日付（13月）
    /// let result = Da::from_string("20231315");
    /// assert!(matches!(result, Err(DaError::ParseError { .. })));
    ///
    /// // 閏年でない年の2月29日
    /// let result = Da::from_string("20230229");
    /// assert!(matches!(result, Err(DaError::ParseError { .. })));
    /// ```
    pub fn from_string(str: &str) -> Result<Vec<Option<Self>>, DaError> {
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

    pub fn from_buf(buf: &[u8]) -> Result<Vec<Option<Self>>, DaError> {
        let str = str::from_utf8(buf).map_err(DaError::InvalidUtf8)?;
        Self::from_string(str)
    }

    fn from_string_single(str: &str) -> Result<Option<Self>, DaError> {
        if str.is_empty() {
            return Ok(None);
        }

        if str.len() != Self::BYTE_LENGTH {
            return Err(DaError::InvalidLength {
                string: str.to_string(),
                length: str.len(),
            });
        }

        // 各文字が数字であることを確認
        for (i, c) in str.chars().enumerate() {
            if !c.is_ascii_digit() {
                return Err(DaError::InvalidCharacter {
                    string: str.to_string(),
                    character: c,
                    position: i,
                });
            }
        }

        let date = NaiveDate::parse_from_str(str, "%Y%m%d").map_err(|e| DaError::ParseError {
            string: str.to_string(),
            error: e,
        })?;

        Ok(Some(Self { date }))
    }
}

impl Display for Da {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.date.format("%Y%m%d"))
    }
}

#[derive(Error, Debug)]
pub enum DaError {
    #[error("文字列の長さが8バイトではありません (文字列=\"{string}\", 長さ={length})")]
    InvalidLength { string: String, length: usize },

    #[error(
        "文字列に数字以外の文字が含まれています (文字列=\"{string}\", 文字='{character}', 位置={position})"
    )]
    InvalidCharacter {
        string: String,
        character: char,
        position: usize,
    },

    #[error("文字列から日付へのパースに失敗しました (文字列=\"{string}\"): {error}")]
    ParseError {
        string: String,
        error: chrono::ParseError,
    },

    #[error("バイト列をUTF-8として解釈できません: {0}")]
    InvalidUtf8(#[from] Utf8Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string() {
        // 正常系: 通常の日付
        {
            // Arrange
            let input = "20231115";
            let expected = vec![Some(Da {
                date: NaiveDate::from_ymd_opt(2023, 11, 15).unwrap(),
            })];

            // Act
            let actual = Da::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 閏年の日付
        {
            // Arrange
            let input = "20240229";
            let expected = vec![Some(Da {
                date: NaiveDate::from_ymd_opt(2024, 2, 29).unwrap(),
            })];

            // Act
            let actual = Da::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 年初の日付
        {
            // Arrange
            let input = "20230101";
            let expected = vec![Some(Da {
                date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            })];

            // Act
            let actual = Da::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 年末の日付
        {
            // Arrange
            let input = "20231231";
            let expected = vec![Some(Da {
                date: NaiveDate::from_ymd_opt(2023, 12, 31).unwrap(),
            })];

            // Act
            let actual = Da::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数の値（バックスラッシュ区切り）
        {
            // Arrange
            let input = r"20231115\20240229";
            let expected = vec![
                Some(Da {
                    date: NaiveDate::from_ymd_opt(2023, 11, 15).unwrap(),
                }),
                Some(Da {
                    date: NaiveDate::from_ymd_opt(2024, 2, 29).unwrap(),
                }),
            ];

            // Act
            let actual = Da::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空の値を含むケース
        {
            // Arrange
            let input = r"\20231115";
            let expected = vec![
                None,
                Some(Da {
                    date: NaiveDate::from_ymd_opt(2023, 11, 15).unwrap(),
                }),
            ];

            // Act
            let actual = Da::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空文字列
        {
            // Arrange
            let input = "";
            let expected = vec![None];

            // Act
            let actual = Da::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数値の最後の値が空白パディングされ、その値の長さが9になるケース
        //        詳しくは以下を参照。
        //        https://dicom.nema.org/medical/dicom/2025c/output/chtml/part05/sect_6.4.html
        {
            // Arrange
            let input = r"20231115\20240229 ";
            let expected = vec![
                Some(Da {
                    date: NaiveDate::from_ymd_opt(2023, 11, 15).unwrap(),
                }),
                Some(Da {
                    date: NaiveDate::from_ymd_opt(2024, 2, 29).unwrap(),
                }),
            ];

            // Act
            let actual = Da::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 7バイトの日付（短すぎる）
        {
            // Arrange
            let input = "2023111";

            // Act
            let result = Da::from_string(input);

            // Assert
            match result.unwrap_err() {
                DaError::InvalidLength { string, length } => {
                    assert_eq!(string, input);
                    assert_eq!(length, 7);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 9バイトの日付（長すぎる）
        {
            // Arrange
            let input = "202311151";

            // Act
            let result = Da::from_string(input);

            // Assert
            match result.unwrap_err() {
                DaError::InvalidLength { string, length } => {
                    assert_eq!(string, input);
                    assert_eq!(length, 9);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 数字以外の文字を含む日付
        {
            // Arrange
            let input = "2023A115";

            // Act
            let result = Da::from_string(input);

            // Assert
            match result.unwrap_err() {
                DaError::InvalidCharacter {
                    string,
                    character,
                    position,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(character, 'A');
                    assert_eq!(position, 4);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 存在しない日付（13月）
        {
            // Arrange
            let input = "20231315";

            // Act
            let result = Da::from_string(input);

            // Assert
            match result.unwrap_err() {
                DaError::ParseError { string, .. } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 存在しない日付（2月30日）
        {
            // Arrange
            let input = "20230230";

            // Act
            let result = Da::from_string(input);

            // Assert
            match result.unwrap_err() {
                DaError::ParseError { string, .. } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 閏年でない年の2月29日
        {
            // Arrange
            let input = "20230229";

            // Act
            let result = Da::from_string(input);

            // Assert
            match result.unwrap_err() {
                DaError::ParseError { string, .. } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_from_buf() {
        // 正常系: 通常の日付
        {
            // Arrange
            let buf = b"20231115";
            let expected = vec![Some(Da {
                date: NaiveDate::from_ymd_opt(2023, 11, 15).unwrap(),
            })];

            // Act
            let actual = Da::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数の値
        {
            // Arrange
            let buf = b"20231115\\20240229";
            let expected = vec![
                Some(Da {
                    date: NaiveDate::from_ymd_opt(2023, 11, 15).unwrap(),
                }),
                Some(Da {
                    date: NaiveDate::from_ymd_opt(2024, 2, 29).unwrap(),
                }),
            ];

            // Act
            let actual = Da::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空の値を含むケース
        {
            // Arrange
            let buf = b"\\20231115";
            let expected = vec![
                None,
                Some(Da {
                    date: NaiveDate::from_ymd_opt(2023, 11, 15).unwrap(),
                }),
            ];

            // Act
            let actual = Da::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 不正なUTF-8バイト列
        {
            // Arrange
            let buf = b"\xff\xfe\xff\xfe\xff\xfe\xff\xfe";

            // Act
            let result = Da::from_buf(buf);

            // Assert
            match result.unwrap_err() {
                DaError::InvalidUtf8(_) => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_to_string() {
        // Arrange
        let da = Da {
            date: NaiveDate::from_ymd_opt(2023, 11, 15).unwrap(),
        };
        let expected = "20231115";

        // Act
        let actual = da.to_string();

        // Assert
        assert_eq!(expected, actual);
    }
}
