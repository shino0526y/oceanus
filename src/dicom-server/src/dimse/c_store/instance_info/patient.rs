use chrono::NaiveDate;
use dicom_lib::core::value::{
    SpecificCharacterSet,
    values::{Cs, Da, Lo, Pn},
};

pub struct Patient {
    pub id: String,
    pub family_name_alphabet: String,
    pub given_name_alphabet: String,
    pub family_name_kanji: String,
    pub given_name_kanji: String,
    pub family_name_hiragana: String,
    pub given_name_hiragana: String,
    pub birth_date: Option<NaiveDate>,
    pub sex: Option<Sex>,
}

pub enum Sex {
    M,
    F,
    O,
}

impl Patient {
    pub fn new(
        char_set: SpecificCharacterSet,
        patients_name: Option<Pn>,
        patient_id: Option<Lo>,
        patients_birth_date: Option<Da>,
        patients_sex: Option<Cs>,
    ) -> Result<Self, String> {
        let id = if let Some(patient_id) = &patient_id {
            patient_id.string()
        } else {
            ""
        }
        .to_string();

        let mut family_name_alphabet = "";
        let mut given_name_alphabet = "";
        let mut family_name_kanji = "";
        let mut given_name_kanji = "";
        let mut family_name_hiragana = "";
        let mut given_name_hiragana = "";
        {
            let mut family_name_single_byte = "";
            let mut given_name_single_byte = "";
            let mut family_name_ideographic = "";
            let mut given_name_ideographic = "";
            let mut family_name_phonetic = "";
            let mut given_name_phonetic = "";
            if let Some(patients_name) = &patients_name {
                if let Some(single_byte) = patients_name.single_byte_name() {
                    family_name_single_byte = single_byte.family_name();
                    given_name_single_byte = single_byte.given_name();
                }
                if let Some(ideographic) = patients_name.ideographic_name() {
                    family_name_ideographic = ideographic.family_name();
                    given_name_ideographic = ideographic.given_name();
                }
                if let Some(phonetic) = patients_name.phonetic_name() {
                    family_name_phonetic = phonetic.family_name();
                    given_name_phonetic = phonetic.given_name();
                }
            }

            match char_set {
                SpecificCharacterSet::None => {
                    // 例: "YAMADA^TARO"
                    family_name_alphabet = family_name_single_byte;
                    given_name_alphabet = given_name_single_byte;
                }
                SpecificCharacterSet::Iso2022Ir6AndIso2022Ir87 | SpecificCharacterSet::IsoIr192 => {
                    // 例: "YAMADA^TARO=山田^太郎=やまだ^たろう"
                    family_name_alphabet = family_name_single_byte;
                    given_name_alphabet = given_name_single_byte;
                    family_name_kanji = family_name_ideographic;
                    given_name_kanji = given_name_ideographic;
                    family_name_hiragana = family_name_phonetic;
                    given_name_hiragana = given_name_phonetic;
                }
                SpecificCharacterSet::Iso2022Ir6AndIso2022Ir13AndIso2022Ir87 => {
                    // 例: "YAMADA^TARO=山田^太郎=ﾔﾏﾀﾞ^ﾀﾛｳ"

                    // TODO: 半角カタカナをひらがなに変換する処理の実装
                    family_name_alphabet = family_name_single_byte;
                    given_name_alphabet = given_name_single_byte;
                    family_name_kanji = family_name_ideographic;
                    given_name_kanji = given_name_ideographic;
                }
                SpecificCharacterSet::Iso2022Ir13AndIso2022Ir87 => {
                    // 例: "ﾔﾏﾀﾞ^ﾀﾛｳ=山田^太郎=やまだ^たろう"

                    // シングルバイト文字コンポーネントグループは半角カタカナであるため、name_alphabetにはセットしない
                    family_name_kanji = family_name_ideographic;
                    given_name_kanji = given_name_ideographic;
                    family_name_hiragana = family_name_phonetic;
                    given_name_hiragana = given_name_phonetic;
                }
                SpecificCharacterSet::IsoIr13 => {
                    // 例: "ﾔﾏﾀﾞ^ﾀﾛｳ"

                    // TODO: 半角カタカナをひらがなに変換する処理の実装
                }
            }
        }

        let birth_date = if let Some(birth_date) = patients_birth_date {
            Some(*birth_date.date())
        } else {
            None
        };

        let sex = if let Some(patients_sex) = patients_sex {
            match patients_sex.code() {
                "M" => Some(Sex::M),
                "F" => Some(Sex::F),
                "O" => Some(Sex::O),
                invalid => Err(format!("Patient's Sexの値が不正です: {}", invalid))?,
            }
        } else {
            None
        };

        Ok(Patient {
            id,
            family_name_alphabet: family_name_alphabet.to_string(),
            given_name_alphabet: given_name_alphabet.to_string(),
            family_name_kanji: family_name_kanji.to_string(),
            given_name_kanji: given_name_kanji.to_string(),
            family_name_hiragana: family_name_hiragana.to_string(),
            given_name_hiragana: given_name_hiragana.to_string(),
            birth_date,
            sex,
        })
    }
}
