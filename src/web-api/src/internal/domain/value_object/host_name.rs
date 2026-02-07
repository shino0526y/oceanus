use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HostName(String);

impl HostName {
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();

        if value.is_empty() {
            return Err("ホスト名は空にできません".to_string());
        }

        let is_valid_hostname = Self::is_valid_hostname(&value);
        let is_valid_ipv4 = Self::is_valid_ipv4(&value);
        if !is_valid_hostname && !is_valid_ipv4 {
            return Err(format!(
                "ホスト名またはIPv4アドレスのどちらの形式も満たしていません (入力文字列=\"{value}\")"
            )
            .to_string());
        }

        Ok(Self(value))
    }

    pub fn value(&self) -> &str {
        &self.0
    }

    /// ホスト名がRFC 1123形式に準拠しているかチェックする
    fn is_valid_hostname(hostname: &str) -> bool {
        // ホスト名全体の長さチェック (1〜253文字)
        if hostname.is_empty() || hostname.len() > 253 {
            return false;
        }

        // 構造とラベルの長さチェック (各ラベル1〜63文字)
        HOSTNAME_REGEX
            .get_or_init(|| {
                Regex::new(
                    r"(?i)^(([a-z0-9]|[a-z0-9][a-z0-9\-]{0,61}[a-z0-9])\.)*([a-z0-9]|[a-z0-9][a-z0-9\-]{0,61}[a-z0-9])$"
                ).unwrap()
            })
            .is_match(hostname)
    }

    /// IPアドレスがIPv4形式であるかチェックする
    fn is_valid_ipv4(ip: &str) -> bool {
        IPV4_REGEX
            .get_or_init(|| {
                Regex::new(
                    r"^((25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.){3}(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])$"
                ).unwrap()
            })
            .is_match(ip)
    }
}

static HOSTNAME_REGEX: OnceLock<Regex> = OnceLock::new();
static IPV4_REGEX: OnceLock<Regex> = OnceLock::new();
