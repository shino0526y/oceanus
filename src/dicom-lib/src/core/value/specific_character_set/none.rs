use super::encoding::ascii;

pub(crate) fn generate_string_lossy(bytes: &[u8]) -> String {
    ascii::generate_string_lossy(bytes, 0, bytes.len())
}

pub(crate) fn generate_person_name_strings_lossy(bytes: &[u8]) -> Vec<String> {
    generate_string_lossy(bytes)
        .trim_end_matches(' ')
        .split('\\')
        .map(|s| s.to_string())
        .collect()
}
