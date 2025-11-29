mod pn_component_group;

use crate::core::value::{self, SpecificCharacterSet};
pub use pn_component_group::PnComponentGroup;
use std::fmt::{Display, Formatter};
use thiserror::Error;

/// 人名（Person Name）
#[derive(Debug, PartialEq)]
pub struct Pn {
    /// シングルバイト文字 コンポーネントグループ
    ///
    /// # 例
    /// `"Yamada^Taro"`
    single_byte_name: Option<PnComponentGroup>,
    /// 表意文字 コンポーネントグループ
    ///
    /// # 例
    /// `"山田^太郎"`
    ideographic_name: Option<PnComponentGroup>,
    /// 表音文字 コンポーネントグループ
    ///
    /// # 例
    /// `"やまだ^たろう"`
    phonetic_name: Option<PnComponentGroup>,
}

impl Pn {
    pub fn single_byte_name(&self) -> Option<&PnComponentGroup> {
        self.single_byte_name.as_ref()
    }

    pub fn ideographic_name(&self) -> Option<&PnComponentGroup> {
        self.ideographic_name.as_ref()
    }

    pub fn phonetic_name(&self) -> Option<&PnComponentGroup> {
        self.phonetic_name.as_ref()
    }

    /// 文字列から人名（PN）の値をパースします。
    ///
    /// この関数は、DICOM規格に準拠したPN型の文字列をパースし、
    /// バックスラッシュ(`\`)で区切られた複数の値を処理します。
    /// 人名は最大3つのコンポーネントグループ（シングルバイト文字、表意文字、表音文字）を持ち、
    /// 各コンポーネントグループはキャレット(`^`)で区切られた最大5つのコンポーネント
    /// （姓、名、ミドルネーム、プレフィックス、サフィックス）を持ちます。
    /// 各コンポーネントグループは最大64文字までです。
    ///
    /// # 引数
    ///
    /// * `str` - パースする文字列。複数の人名はバックスラッシュ(`\`)で区切られます。
    ///
    /// # 戻り値
    ///
    /// * `Ok(Vec<Option<Self>>)` - パースに成功した場合、PN値のベクタを返します。
    ///   空文字列や空白のみの値は`None`として表現されます。
    /// * `Err(PnError)` - パースに失敗した場合、エラーを返します。
    ///   コンポーネントグループが4つ以上、コンポーネントが6つ以上、
    ///   またはコンポーネントグループが64文字を超える場合にエラーとなります。
    ///
    /// # パディング処理
    ///
    /// DICOM規格に従い、文字列が偶数バイト長で末尾が空白の場合、
    /// その空白は自動的に除去されます。
    ///
    /// # コンポーネントグループの構造
    ///
    /// - 1つ目: シングルバイト文字（アルファベット表記）
    /// - 2つ目: 表意文字（漢字など）
    /// - 3つ目: 表音文字（ひらがな、カタカナなど）
    ///
    /// コンポーネントグループは等号(`=`)で区切られます。
    ///
    /// # コンポーネントの構造
    ///
    /// - 1つ目: 姓（Family Name）
    /// - 2つ目: 名（Given Name）
    /// - 3つ目: ミドルネーム（Middle Name）
    /// - 4つ目: プレフィックス（Name Prefix）
    /// - 5つ目: サフィックス（Name Suffix）
    ///
    /// コンポーネントはキャレット(`^`)で区切られます。
    ///
    /// # 例
    ///
    /// ```
    /// use dicom_lib::core::value::values::Pn;
    ///
    /// // 姓と名のみ
    /// let result = Pn::from_string("Doe^John").unwrap();
    /// assert_eq!(result.len(), 1);
    /// let pn = result[0].as_ref().unwrap();
    /// let single_byte = pn.single_byte_name().unwrap();
    /// assert_eq!(single_byte.family_name(), "Doe");
    /// assert_eq!(single_byte.given_name(), "John");
    ///
    /// // 全てのコンポーネントを含む
    /// let result = Pn::from_string("Doe^John^Robert^Dr.^Jr.").unwrap();
    /// let pn = result[0].as_ref().unwrap();
    /// let single_byte = pn.single_byte_name().unwrap();
    /// assert_eq!(single_byte.family_name(), "Doe");
    /// assert_eq!(single_byte.given_name(), "John");
    /// assert_eq!(single_byte.middle_name(), "Robert");
    /// assert_eq!(single_byte.name_prefix(), "Dr.");
    /// assert_eq!(single_byte.name_suffix(), "Jr.");
    ///
    /// // 3つのコンポーネントグループ（シングルバイト、表意文字、表音文字）
    /// let result = Pn::from_string("Yamada^Taro=山田^太郎=やまだ^たろう").unwrap();
    /// let pn = result[0].as_ref().unwrap();
    /// assert_eq!(pn.single_byte_name().unwrap().family_name(), "Yamada");
    /// assert_eq!(pn.ideographic_name().unwrap().family_name(), "山田");
    /// assert_eq!(pn.phonetic_name().unwrap().family_name(), "やまだ");
    ///
    /// // 複数の人名（バックスラッシュ区切り）
    /// let result = Pn::from_string(r"Doe^John\Doe^Jane").unwrap();
    /// assert_eq!(result.len(), 2);
    /// assert_eq!(result[0].as_ref().unwrap().single_byte_name().unwrap().given_name(), "John");
    /// assert_eq!(result[1].as_ref().unwrap().single_byte_name().unwrap().given_name(), "Jane");
    ///
    /// // 空値を含むケース
    /// let result = Pn::from_string(r"\Doe^John").unwrap();
    /// assert_eq!(result.len(), 2);
    /// assert!(result[0].is_none());
    /// assert_eq!(result[1].as_ref().unwrap().single_byte_name().unwrap().family_name(), "Doe");
    ///
    /// // 空のコンポーネント（名が空）
    /// let result = Pn::from_string("Doe^^Robert").unwrap();
    /// let pn = result[0].as_ref().unwrap();
    /// let single_byte = pn.single_byte_name().unwrap();
    /// assert_eq!(single_byte.family_name(), "Doe");
    /// assert_eq!(single_byte.given_name(), "");
    /// assert_eq!(single_byte.middle_name(), "Robert");
    ///
    /// // 表意文字のみ
    /// let result = Pn::from_string("=山田^太郎").unwrap();
    /// let pn = result[0].as_ref().unwrap();
    /// assert!(pn.single_byte_name().is_none());
    /// assert_eq!(pn.ideographic_name().unwrap().family_name(), "山田");
    ///
    /// // プレフィックスとサフィックスを含む
    /// let result = Pn::from_string("Adams^John Robert Quincy^^Rev.^B.A. M.Div.").unwrap();
    /// let pn = result[0].as_ref().unwrap();
    /// let single_byte = pn.single_byte_name().unwrap();
    /// assert_eq!(single_byte.name_prefix(), "Rev.");
    /// assert_eq!(single_byte.name_suffix(), "B.A. M.Div.");
    /// ```
    ///
    /// # エラー
    ///
    /// ```
    /// use dicom_lib::core::value::values::{Pn, pn::PnError};
    ///
    /// // コンポーネントグループが多すぎる（4つ以上）
    /// let result = Pn::from_string("Yamada^Taro=山田^太郎=やまだ^たろう=Extra");
    /// assert!(matches!(result, Err(PnError::TooManyComponentGroups { .. })));
    ///
    /// // コンポーネントが多すぎる（6つ以上）
    /// let result = Pn::from_string("A^B^C^D^E^F");
    /// assert!(matches!(result, Err(PnError::TooManyComponents { .. })));
    ///
    /// // コンポーネントグループが64文字を超える
    /// let input = "Picasso^Pablo^Diego José Francisco de Paula Juan Nepomuceno María de los Remedios Cipriano de la Santísima Trinidad Ruiz y";
    /// let result = Pn::from_string(input);
    /// assert!(matches!(result, Err(PnError::InvalidLength { .. })));
    /// ```
    pub fn from_string(str: &str) -> Result<Vec<Option<Self>>, PnError> {
        let str = if str.len().is_multiple_of(2) && str.ends_with(' ') {
            &str[..str.len() - 1]
        } else {
            str
        };
        let person_names = str.split('\\').collect::<Vec<_>>();
        let mut pn_values = Vec::with_capacity(person_names.len());

        for person_name in person_names {
            pn_values.push(Self::from_string_single(person_name)?);
        }

        Ok(pn_values)
    }

    pub fn from_buf_lossy(
        buf: &[u8],
        char_set: SpecificCharacterSet,
    ) -> Result<Vec<Option<Self>>, PnError> {
        let strings = value::generate_person_name_strings_lossy(buf, char_set);

        let mut values = Vec::with_capacity(strings.len());

        for str in strings {
            values.push(Self::from_string_single(&str)?);
        }

        Ok(values)
    }

    fn from_string_single(str: &str) -> Result<Option<Self>, PnError> {
        let component_groups = str.split('=').collect::<Vec<_>>();
        if component_groups.len() > 3 {
            return Err(PnError::TooManyComponentGroups {
                string: str.to_string(),
                component_group_count: component_groups.len(),
            });
        }

        let mut single_byte_component_group = None;
        let mut ideographic_component_group = None;
        let mut phonetic_component_group = None;

        for (i, component_group) in component_groups.iter().enumerate() {
            let group = PnComponentGroup::from_string(component_group)?;

            match i {
                0 => single_byte_component_group = group,
                1 => ideographic_component_group = group,
                2 => phonetic_component_group = group,
                _ => unreachable!(),
            };
        }

        if single_byte_component_group.is_none()
            && ideographic_component_group.is_none()
            && phonetic_component_group.is_none()
        {
            return Ok(None);
        }

        Ok(Some(Self {
            single_byte_name: single_byte_component_group,
            ideographic_name: ideographic_component_group,
            phonetic_name: phonetic_component_group,
        }))
    }
}

impl Display for Pn {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut component_groups = Vec::with_capacity(3);

        if let Some(ref single_byte) = self.single_byte_name {
            component_groups.push(single_byte.to_string());
        } else {
            component_groups.push(String::new());
        }

        if let Some(ref phonetic) = self.phonetic_name {
            if let Some(ref ideographic) = self.ideographic_name {
                component_groups.push(ideographic.to_string());
            } else {
                component_groups.push(String::new());
            }

            component_groups.push(phonetic.to_string());
        } else if let Some(ref ideographic) = self.ideographic_name {
            component_groups.push(ideographic.to_string());
        }

        write!(f, "{}", component_groups.join("="))
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Error, Debug)]
pub enum PnError {
    #[error(
        "コンポーネントグループ文字列の文字数が64文字を超えています (文字列=\"{string}\", 文字数={char_count})"
    )]
    InvalidLength { string: String, char_count: usize },

    #[error(
        "コンポーネントグループの数が多すぎます (文字列=\"{string}\", コンポーネントグループの数={component_group_count})"
    )]
    TooManyComponentGroups {
        string: String,
        component_group_count: usize,
    },

    #[error(
        "コンポーネントの数が多すぎます (文字列=\"{string}\", コンポーネントの数={component_count})"
    )]
    TooManyComponents {
        string: String,
        component_count: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string() {
        // 正常系: ミドルネーム以外すべてを持つ（Rev. John Robert Quincy Adams, B.A. M.Div.）
        //        https://dicom.nema.org/medical/dicom/2025c/output/chtml/part05/sect_6.2.html#sect_6.2.1.1
        {
            // Arrange
            let input = "Adams^John Robert Quincy^^Rev.^B.A. M.Div.";
            let expected = vec![Some(Pn {
                single_byte_name: Some(PnComponentGroup {
                    family_name: "Adams".to_string(),
                    given_name: "John Robert Quincy".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "Rev.".to_string(),
                    name_suffix: "B.A. M.Div.".to_string(),
                }),
                ideographic_name: None,
                phonetic_name: None,
            })];

            // Act
            let actual = Pn::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 姓、名、サフィックス（Susan Morrison-Jones, Ph.D., Chief Executive Officer）
        //        https://dicom.nema.org/medical/dicom/2025c/output/chtml/part05/sect_6.2.html#sect_6.2.1.1
        {
            // Arrange
            let input = "Morrison-Jones^Susan^^^Ph.D., Chief Executive Officer";
            let expected = vec![Some(Pn {
                single_byte_name: Some(PnComponentGroup {
                    family_name: "Morrison-Jones".to_string(),
                    given_name: "Susan".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "Ph.D., Chief Executive Officer".to_string(),
                }),
                ideographic_name: None,
                phonetic_name: None,
            })];

            // Act
            let actual = Pn::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 姓と名のみ（John Doe）
        //        https://dicom.nema.org/medical/dicom/2025c/output/chtml/part05/sect_6.2.html#sect_6.2.1.1
        {
            // Arrange
            let input = "Doe^John";
            let expected = vec![Some(Pn {
                single_byte_name: Some(PnComponentGroup {
                    family_name: "Doe".to_string(),
                    given_name: "John".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
                ideographic_name: None,
                phonetic_name: None,
            })];

            // Act
            let actual = Pn::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 3つのコンポーネントグループ（シングルバイト、表意文字、表音文字）
        {
            // Arrange
            let input = "Yamada^Taro=山田^太郎=やまだ^たろう";
            let expected = vec![Some(Pn {
                single_byte_name: Some(PnComponentGroup {
                    family_name: "Yamada".to_string(),
                    given_name: "Taro".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
                ideographic_name: Some(PnComponentGroup {
                    family_name: "山田".to_string(),
                    given_name: "太郎".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
                phonetic_name: Some(PnComponentGroup {
                    family_name: "やまだ".to_string(),
                    given_name: "たろう".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
            })];

            // Act
            let actual = Pn::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 全てのコンポーネントを含む
        {
            // Arrange
            let input = "Doe^John^Robert^Dr.^Jr.";
            let expected = vec![Some(Pn {
                single_byte_name: Some(PnComponentGroup {
                    family_name: "Doe".to_string(),
                    given_name: "John".to_string(),
                    middle_name: "Robert".to_string(),
                    name_prefix: "Dr.".to_string(),
                    name_suffix: "Jr.".to_string(),
                }),
                ideographic_name: None,
                phonetic_name: None,
            })];

            // Act
            let actual = Pn::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数の人名（バックスラッシュ区切り）
        {
            // Arrange
            let input = r"Doe^John\Doe^Jane";
            let expected = vec![
                Some(Pn {
                    single_byte_name: Some(PnComponentGroup {
                        family_name: "Doe".to_string(),
                        given_name: "John".to_string(),
                        middle_name: "".to_string(),
                        name_prefix: "".to_string(),
                        name_suffix: "".to_string(),
                    }),
                    ideographic_name: None,
                    phonetic_name: None,
                }),
                Some(Pn {
                    single_byte_name: Some(PnComponentGroup {
                        family_name: "Doe".to_string(),
                        given_name: "Jane".to_string(),
                        middle_name: "".to_string(),
                        name_prefix: "".to_string(),
                        name_suffix: "".to_string(),
                    }),
                    ideographic_name: None,
                    phonetic_name: None,
                }),
            ];

            // Act
            let actual = Pn::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空の値を含むケース
        {
            // Arrange
            let input = r"\Doe^John";
            let expected = vec![
                None,
                Some(Pn {
                    single_byte_name: Some(PnComponentGroup {
                        family_name: "Doe".to_string(),
                        given_name: "John".to_string(),
                        middle_name: "".to_string(),
                        name_prefix: "".to_string(),
                        name_suffix: "".to_string(),
                    }),
                    ideographic_name: None,
                    phonetic_name: None,
                }),
            ];

            // Act
            let actual = Pn::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空のコンポーネント
        {
            // Arrange
            let input = "Doe^^Robert";
            let expected = vec![Some(Pn {
                single_byte_name: Some(PnComponentGroup {
                    family_name: "Doe".to_string(),
                    given_name: "".to_string(),
                    middle_name: "Robert".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
                ideographic_name: None,
                phonetic_name: None,
            })];

            // Act
            let actual = Pn::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空文字列
        {
            // Arrange
            let input = "";
            let expected = vec![None];

            // Act
            let actual = Pn::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 空白のみ
        {
            // Arrange
            let input = "  ";
            let expected = vec![None];

            // Act
            let actual = Pn::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: パディング処理（末尾のスペースを除去）
        {
            // Arrange
            let input = "Yamada^Taro ";
            let expected = vec![Some(Pn {
                single_byte_name: Some(PnComponentGroup {
                    family_name: "Yamada".to_string(),
                    given_name: "Taro".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
                ideographic_name: None,
                phonetic_name: None,
            })];

            // Act
            let actual = Pn::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 複数値のパディング処理
        {
            // Arrange
            let input = r"Doe^John\Doe^Jane ";
            let expected = vec![
                Some(Pn {
                    single_byte_name: Some(PnComponentGroup {
                        family_name: "Doe".to_string(),
                        given_name: "John".to_string(),
                        middle_name: "".to_string(),
                        name_prefix: "".to_string(),
                        name_suffix: "".to_string(),
                    }),
                    ideographic_name: None,
                    phonetic_name: None,
                }),
                Some(Pn {
                    single_byte_name: Some(PnComponentGroup {
                        family_name: "Doe".to_string(),
                        given_name: "Jane".to_string(),
                        middle_name: "".to_string(),
                        name_prefix: "".to_string(),
                        name_suffix: "".to_string(),
                    }),
                    ideographic_name: None,
                    phonetic_name: None,
                }),
            ];

            // Act
            let actual = Pn::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 表意文字のみ
        {
            // Arrange
            let input = "=山田^太郎";
            let expected = vec![Some(Pn {
                single_byte_name: None,
                ideographic_name: Some(PnComponentGroup {
                    family_name: "山田".to_string(),
                    given_name: "太郎".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
                phonetic_name: None,
            })];

            // Act
            let actual = Pn::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 表意文字と表音文字のみ
        {
            // Arrange
            let input = "==やまだ^たろう";
            let expected = vec![Some(Pn {
                single_byte_name: None,
                ideographic_name: None,
                phonetic_name: Some(PnComponentGroup {
                    family_name: "やまだ".to_string(),
                    given_name: "たろう".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
            })];

            // Act
            let actual = Pn::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: コンポーネントグループが多すぎる（4つ以上）
        {
            // Arrange
            let input = "Yamada^Taro=山田^太郎=やまだ^たろう=Extra";

            // Act
            let result = Pn::from_string(input);

            // Assert
            match result.unwrap_err() {
                PnError::TooManyComponentGroups {
                    string,
                    component_group_count,
                } => {
                    assert_eq!(input, string);
                    assert_eq!(4, component_group_count);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: コンポーネントが多すぎる（6つ以上）
        {
            // Arrange
            let input = "A^B^C^D^E^F";

            // Act
            let result = Pn::from_string(input);

            // Assert
            match result.unwrap_err() {
                PnError::TooManyComponents {
                    string,
                    component_count,
                } => {
                    assert_eq!(input, string);
                    assert_eq!(6, component_count);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: コンポーネントグループの文字数が64文字を超える
        {
            // Arrange
            let input = "Picasso^Pablo^Diego José Francisco de Paula Juan Nepomuceno María de los Remedios Cipriano de la Santísima Trinidad Ruiz y";

            // Act
            let result = Pn::from_string(input);

            // Assert
            match result.unwrap_err() {
                PnError::InvalidLength { string, char_count } => {
                    assert_eq!(input, string);
                    assert_eq!(input.chars().count(), char_count);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_from_buf_lossy() {
        // 正常系: ISO 2022 IR 6 & ISO 2022 IR 87（ASCII & JIS X 0208）
        // https://dicom.nema.org/medical/dicom/2025c/output/chtml/part05/sect_H.3.html
        {
            // Arrange
            let buf = [
                0x59, 0x61, 0x6d, 0x61, 0x64, 0x61, 0x5e, 0x54, 0x61, 0x72, 0x6f, 0x75, 0x3d, 0x1b,
                0x24, 0x42, 0x3b, 0x33, 0x45, 0x44, 0x1b, 0x28, 0x42, 0x5e, 0x1b, 0x24, 0x42, 0x42,
                0x40, 0x4f, 0x3a, 0x1b, 0x28, 0x42, 0x3d, 0x1b, 0x24, 0x42, 0x24, 0x64, 0x24, 0x5e,
                0x24, 0x40, 0x1b, 0x28, 0x42, 0x5e, 0x1b, 0x24, 0x42, 0x24, 0x3f, 0x24, 0x6d, 0x24,
                0x26, 0x1b, 0x28, 0x42,
            ];
            let expected = vec![Some(Pn {
                single_byte_name: Some(PnComponentGroup {
                    family_name: "Yamada".to_string(),
                    given_name: "Tarou".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
                ideographic_name: Some(PnComponentGroup {
                    family_name: "山田".to_string(),
                    given_name: "太郎".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
                phonetic_name: Some(PnComponentGroup {
                    family_name: "やまだ".to_string(),
                    given_name: "たろう".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
            })];

            // Act
            let actual =
                Pn::from_buf_lossy(&buf, SpecificCharacterSet::Iso2022Ir6AndIso2022Ir87).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: ISO 2022 IR 13 & ISO 2022 IR 87（JIS X 0201カタカナ & JIS X 0208）
        // https://dicom.nema.org/medical/dicom/2025c/output/chtml/part05/sect_H.3.2.html
        {
            // Arrange
            let buf = [
                0xd4, 0xcf, 0xc0, 0xde, 0x5e, 0xc0, 0xdb, 0xb3, 0x3d, 0x1b, 0x24, 0x42, 0x3b, 0x33,
                0x45, 0x44, 0x1b, 0x28, 0x4a, 0x5e, 0x1b, 0x24, 0x42, 0x42, 0x40, 0x4f, 0x3a, 0x1b,
                0x28, 0x4a, 0x3d, 0x1b, 0x24, 0x42, 0x24, 0x64, 0x24, 0x5e, 0x24, 0x40, 0x1b, 0x28,
                0x4a, 0x5e, 0x1b, 0x24, 0x42, 0x24, 0x3f, 0x24, 0x6d, 0x24, 0x26, 0x1b, 0x28, 0x4a,
            ];
            let expected = vec![Some(Pn {
                single_byte_name: Some(PnComponentGroup {
                    family_name: "ﾔﾏﾀﾞ".to_string(),
                    given_name: "ﾀﾛｳ".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
                ideographic_name: Some(PnComponentGroup {
                    family_name: "山田".to_string(),
                    given_name: "太郎".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
                phonetic_name: Some(PnComponentGroup {
                    family_name: "やまだ".to_string(),
                    given_name: "たろう".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
            })];

            // Act
            let actual =
                Pn::from_buf_lossy(&buf, SpecificCharacterSet::Iso2022Ir13AndIso2022Ir87).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: ISO 2022 IR 6 & ISO 2022 IR 13 & ISO 2022 IR 87
        {
            // Arrange
            let buf = [
                0x59, 0x61, 0x6d, 0x61, 0x64, 0x61, 0x5e, 0x54, 0x61, 0x72, 0x6f, 0x75, 0x3d, 0x1b,
                0x24, 0x42, 0x3b, 0x33, 0x45, 0x44, 0x1b, 0x28, 0x42, 0x5e, 0x1b, 0x24, 0x42, 0x42,
                0x40, 0x4f, 0x3a, 0x1b, 0x28, 0x42, 0x3d, 0x1b, 0x29, 0x49, 0xd4, 0xcf, 0xc0, 0xde,
                0x1b, 0x28, 0x42, 0x5e, 0x1b, 0x29, 0x49, 0xc0, 0xdb, 0xb3, 0x1b, 0x28, 0x42, 0x20,
            ];
            let expected = vec![Some(Pn {
                single_byte_name: Some(PnComponentGroup {
                    family_name: "Yamada".to_string(),
                    given_name: "Tarou".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
                ideographic_name: Some(PnComponentGroup {
                    family_name: "山田".to_string(),
                    given_name: "太郎".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
                phonetic_name: Some(PnComponentGroup {
                    family_name: "ﾔﾏﾀﾞ".to_string(),
                    given_name: "ﾀﾛｳ".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
            })];

            // Act
            let actual = Pn::from_buf_lossy(
                &buf,
                SpecificCharacterSet::Iso2022Ir6AndIso2022Ir13AndIso2022Ir87,
            )
            .unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: ISO IR 192（UTF-8）
        {
            // Arrange
            let buf = [
                0x59, 0x61, 0x6d, 0x61, 0x64, 0x61, 0x5e, 0x54, 0x61, 0x72, 0x6f, 0x75, 0x3d, 0xe5,
                0xb1, 0xb1, 0xe7, 0x94, 0xb0, 0x5e, 0xe5, 0xa4, 0xaa, 0xe9, 0x83, 0x8e, 0x3d, 0xe3,
                0x82, 0x84, 0xe3, 0x81, 0xbe, 0xe3, 0x81, 0xa0, 0x5e, 0xe3, 0x81, 0x9f, 0xe3, 0x82,
                0x8d, 0xe3, 0x81, 0x86,
            ];
            let expected = vec![Some(Pn {
                single_byte_name: Some(PnComponentGroup {
                    family_name: "Yamada".to_string(),
                    given_name: "Tarou".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
                ideographic_name: Some(PnComponentGroup {
                    family_name: "山田".to_string(),
                    given_name: "太郎".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
                phonetic_name: Some(PnComponentGroup {
                    family_name: "やまだ".to_string(),
                    given_name: "たろう".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
            })];

            // Act
            let actual = Pn::from_buf_lossy(&buf, SpecificCharacterSet::IsoIr192).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: JIS X 0208の外字（Shift_JISエンコーディングの「髙」）
        {
            // Arrange
            let buf = [
                0x54, 0x61, 0x6b, 0x61, 0x68, 0x61, 0x73, 0x68, 0x69, 0x5e, 0x44, 0x61, 0x69, 0x73,
                0x75, 0x6b, 0x65, 0x3d, 0x1b, 0x24, 0x42, 0xfb, 0xfc, 0x36, 0x36, 0x1b, 0x28, 0x42,
                0x5e, 0x1b, 0x24, 0x42, 0x42, 0x67, 0x4a, 0x65, 0x1b, 0x28, 0x42, 0x3d, 0x1b, 0x24,
                0x42, 0x24, 0x3f, 0x24, 0x2b, 0x24, 0x4f, 0x24, 0x37, 0x1b, 0x28, 0x42, 0x5e, 0x1b,
                0x24, 0x42, 0x24, 0x40, 0x24, 0x24, 0x24, 0x39, 0x24, 0x31, 0x1b, 0x28, 0x42, 0x20,
            ];
            let expected = vec![Some(Pn {
                single_byte_name: Some(PnComponentGroup {
                    family_name: "Takahashi".to_string(),
                    given_name: "Daisuke".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
                ideographic_name: Some(PnComponentGroup {
                    family_name: "�橋".to_string(),
                    given_name: "大輔".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
                phonetic_name: Some(PnComponentGroup {
                    family_name: "たかはし".to_string(),
                    given_name: "だいすけ".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
            })];

            // Act
            let actual =
                Pn::from_buf_lossy(&buf, SpecificCharacterSet::Iso2022Ir6AndIso2022Ir87).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn test_to_string() {
        // 正常系: 全てのコンポーネントグループを含む
        {
            // Arrange
            let pn = Pn {
                single_byte_name: Some(PnComponentGroup {
                    family_name: "Yamada".to_string(),
                    given_name: "Taro".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
                ideographic_name: Some(PnComponentGroup {
                    family_name: "山田".to_string(),
                    given_name: "太郎".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
                phonetic_name: Some(PnComponentGroup {
                    family_name: "やまだ".to_string(),
                    given_name: "たろう".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
            };
            let expected = "Yamada^Taro=山田^太郎=やまだ^たろう";

            // Act
            let actual = pn.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: シングルバイトのみ
        {
            // Arrange
            let pn = Pn {
                single_byte_name: Some(PnComponentGroup {
                    family_name: "Doe".to_string(),
                    given_name: "John".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
                ideographic_name: None,
                phonetic_name: None,
            };
            let expected = "Doe^John";

            // Act
            let actual = pn.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: シングルバイトと表意文字
        {
            // Arrange
            let pn = Pn {
                single_byte_name: Some(PnComponentGroup {
                    family_name: "Yamada".to_string(),
                    given_name: "Taro".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
                ideographic_name: Some(PnComponentGroup {
                    family_name: "山田".to_string(),
                    given_name: "太郎".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
                phonetic_name: None,
            };
            let expected = "Yamada^Taro=山田^太郎";

            // Act
            let actual = pn.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: シングルバイトが空で表意文字のみ
        {
            // Arrange
            let pn = Pn {
                single_byte_name: None,
                ideographic_name: Some(PnComponentGroup {
                    family_name: "山田".to_string(),
                    given_name: "太郎".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
                phonetic_name: None,
            };
            let expected = "=山田^太郎";

            // Act
            let actual = pn.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: シングルバイトが空で表意文字と表音文字
        {
            // Arrange
            let pn = Pn {
                single_byte_name: None,
                ideographic_name: Some(PnComponentGroup {
                    family_name: "山田".to_string(),
                    given_name: "太郎".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
                phonetic_name: Some(PnComponentGroup {
                    family_name: "やまだ".to_string(),
                    given_name: "たろう".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "".to_string(),
                    name_suffix: "".to_string(),
                }),
            };
            let expected = "=山田^太郎=やまだ^たろう";

            // Act
            let actual = pn.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 全てのコンポーネントを含む
        {
            // Arrange
            let pn = Pn {
                single_byte_name: Some(PnComponentGroup {
                    family_name: "Adams".to_string(),
                    given_name: "John Robert Quincy".to_string(),
                    middle_name: "".to_string(),
                    name_prefix: "Rev.".to_string(),
                    name_suffix: "B.A. M.Div.".to_string(),
                }),
                ideographic_name: None,
                phonetic_name: None,
            };
            let expected = "Adams^John Robert Quincy^^Rev.^B.A. M.Div.";

            // Act
            let actual = pn.to_string();

            // Assert
            assert_eq!(expected, actual);
        }
    }
}
