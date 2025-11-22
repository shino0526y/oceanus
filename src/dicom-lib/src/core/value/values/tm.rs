use chrono::{NaiveTime, Timelike};
use std::{
    fmt::{Display, Formatter},
    str::Utf8Error,
};
use thiserror::Error;

/// 時刻（Time）
#[derive(Debug, PartialEq, Clone)]
pub struct Tm {
    time: NaiveTime,
}

impl Tm {
    const MAX_BYTE_LENGTH: usize = 64;

    pub fn time(&self) -> &NaiveTime {
        &self.time
    }

    /// 文字列から時刻（TM）の値をパースします。
    ///
    /// この関数は、DICOM規格に準拠したTM型の文字列をパースし、
    /// バックスラッシュ(`\`)で区切られた複数の値を処理します。
    /// 時刻は`HH`、`HHMM`、`HHMMSS`、または`HHMMSS.FFFFFF`形式で、
    /// 0-9の数字と小数点（`.`）のみが許可されます。
    /// 小数秒は最大6桁（マイクロ秒精度）まで指定できます。
    ///
    /// # 引数
    ///
    /// * `str` - パースする文字列。複数の値はバックスラッシュ(`\`)で区切られます。
    ///
    /// # 戻り値
    ///
    /// * `Ok(Vec<Option<Self>>)` - パースに成功した場合、TM値のベクタを返します。
    ///   空文字列は`None`として表現されます。
    /// * `Err(TmError)` - パースに失敗した場合、エラーを返します。
    ///   不正な文字が含まれている場合、不正な形式の場合、
    ///   または無効な時刻（24時、60分、61秒など）の場合にエラーとなります。
    ///
    /// # パディング処理
    ///
    /// DICOM規格に従い、文字列が偶数バイト長で末尾が空白の場合、
    /// その空白は自動的に除去されます。
    ///
    /// # 形式
    ///
    /// - `HH`: 時のみ（例: `10` → 10:00:00）
    /// - `HHMM`: 時と分（例: `1030` → 10:30:00）
    /// - `HHMMSS`: 時、分、秒（例: `103045` → 10:30:45）
    /// - `HH.FFFFFF`: 時と小数秒（例: `10.5` → 10:00:00.500000）
    /// - `HHMM.FFFFFF`: 時、分、小数秒（例: `1030.123` → 10:30:00.123000）
    /// - `HHMMSS.FFFFFF`: 時、分、秒、小数秒（例: `103045.123456` → 10:30:45.123456）
    ///
    /// 小数秒は1～6桁で指定でき、不足分は0でパディングされます。
    ///
    /// # 例
    ///
    /// ```
    /// use dicom_lib::core::value::values::Tm;
    /// use chrono::NaiveTime;
    ///
    /// // HHMMSS形式（時、分、秒）
    /// let result = Tm::from_string("103045").unwrap();
    /// assert_eq!(result.len(), 1);
    /// assert_eq!(result[0].as_ref().unwrap().time(), &NaiveTime::from_hms_opt(10, 30, 45).unwrap());
    ///
    /// // 小数秒付き（マイクロ秒精度）
    /// let result = Tm::from_string("103045.123456").unwrap();
    /// assert_eq!(result[0].as_ref().unwrap().time(), &NaiveTime::from_hms_micro_opt(10, 30, 45, 123456).unwrap());
    ///
    /// // 小数秒付き（1桁）
    /// let result = Tm::from_string("103045.1").unwrap();
    /// assert_eq!(result[0].as_ref().unwrap().time(), &NaiveTime::from_hms_micro_opt(10, 30, 45, 100000).unwrap());
    ///
    /// // HH形式（時のみ）
    /// let result = Tm::from_string("10").unwrap();
    /// assert_eq!(result[0].as_ref().unwrap().time(), &NaiveTime::from_hms_opt(10, 0, 0).unwrap());
    ///
    /// // HHMM形式（時と分）
    /// let result = Tm::from_string("1030").unwrap();
    /// assert_eq!(result[0].as_ref().unwrap().time(), &NaiveTime::from_hms_opt(10, 30, 0).unwrap());
    ///
    /// // 深夜0時
    /// let result = Tm::from_string("000000").unwrap();
    /// assert_eq!(result[0].as_ref().unwrap().time(), &NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    ///
    /// // 深夜直前
    /// let result = Tm::from_string("235959").unwrap();
    /// assert_eq!(result[0].as_ref().unwrap().time(), &NaiveTime::from_hms_opt(23, 59, 59).unwrap());
    ///
    /// // 複数の時刻をパース（バックスラッシュ区切り）
    /// let result = Tm::from_string(r"103045\120000").unwrap();
    /// assert_eq!(result.len(), 2);
    /// assert_eq!(result[0].as_ref().unwrap().time(), &NaiveTime::from_hms_opt(10, 30, 45).unwrap());
    /// assert_eq!(result[1].as_ref().unwrap().time(), &NaiveTime::from_hms_opt(12, 0, 0).unwrap());
    ///
    /// // 空値を含むケース
    /// let result = Tm::from_string(r"\103045").unwrap();
    /// assert_eq!(result.len(), 2);
    /// assert!(result[0].is_none());
    /// assert_eq!(result[1].as_ref().unwrap().time(), &NaiveTime::from_hms_opt(10, 30, 45).unwrap());
    /// ```
    ///
    /// # エラー
    ///
    /// ```
    /// use dicom_lib::core::value::values::{Tm, tm::TmError};
    ///
    /// // 不正な長さ（3文字、5文字など）
    /// let result = Tm::from_string("103");
    /// assert!(matches!(result, Err(TmError::InvalidFormat { .. })));
    ///
    /// // 数字以外の文字を含む
    /// let result = Tm::from_string("10A045");
    /// assert!(matches!(result, Err(TmError::InvalidCharacter { .. })));
    ///
    /// // 小数点が複数
    /// let result = Tm::from_string("103.45.123");
    /// assert!(matches!(result, Err(TmError::InvalidFormat { .. })));
    ///
    /// // 小数秒が7桁以上
    /// let result = Tm::from_string("103045.1234567");
    /// assert!(matches!(result, Err(TmError::InvalidFormat { .. })));
    ///
    /// // 存在しない時刻（24時）
    /// let result = Tm::from_string("240000");
    /// assert!(matches!(result, Err(TmError::ParseError { .. })));
    ///
    /// // 存在しない時刻（60分）
    /// let result = Tm::from_string("106000");
    /// assert!(matches!(result, Err(TmError::ParseError { .. })));
    ///
    /// // 存在しない時刻（61秒）
    /// let result = Tm::from_string("103061");
    /// assert!(matches!(result, Err(TmError::ParseError { .. })));
    /// ```
    pub fn from_string(str: &str) -> Result<Vec<Option<Self>>, TmError> {
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

    pub fn from_buf(buf: &[u8]) -> Result<Vec<Option<Self>>, TmError> {
        let str = str::from_utf8(buf).map_err(TmError::InvalidUtf8)?;
        Self::from_string(str)
    }

    fn from_string_single(str: &str) -> Result<Option<Self>, TmError> {
        if str.len() > Self::MAX_BYTE_LENGTH {
            return Err(TmError::InvalidLength {
                string: str.to_string(),
                byte_length: str.len(),
            });
        }

        if str.is_empty() {
            return Ok(None);
        }

        // 小数点が含まれる場合
        if str.contains('.') {
            // "HHMMSS.FFFFFF"形式の検証
            let parts: Vec<&str> = str.split('.').collect();
            if parts.len() != 2 {
                return Err(TmError::InvalidFormat {
                    string: str.to_string(),
                });
            }

            let time_part = parts[0];
            let frac_part = parts[1];

            // 時刻部分は2、4、または6文字（HH、HHMM、HHMMSS）
            if !matches!(time_part.len(), 2 | 4 | 6) {
                return Err(TmError::InvalidFormat {
                    string: str.to_string(),
                });
            }
            // 小数部は1-6文字
            if frac_part.is_empty() || frac_part.len() > 6 {
                return Err(TmError::InvalidFormat {
                    string: str.to_string(),
                });
            }

            // 時刻部分が数字のみであることを確認
            for (i, c) in time_part.chars().enumerate() {
                if !c.is_ascii_digit() {
                    return Err(TmError::InvalidCharacter {
                        string: str.to_string(),
                        character: c,
                        position: i,
                    });
                }
            }
            // 小数部が数字のみであることを確認
            for (i, c) in frac_part.chars().enumerate() {
                if !c.is_ascii_digit() {
                    return Err(TmError::InvalidCharacter {
                        string: str.to_string(),
                        character: c,
                        position: time_part.len() + 1 + i,
                    });
                }
            }

            // 時刻部分の長さに応じて、秒と分を0で埋める
            let padded_time = match time_part.len() {
                2 => format!("{}0000", time_part), // HH -> HH0000
                4 => format!("{}00", time_part),   // HHMM -> HHMM00
                6 => time_part.to_string(),        // HHMMSS -> HHMMSS
                _ => unreachable!(),
            };

            // マイクロ秒まで拡張（6桁にパディング）
            let frac_padded = format!("{:0<6}", frac_part);
            let time_str = format!("{}.{}", padded_time, frac_padded);

            let time = NaiveTime::parse_from_str(&time_str, "%H%M%S%.f").map_err(|e| {
                TmError::ParseError {
                    string: str.to_string(),
                    error: e,
                }
            })?;

            Ok(Some(Self { time }))
        } else {
            // 小数点なし: "HH"、"HHMM"、"HHMMSS"形式の検証
            if !matches!(str.len(), 2 | 4 | 6) {
                return Err(TmError::InvalidFormat {
                    string: str.to_string(),
                });
            }

            // 各文字が数字であることを確認
            for (i, c) in str.chars().enumerate() {
                if !c.is_ascii_digit() {
                    return Err(TmError::InvalidCharacter {
                        string: str.to_string(),
                        character: c,
                        position: i,
                    });
                }
            }

            // 長さに応じて、秒と分を0で埋める
            let padded_time = match str.len() {
                2 => format!("{}0000", str), // HH -> HH0000
                4 => format!("{}00", str),   // HHMM -> HHMM00
                6 => str.to_string(),        // HHMMSS -> HHMMSS
                _ => unreachable!(),
            };

            let time = NaiveTime::parse_from_str(&padded_time, "%H%M%S").map_err(|e| {
                TmError::ParseError {
                    string: str.to_string(),
                    error: e,
                }
            })?;

            Ok(Some(Self { time }))
        }
    }
}

impl Display for Tm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // マイクロ秒がある場合は表示、ない場合はHHMMSSのみ
        if self.time.nanosecond() == 0 {
            write!(f, "{}", self.time.format("%H%M%S"))
        } else {
            // マイクロ秒を取得（ナノ秒を1000で割る）
            let micros = self.time.nanosecond() / 1000;
            // 末尾のゼロを削除
            let frac_str = format!("{:06}", micros).trim_end_matches('0').to_string();
            write!(f, "{}.{}", self.time.format("%H%M%S"), frac_str)
        }
    }
}

#[derive(Error, Debug)]
pub enum TmError {
    #[error("文字列の長さが14バイトを超えています (文字列=\"{string}\", 長さ={byte_length})")]
    InvalidLength { string: String, byte_length: usize },

    #[error("文字列の形式が不正です (文字列=\"{string}\")")]
    InvalidFormat { string: String },

    #[error(
        "文字列に不正な文字が含まれています (文字列=\"{string}\", 文字='{character}', 位置={position})"
    )]
    InvalidCharacter {
        string: String,
        character: char,
        position: usize,
    },

    #[error("文字列から時刻へのパースに失敗しました (時刻=\"{string}\"): {error}")]
    ParseError {
        string: String,
        error: chrono::ParseError,
    },

    #[error("バイト列をUTF-8として解釈できません: {0}")]
    InvalidUtf8(Utf8Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string() {
        // 正常系: 通常の時刻
        {
            // Arrange
            let input = "103045";
            let expected = vec![Some(Tm {
                time: NaiveTime::from_hms_opt(10, 30, 45).unwrap(),
            })];

            // Act
            let actual = Tm::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 小数秒付き（1桁）
        {
            // Arrange
            let input = "103045.1";
            let expected = vec![Some(Tm {
                time: NaiveTime::from_hms_micro_opt(10, 30, 45, 100000).unwrap(),
            })];

            // Act
            let actual = Tm::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 小数秒付き（6桁、最大精度）
        {
            // Arrange
            let input = "103045.123456";
            let expected = vec![Some(Tm {
                time: NaiveTime::from_hms_micro_opt(10, 30, 45, 123456).unwrap(),
            })];

            // Act
            let actual = Tm::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 深夜0時
        {
            // Arrange
            let input = "000000";
            let expected = vec![Some(Tm {
                time: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            })];

            // Act
            let actual = Tm::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 深夜直前
        {
            // Arrange
            let input = "235959";
            let expected = vec![Some(Tm {
                time: NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
            })];

            // Act
            let actual = Tm::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 正午
        {
            // Arrange
            let input = "120000";
            let expected = vec![Some(Tm {
                time: NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            })];

            // Act
            let actual = Tm::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数の値（バックスラッシュ区切り）
        {
            // Arrange
            let input = r"103045\120000";
            let expected = vec![
                Some(Tm {
                    time: NaiveTime::from_hms_opt(10, 30, 45).unwrap(),
                }),
                Some(Tm {
                    time: NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
                }),
            ];

            // Act
            let actual = Tm::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空の値を含むケース
        {
            // Arrange
            let input = r"\103045";
            let expected = vec![
                None,
                Some(Tm {
                    time: NaiveTime::from_hms_opt(10, 30, 45).unwrap(),
                }),
            ];

            // Act
            let actual = Tm::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空文字列
        {
            // Arrange
            let input = "";
            let expected = vec![None];

            // Act
            let actual = Tm::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数値の最後の値が空白パディングされ、その値の長さが7になるケース
        //        詳しくは以下を参照。
        //        https://dicom.nema.org/medical/dicom/2025c/output/chtml/part05/sect_6.4.html
        {
            // Arrange
            let input = r"103045\120000 ";
            let expected = vec![
                Some(Tm {
                    time: NaiveTime::from_hms_opt(10, 30, 45).unwrap(),
                }),
                Some(Tm {
                    time: NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
                }),
            ];

            // Act
            let actual = Tm::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 5文字の時刻（短すぎる）
        {
            // Arrange
            let input = "10304";

            // Act
            let result = Tm::from_string(input);

            // Assert
            match result.unwrap_err() {
                TmError::InvalidFormat { string, .. } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 14文字の時刻（長すぎる）
        {
            // Arrange
            let input = "103045.1234567";

            // Act
            let result = Tm::from_string(input);

            // Assert
            match result.unwrap_err() {
                TmError::InvalidFormat { string, .. } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 数字以外の文字を含む時刻
        {
            // Arrange
            let input = "10A045";

            // Act
            let result = Tm::from_string(input);

            // Assert
            match result.unwrap_err() {
                TmError::InvalidCharacter {
                    string,
                    character,
                    position,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(character, 'A');
                    assert_eq!(position, 2);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 不正な形式（小数点が複数）
        {
            // Arrange
            let input = "103.45.123";

            // Act
            let result = Tm::from_string(input);

            // Assert
            match result.unwrap_err() {
                TmError::InvalidFormat { string } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 存在しない時刻（24時）
        {
            // Arrange
            let input = "240000";

            // Act
            let result = Tm::from_string(input);

            // Assert
            match result.unwrap_err() {
                TmError::ParseError { string, .. } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 存在しない時刻（60分）
        {
            // Arrange
            let input = "106000";

            // Act
            let result = Tm::from_string(input);

            // Assert
            match result.unwrap_err() {
                TmError::ParseError { string, .. } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 存在しない時刻（61秒）
        {
            // Arrange
            let input = "103061";

            // Act
            let result = Tm::from_string(input);

            // Assert
            match result.unwrap_err() {
                TmError::ParseError { string, .. } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 正常系: HH形式（時のみ）
        {
            // Arrange
            let input = "10";
            let expected = vec![Some(Tm {
                time: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
            })];

            // Act
            let actual = Tm::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: HHMM形式（時と分）
        {
            // Arrange
            let input = "1030";
            let expected = vec![Some(Tm {
                time: NaiveTime::from_hms_opt(10, 30, 0).unwrap(),
            })];

            // Act
            let actual = Tm::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: HH.FFFFFF形式（時と小数秒）
        {
            // Arrange
            let input = "10.5";
            let expected = vec![Some(Tm {
                time: NaiveTime::from_hms_micro_opt(10, 0, 0, 500000).unwrap(),
            })];

            // Act
            let actual = Tm::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: HHMM.FFFFFF形式（時、分、小数秒）
        {
            // Arrange
            let input = "1030.123456";
            let expected = vec![Some(Tm {
                time: NaiveTime::from_hms_micro_opt(10, 30, 0, 123456).unwrap(),
            })];

            // Act
            let actual = Tm::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 小数秒が0の場合（末尾の0は削除される）
        {
            // Arrange
            let input = "103045.000000";
            let expected = vec![Some(Tm {
                time: NaiveTime::from_hms_opt(10, 30, 45).unwrap(),
            })];

            // Act
            let actual = Tm::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 3文字の時刻（不正な長さ）
        {
            // Arrange
            let input = "103";

            // Act
            let result = Tm::from_string(input);

            // Assert
            match result.unwrap_err() {
                TmError::InvalidFormat { string } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: 5文字の時刻（不正な長さ）
        {
            // Arrange
            let input = "10304";

            // Act
            let result = Tm::from_string(input);

            // Assert
            match result.unwrap_err() {
                TmError::InvalidFormat { string } => {
                    assert_eq!(string, input);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_from_buf() {
        // 正常系: 通常の時刻
        {
            // Arrange
            let buf = b"103045";
            let expected = vec![Some(Tm {
                time: NaiveTime::from_hms_opt(10, 30, 45).unwrap(),
            })];

            // Act
            let actual = Tm::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 小数秒付き
        {
            // Arrange
            let buf = b"103045.123";
            let expected = vec![Some(Tm {
                time: NaiveTime::from_hms_micro_opt(10, 30, 45, 123000).unwrap(),
            })];

            // Act
            let actual = Tm::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数の値
        {
            // Arrange
            let buf = b"103045\\120000";
            let expected = vec![
                Some(Tm {
                    time: NaiveTime::from_hms_opt(10, 30, 45).unwrap(),
                }),
                Some(Tm {
                    time: NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
                }),
            ];

            // Act
            let actual = Tm::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空の値を含むケース
        {
            // Arrange
            let buf = b"\\103045";
            let expected = vec![
                None,
                Some(Tm {
                    time: NaiveTime::from_hms_opt(10, 30, 45).unwrap(),
                }),
            ];

            // Act
            let actual = Tm::from_buf(buf).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 不正なUTF-8バイト列
        {
            // Arrange
            let buf = b"\xff\xfe\xff\xfe\xff\xfe";

            // Act
            let result = Tm::from_buf(buf);

            // Assert
            match result.unwrap_err() {
                TmError::InvalidUtf8(_) => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_to_string() {
        // 正常系: 秒のみ
        {
            // Arrange
            let tm = Tm {
                time: NaiveTime::from_hms_opt(10, 30, 45).unwrap(),
            };
            let expected = "103045";

            // Act
            let actual = tm.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 小数秒付き（末尾のゼロは削除）
        {
            // Arrange
            let tm = Tm {
                time: NaiveTime::from_hms_micro_opt(10, 30, 45, 123000).unwrap(),
            };
            let expected = "103045.123";

            // Act
            let actual = tm.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 小数秒付き（6桁）
        {
            // Arrange
            let tm = Tm {
                time: NaiveTime::from_hms_micro_opt(10, 30, 45, 123456).unwrap(),
            };
            let expected = "103045.123456";

            // Act
            let actual = tm.to_string();

            // Assert
            assert_eq!(expected, actual);
        }
    }
}
