pub(crate) fn generate_string_lossy(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

pub(crate) fn generate_person_name_strings_lossy(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes)
        .trim_end_matches(' ')
        .split('\\')
        .map(|s| s.to_string())
        .collect()
}
