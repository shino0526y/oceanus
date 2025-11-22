use crate::core::value::values::pn::PnError;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub struct PnComponentGroup {
    pub(super) family_name: String,
    pub(super) given_name: String,
    pub(super) middle_name: String,
    pub(super) name_prefix: String,
    pub(super) name_suffix: String,
}

impl PnComponentGroup {
    const MAX_CHAR_COUNT: usize = 64;

    pub fn family_name(&self) -> &str {
        &self.family_name
    }

    pub fn given_name(&self) -> &str {
        &self.given_name
    }

    pub fn middle_name(&self) -> &str {
        &self.middle_name
    }

    pub fn name_prefix(&self) -> &str {
        &self.name_prefix
    }

    pub fn name_suffix(&self) -> &str {
        &self.name_suffix
    }

    pub fn from_string(str: &str) -> Result<Option<Self>, PnError> {
        let char_count = str.chars().count();
        if char_count > Self::MAX_CHAR_COUNT {
            return Err(PnError::InvalidLength {
                string: str.to_string(),
                char_count,
            });
        }

        let trimmed = str.trim_matches(' ');
        if trimmed.is_empty() {
            return Ok(None);
        }

        let components = trimmed.split('^').collect::<Vec<_>>();
        if components.len() > 5 {
            return Err(PnError::TooManyComponents {
                string: trimmed.to_string(),
                component_count: components.len(),
            });
        }

        let mut family_name = "";
        let mut given_name = "";
        let mut middle_name = "";
        let mut name_prefix = "";
        let mut name_suffix = "";

        for (i, component) in components.iter().enumerate() {
            match i {
                0 => family_name = component,
                1 => given_name = component,
                2 => middle_name = component,
                3 => name_prefix = component,
                4 => name_suffix = component,
                _ => unreachable!(),
            }
        }

        if family_name.is_empty()
            && given_name.is_empty()
            && middle_name.is_empty()
            && name_prefix.is_empty()
            && name_suffix.is_empty()
        {
            return Ok(None);
        }

        Ok(Some(Self {
            family_name: family_name.to_string(),
            given_name: given_name.to_string(),
            middle_name: middle_name.to_string(),
            name_prefix: name_prefix.to_string(),
            name_suffix: name_suffix.to_string(),
        }))
    }
}

impl Display for PnComponentGroup {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut components = vec![
            self.family_name.as_str(),
            self.given_name.as_str(),
            self.middle_name.as_str(),
            self.name_prefix.as_str(),
            self.name_suffix.as_str(),
        ];

        while let Some(&last) = components.last() {
            if last.is_empty() {
                components.pop();
            } else {
                break;
            }
        }

        write!(f, "{}", components.join("^"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_string() {
        // 正常系: ミドルネーム以外すべてを持つ（Rev. John Robert Quincy Adams, B.A. M.Div.）
        //        https://dicom.nema.org/medical/dicom/2025c/output/chtml/part05/sect_6.2.html#sect_6.2.1.1
        {
            // Arrange
            let component_group = PnComponentGroup {
                family_name: "Adams".to_string(),
                given_name: "John Robert Quincy".to_string(),
                middle_name: "".to_string(),
                name_prefix: "Rev.".to_string(),
                name_suffix: "B.A. M.Div.".to_string(),
            };
            let expected = "Adams^John Robert Quincy^^Rev.^B.A. M.Div.";

            // Act
            let actual = component_group.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 姓、名、サフィックス（Susan Morrison-Jones, Ph.D., Chief Executive Officer）
        //        https://dicom.nema.org/medical/dicom/2025c/output/chtml/part05/sect_6.2.html#sect_6.2.1.1
        {
            // Arrange
            let component_group = PnComponentGroup {
                family_name: "Morrison-Jones".to_string(),
                given_name: "Susan".to_string(),
                middle_name: "".to_string(),
                name_prefix: "".to_string(),
                name_suffix: "Ph.D., Chief Executive Officer".to_string(),
            };
            let expected = "Morrison-Jones^Susan^^^Ph.D., Chief Executive Officer";

            // Act
            let actual = component_group.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 姓と名のみ（John Doe）
        //        https://dicom.nema.org/medical/dicom/2025c/output/chtml/part05/sect_6.2.html#sect_6.2.1.1
        {
            // Arrange
            let component_group = PnComponentGroup {
                family_name: "Doe".to_string(),
                given_name: "John".to_string(),
                middle_name: "".to_string(),
                name_prefix: "".to_string(),
                name_suffix: "".to_string(),
            };
            let expected = "Doe^John";

            // Act
            let actual = component_group.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: ペット名（Smith^Fluffy）
        //        https://dicom.nema.org/medical/dicom/2025c/output/chtml/part05/sect_6.2.html#sect_6.2.1.1
        {
            // Arrange
            let component_group = PnComponentGroup {
                family_name: "Smith".to_string(),
                given_name: "Fluffy".to_string(),
                middle_name: "".to_string(),
                name_prefix: "".to_string(),
                name_suffix: "".to_string(),
            };
            let expected = "Smith^Fluffy";

            // Act
            let actual = component_group.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 競走馬名（ABC Farms^Running on Water）
        //        https://dicom.nema.org/medical/dicom/2025c/output/chtml/part05/sect_6.2.html#sect_6.2.1.1
        {
            // Arrange
            let component_group = PnComponentGroup {
                family_name: "ABC Farms".to_string(),
                given_name: "Running on Water".to_string(),
                middle_name: "".to_string(),
                name_prefix: "".to_string(),
                name_suffix: "".to_string(),
            };
            let expected = "ABC Farms^Running on Water";

            // Act
            let actual = component_group.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 姓のみ
        {
            // Arrange
            let component_group = PnComponentGroup {
                family_name: "Smith".to_string(),
                given_name: "".to_string(),
                middle_name: "".to_string(),
                name_prefix: "".to_string(),
                name_suffix: "".to_string(),
            };
            let expected = "Smith";

            // Act
            let actual = component_group.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: 姓と名とミドルネーム
        {
            // Arrange
            let component_group = PnComponentGroup {
                family_name: "Doe".to_string(),
                given_name: "John".to_string(),
                middle_name: "Robert".to_string(),
                name_prefix: "".to_string(),
                name_suffix: "".to_string(),
            };
            let expected = "Doe^John^Robert";

            // Act
            let actual = component_group.to_string();

            // Assert
            assert_eq!(expected, actual);
        }

        // 正常系: ミドルネームが空で、プレフィックスが存在する場合
        {
            // Arrange
            let component_group = PnComponentGroup {
                family_name: "Adams".to_string(),
                given_name: "John".to_string(),
                middle_name: "".to_string(),
                name_prefix: "Dr.".to_string(),
                name_suffix: "".to_string(),
            };
            let expected = "Adams^John^^Dr.";

            // Act
            let actual = component_group.to_string();

            // Assert
            assert_eq!(expected, actual);
        }
    }
}
