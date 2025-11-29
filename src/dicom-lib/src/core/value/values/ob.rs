use std::fmt::{Display, Formatter};

/// その他のバイト（Other Byte）
///
/// OBは転送構文によってエンコーディングが指定されるオクテットストリームです。
/// バイト順序に影響されない（byte ordering insensitive）VRです。
/// 偶数長にするために必要に応じて単一のNULLバイト（00H）でパディングされます。
#[derive(Debug, PartialEq, Clone)]
pub struct Ob {
    values: Vec<u8>,
}

impl Ob {
    pub fn values(&self) -> &[u8] {
        &self.values
    }

    /// バイト列からOB（Other Byte）の値を作成します。
    ///
    /// この関数は、DICOM規格に準拠したOB型のバイナリデータを作成します。
    /// OBは転送構文によってエンコーディングが指定されるオクテットストリームで、
    /// 任意のバイナリデータを格納できます。
    /// バイト順序に影響されないVRです。
    ///
    /// # 引数
    ///
    /// * `buf` - バイト列。任意の長さが許可されます。
    ///
    /// # 戻り値
    ///
    /// * `Self` - OB値を含む構造体を返します。
    ///   空のバッファの場合は、空のVec<u8>を持つ構造体を返します。
    ///
    /// # 例
    ///
    /// ```
    /// use dicom_lib::core::value::values::Ob;
    ///
    /// // 任意のバイト列を格納
    /// let buf = [0x01, 0x02, 0x03, 0x04, 0x05];
    /// let result = Ob::from_buf(&buf);
    /// assert_eq!(result.values(), &[0x01, 0x02, 0x03, 0x04, 0x05]);
    ///
    /// // 空のバッファ
    /// let buf = [];
    /// let result = Ob::from_buf(&buf);
    /// assert_eq!(result.values(), &[]);
    ///
    /// // 奇数長のバイト列（パディングなし）
    /// let buf = [0xff, 0xee, 0xdd];
    /// let result = Ob::from_buf(&buf);
    /// assert_eq!(result.values(), &[0xff, 0xee, 0xdd]);
    /// ```
    pub fn from_buf(buf: &[u8]) -> Self {
        Self {
            values: buf.to_vec(),
        }
    }
}

impl Display for Ob {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.values.is_empty() {
            return write!(f, "[]");
        }

        // バイト数が多い場合は最初の数バイトのみ表示
        const MAX_DISPLAY_BYTES: usize = 16;

        write!(f, "[")?;

        if self.values.len() <= MAX_DISPLAY_BYTES {
            write!(f, "{}", self.values[0])?;
            for byte in &self.values[1..] {
                write!(f, ", {}", byte)?;
            }
        } else {
            // 最初の数バイトと省略記号、最後のバイト、合計バイト数を表示
            for byte in &self.values[..MAX_DISPLAY_BYTES] {
                write!(f, "{}, ", byte)?;
            }
            write!(f, "... ({} values total)", self.values.len())?;
        }

        write!(f, "]")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_buf() {
        // 正常系: 通常のバイト列
        {
            // Arrange
            let buf = [0x01, 0x02, 0x03, 0x04];
            let expected = Ob {
                values: vec![1, 2, 3, 4],
            };

            // Act
            let actual = Ob::from_buf(&buf);

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空のバッファ
        {
            // Arrange
            let buf = [];
            let expected = Ob { values: vec![] };

            // Act
            let actual = Ob::from_buf(&buf);

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 単一バイト
        {
            // Arrange
            let buf = [0xff];
            let expected = Ob { values: vec![255] };

            // Act
            let actual = Ob::from_buf(&buf);

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: NULLバイトを含む
        {
            // Arrange
            let buf = [0x01, 0x00, 0x02, 0x00];
            let expected = Ob {
                values: vec![1, 0, 2, 0],
            };

            // Act
            let actual = Ob::from_buf(&buf);

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 奇数長
        {
            // Arrange
            let buf = [0x01, 0x02, 0x03];
            let expected = Ob {
                values: vec![1, 2, 3],
            };

            // Act
            let actual = Ob::from_buf(&buf);

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 偶数長
        {
            // Arrange
            let buf = [0x01, 0x02, 0x03, 0x04];
            let expected = Ob {
                values: vec![1, 2, 3, 4],
            };

            // Act
            let actual = Ob::from_buf(&buf);

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 全てゼロ
        {
            // Arrange
            let buf = [0x00, 0x00, 0x00, 0x00];
            let expected = Ob {
                values: vec![0, 0, 0, 0],
            };

            // Act
            let actual = Ob::from_buf(&buf);

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 全て0xff
        {
            // Arrange
            let buf = [0xff, 0xff, 0xff, 0xff];
            let expected = Ob {
                values: vec![255, 255, 255, 255],
            };

            // Act
            let actual = Ob::from_buf(&buf);

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 大きなバッファ
        {
            // Arrange
            let buf: Vec<u8> = (0..100).map(|i| (i % 256) as u8).collect();
            let expected = Ob {
                values: buf.clone(),
            };

            // Act
            let actual = Ob::from_buf(&buf);

            // Assert
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn test_to_string() {
        // 正常系: 短いバイト列
        {
            // Arrange
            let ob = Ob {
                values: vec![1, 2, 3, 4],
            };
            let expected = "[1, 2, 3, 4]";

            // Act
            let actual = ob.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空のバイト列
        {
            // Arrange
            let ob = Ob { values: vec![] };
            let expected = "[]";

            // Act
            let actual = ob.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 単一バイト
        {
            // Arrange
            let ob = Ob { values: vec![255] };
            let expected = "[255]";

            // Act
            let actual = ob.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 16バイト（境界値）
        {
            // Arrange
            let ob = Ob {
                values: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
            };
            let expected = "[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]";

            // Act
            let actual = ob.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 17バイト以上（省略表示）
        {
            // Arrange
            let ob = Ob {
                values: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            };
            let expected =
                "[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, ... (17 values total)]";

            // Act
            let actual = ob.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 大きなバイト列
        {
            // Arrange
            let bytes: Vec<u8> = (0..100).map(|i| (i % 256) as u8).collect();
            let ob = Ob { values: bytes };
            let expected =
                "[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, ... (100 values total)]";

            // Act
            let actual = ob.to_string();

            // Assert
            assert_eq!(actual, expected);
        }
    }
}
