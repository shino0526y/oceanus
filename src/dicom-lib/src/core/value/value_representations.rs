// https://dicom.nema.org/medical/dicom/2025c/output/chtml/part05/sect_6.2.html

pub mod ae;
pub mod cs;
pub mod da;
pub mod fd;
pub mod is;
pub mod lo;
pub mod ob;
pub mod pn;
pub mod sh;
pub mod tm;
pub mod ui;
pub mod ul;
pub mod ur;

pub use ae::Ae;
pub use cs::Cs;
pub use da::Da;
pub use fd::Fd;
pub use is::Is;
pub use lo::Lo;
pub use ob::Ob;
pub use pn::Pn;
pub use sh::Sh;
pub use tm::Tm;
pub use ui::Ui;
pub use ul::Ul;
pub use ur::Ur;

use std::str::Utf8Error;

#[derive(thiserror::Error, Debug)]
pub enum SingleStringValueError {
    #[error("値の変換に失敗しました (文字列=\"{string}\"): {error}")]
    FailedToParse {
        string: String,
        error: Box<dyn std::error::Error>,
    },

    #[error("バイト列をUTF-8として解釈できません: {0}")]
    InvalidUtf8(#[from] Utf8Error),
}

#[derive(thiserror::Error, Debug)]
pub enum MultiStringValueError {
    #[error("値{}の変換に失敗しました (文字列=\"{string}\"): {error}", index + 1)]
    FailedToParse {
        string: String,
        index: usize,
        error: Box<dyn std::error::Error>,
    },

    #[error("バイト列をUTF-8として解釈できません: {0}")]
    InvalidUtf8(#[from] Utf8Error),
}

#[derive(thiserror::Error, Debug)]
pub enum MultiNumberValueError {
    #[error("バイト列の長さが{bytes_per_value}の倍数ではありません (バイト数={byte_length})")]
    InvalidLength {
        bytes_per_value: usize,
        byte_length: usize,
    },
}
