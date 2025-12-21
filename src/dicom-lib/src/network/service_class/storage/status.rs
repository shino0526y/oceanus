pub mod code;

use code::{CannotUnderstand, DataSetMismatch, OutOfResources};

/// Storage Service ClassにおけるC-STOREのステータスコード
///
/// # 参考リンク
/// - <https://dicom.nema.org/medical/dicom/2025c/output/chtml/part04/sect_B.2.3.html#table_B.2-1>
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Status {
    // ----- 成功 -----
    /// 成功 (0x0000)
    ///
    /// SOPインスタンスが正常に保存されたことを示す。
    Success,

    // ----- 警告 -----
    /// データ要素の強制変換 (0xb000)
    ///
    /// SOPインスタンスが保存されたが、一部のデータ要素が強制的に変換されたことを示す。
    CoercionOfDataElements,

    /// 要素が破棄された (0xb006)
    ///
    /// SOPインスタンスが保存されたが、一部の要素が破棄されたことを示す。
    ElementsDiscarded,

    /// データセットがSOPクラスと一致しない (0xb007)
    ///
    /// SOPインスタンスが保存されたが、データセットがSOPクラスと完全に一致しないことを示す。
    DataSetDoesNotMatchSopClassWarning,

    // ----- 失敗 -----
    /// 拒否: リソース不足 (0xa700〜0xa7ff)
    ///
    /// リソース不足のため、SOPインスタンスの保存を実行できなかったことを示す。
    ///
    /// SCPの実装は、0xa700から0xa7ffの範囲内で特定のステータスコードを割り当てることができる。
    OutOfResources(OutOfResources),

    /// エラー: データセットがSOPクラスと一致しない (0xa900〜0xa9ff)
    ///
    /// データセットがSOPクラスと一致しないため、SOPインスタンスを保存できなかったことを示す。
    ///
    /// SCPの実装は、0xa900から0xa9ffの範囲内で特定のステータスコードを割り当てることができる。
    DataSetDoesNotMatchSopClassError(DataSetMismatch),

    /// エラー: 理解できない (0xc000〜0xcfff)
    ///
    /// 特定のデータ要素を理解できないため、SOPインスタンスを保存できなかったことを示す。
    ///
    /// SCPの実装は、0xc000から0xcfffの範囲内で特定のステータスコードを割り当てることができる。
    CannotUnderstand(CannotUnderstand),
}

impl TryFrom<u16> for Status {
    type Error = String;

    fn try_from(val: u16) -> Result<Self, Self::Error> {
        match val {
            // 成功
            0x0000 => Ok(Status::Success),
            // 警告
            0xb000 => Ok(Status::CoercionOfDataElements),
            0xb006 => Ok(Status::ElementsDiscarded),
            0xb007 => Ok(Status::DataSetDoesNotMatchSopClassWarning),
            // 失敗
            v if (0xa700..=0xa7ff).contains(&v) => Ok(Status::OutOfResources(OutOfResources(v))),
            v if (0xa900..=0xa9ff).contains(&v) => {
                Ok(Status::DataSetDoesNotMatchSopClassError(DataSetMismatch(v)))
            }
            v if (0xc000..=0xcfff).contains(&v) => {
                Ok(Status::CannotUnderstand(CannotUnderstand(v)))
            }
            _ => Err(format!(
                "Storage Service Classで定義されていないステータスコードです (コード={val:#06X})"
            )),
        }
    }
}

impl From<Status> for u16 {
    fn from(val: Status) -> Self {
        match val {
            Status::Success => 0x0000,
            Status::CoercionOfDataElements => 0xb000,
            Status::ElementsDiscarded => 0xb006,
            Status::DataSetDoesNotMatchSopClassWarning => 0xb007,
            Status::OutOfResources(code) => code.get(),
            Status::DataSetDoesNotMatchSopClassError(code) => code.get(),
            Status::CannotUnderstand(code) => code.get(),
        }
    }
}

impl From<Status> for crate::network::dimse::c_store::c_store_rsp::Status {
    fn from(val: Status) -> Self {
        use crate::network::dimse::c_store::c_store_rsp::status::code as c_store_code;

        match val {
            Status::Success => crate::network::dimse::c_store::c_store_rsp::Status::Success,
            Status::CoercionOfDataElements => {
                crate::network::dimse::c_store::c_store_rsp::Status::Warning(c_store_code::Warning(
                    0xb000,
                ))
            }
            Status::ElementsDiscarded => {
                crate::network::dimse::c_store::c_store_rsp::Status::Warning(c_store_code::Warning(
                    0xb006,
                ))
            }
            Status::DataSetDoesNotMatchSopClassWarning => {
                crate::network::dimse::c_store::c_store_rsp::Status::Warning(c_store_code::Warning(
                    0xb007,
                ))
            }
            Status::OutOfResources(code) => {
                crate::network::dimse::c_store::c_store_rsp::Status::OutOfResources(
                    c_store_code::OutOfResources(code.0),
                )
            }
            Status::DataSetDoesNotMatchSopClassError(code) => {
                crate::network::dimse::c_store::c_store_rsp::Status::DataSetDoesNotMatchSopClass(
                    c_store_code::DataSetDoesNotMatchSopClass(code.0),
                )
            }
            Status::CannotUnderstand(code) => {
                crate::network::dimse::c_store::c_store_rsp::Status::CannotUnderstand(
                    c_store_code::CannotUnderstand(code.0),
                )
            }
        }
    }
}
