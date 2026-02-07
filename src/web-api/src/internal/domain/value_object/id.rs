#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(String);

impl Id {
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();

        if value.is_empty() {
            return Err("IDは空にできません".to_string());
        }

        if value.contains(char::is_whitespace) {
            return Err("IDに空白を含めることはできません".to_string());
        }

        Ok(Self(value))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}
