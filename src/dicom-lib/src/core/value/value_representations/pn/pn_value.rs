use thiserror::Error;

use super::pn_component_group::PnComponentGroup;
use crate::core::value::{self, SpecificCharacterSet};
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub struct PnValue {
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

impl PnValue {
    pub fn single_byte_name(&self) -> Option<&PnComponentGroup> {
        self.single_byte_name.as_ref()
    }

    pub fn ideographic_name(&self) -> Option<&PnComponentGroup> {
        self.ideographic_name.as_ref()
    }

    pub fn phonetic_name(&self) -> Option<&PnComponentGroup> {
        self.phonetic_name.as_ref()
    }

    pub fn from_bytes_lossy(
        bytes: &[u8],
        char_set: SpecificCharacterSet,
    ) -> Result<Self, PnValueError> {
        let str = value::generate_person_name_strings_lossy(bytes, char_set).join("\\");
        Self::from_string(&str)
    }

    pub fn from_string(str: &str) -> Result<Self, PnValueError> {
        let component_groups = str.split('=').collect::<Vec<_>>();
        if component_groups.len() > 3 {
            return Err(PnValueError::TooManyComponentGroups {
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
            return Err(PnValueError::Empty);
        }

        Ok(Self {
            single_byte_name: single_byte_component_group,
            ideographic_name: ideographic_component_group,
            phonetic_name: phonetic_component_group,
        })
    }
}

impl Display for PnValue {
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
pub enum PnValueError {
    #[error("空値は許容されません")]
    Empty,

    #[error(
        "コンポーネントグループ文字列の文字数が64文字を超えています (文字列=\"{string}\", 文字数={char_count})"
    )]
    InvalidLength { string: String, char_count: usize },

    #[error(
        "コンポーネントの数が多すぎます (文字列=\"{string}\", コンポーネントの数={component_count})"
    )]
    TooManyComponents {
        string: String,
        component_count: usize,
    },

    #[error(
        "コンポーネントグループの数が多すぎます (文字列=\"{string}\", コンポーネントグループの数={component_group_count})"
    )]
    TooManyComponentGroups {
        string: String,
        component_group_count: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_string() {
        // 正常系: ミドルネーム以外すべてを持つ(Rev. John Robert Quincy Adams, B.A. M.Div.)
        {
            // Arrange
            let input = "Adams^John Robert Quincy^^Rev.^B.A. M.Div.";
            let expected = PnValue {
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

            // Act
            let actual = PnValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 3つのコンポーネントグループ(シングルバイト、表意文字、表音文字)
        {
            // Arrange
            let input = "Yamada^Taro=山田^太郎=やまだ^たろう";
            let expected = PnValue {
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

            // Act
            let actual = PnValue::from_string(input).unwrap();

            // Assert
            assert_eq!(expected, actual);
        }

        // 準正常系: 空文字列(Empty)
        {
            // Arrange
            let input = "";

            // Act
            let result = PnValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                PnValueError::Empty => {}
                _ => panic!("期待されたエラーではありません"),
            }
        }

        // 準正常系: コンポーネントグループが4つ(TooManyComponentGroups)
        {
            // Arrange
            let input = "Doe^John=山田^太郎=やまだ^たろう=Extra";

            // Act
            let result = PnValue::from_string(input);

            // Assert
            match result.unwrap_err() {
                PnValueError::TooManyComponentGroups {
                    string,
                    component_group_count,
                } => {
                    assert_eq!(string, input);
                    assert_eq!(component_group_count, 4);
                }
                _ => panic!("期待されたエラーではありません"),
            }
        }
    }

    #[test]
    fn test_to_string() {
        // 正常系: 全てのコンポーネントグループを含む
        {
            // Arrange
            let pn = PnValue {
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
    }
}
