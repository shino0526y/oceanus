use chrono::NaiveDate;
use dicom_lib::core::value::{
    SpecificCharacterSet,
    values::{Cs, Da, Lo, Pn},
};

pub struct Patient {
    pub id: String,
    pub name_alphabet: String,
    pub name_kanji: String,
    pub name_hiragana: String,
    pub birth_date: Option<NaiveDate>,
    pub sex: Option<Sex>,
}

pub enum Sex {
    M,
    F,
    O,
}

impl Sex {
    pub fn to_smallint(&self) -> i16 {
        match self {
            Sex::M => 0,
            Sex::F => 1,
            Sex::O => 2,
        }
    }
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

        let name_alphabet;
        let name_kanji;
        let name_hiragana;
        {
            if let Some(patients_name) = &patients_name {
                match char_set {
                    SpecificCharacterSet::None => {
                        // 例: "YAMADA^TARO"
                        name_alphabet = if let Some(single_byte) = patients_name.single_byte_name()
                        {
                            single_byte.to_string()
                        } else {
                            String::new()
                        };
                        name_kanji = String::new();
                        name_hiragana = String::new();
                    }
                    SpecificCharacterSet::Iso2022Ir6AndIso2022Ir87
                    | SpecificCharacterSet::IsoIr192 => {
                        // 例: "YAMADA^TARO=山田^太郎=やまだ^たろう"
                        name_alphabet = if let Some(single_byte) = patients_name.single_byte_name()
                        {
                            single_byte.to_string()
                        } else {
                            String::new()
                        };
                        name_kanji = if let Some(ideographic) = patients_name.ideographic_name() {
                            ideographic.to_string()
                        } else {
                            String::new()
                        };
                        name_hiragana = if let Some(phonetic) = patients_name.phonetic_name() {
                            phonetic.to_string()
                        } else {
                            String::new()
                        };
                    }
                    SpecificCharacterSet::Iso2022Ir6AndIso2022Ir13AndIso2022Ir87 => {
                        // 例: "YAMADA^TARO=山田^太郎=ﾔﾏﾀﾞ^ﾀﾛｳ"
                        name_alphabet = if let Some(single_byte) = patients_name.single_byte_name()
                        {
                            single_byte.to_string()
                        } else {
                            String::new()
                        };
                        name_kanji = if let Some(ideographic) = patients_name.ideographic_name() {
                            ideographic.to_string()
                        } else {
                            String::new()
                        };
                        name_hiragana = String::new(); // TODO: 半角カタカナをひらがなに変換する処理の実装
                    }
                    SpecificCharacterSet::Iso2022Ir13AndIso2022Ir87 => {
                        // 例: "ﾔﾏﾀﾞ^ﾀﾛｳ=山田^太郎=やまだ^たろう"
                        name_alphabet = String::new();
                        name_kanji = if let Some(ideographic) = patients_name.ideographic_name() {
                            ideographic.to_string()
                        } else {
                            String::new()
                        };
                        name_hiragana = if let Some(phonetic) = patients_name.phonetic_name() {
                            phonetic.to_string()
                        } else {
                            String::new()
                        };
                    }
                    SpecificCharacterSet::IsoIr13 => {
                        // 例: "ﾔﾏﾀﾞ^ﾀﾛｳ"
                        name_alphabet = String::new();
                        name_kanji = String::new();
                        name_hiragana = String::new(); // TODO: 半角カタカナをひらがなに変換する処理の実装
                    }
                }
            } else {
                name_alphabet = String::new();
                name_kanji = String::new();
                name_hiragana = String::new();
            }
        }

        let birth_date = patients_birth_date.map(|birth_date| *birth_date.date());

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
            name_alphabet,
            name_kanji,
            name_hiragana,
            birth_date,
            sex,
        })
    }
}
