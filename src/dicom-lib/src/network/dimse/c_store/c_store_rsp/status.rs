pub mod code;

/// C-STOREのステータスコード
///
/// # 参考リンク
/// - <https://dicom.nema.org/medical/dicom/2025c/output/chtml/part07/chapter_9.html#sect_9.1.1.1.9>
/// - <https://dicom.nema.org/medical/dicom/2025c/output/chtml/part07/chapter_C.html>
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Status {
    // ----- 成功 (0000) -----
    /// 成功 (0x0000)
    ///
    /// 複合SOPインスタンスが正常に保存されたことを示す。
    Success,

    // ----- 警告 (0001 or 0Bxxx or 0107 or 0116) -----
    /// 警告 (0x0001 or 0x0b000〜0x0bfff or 0x0107 or 0x0116)
    ///
    /// DIMSEサービスユーザーが複合SOPインスタンスを保存できたが、潜在的なエラーを検出したことを示す。
    Warning(code::Warning),

    // ----- 失敗 (Axxx or Cxxx or 01xx (0107と0116を除く) or 02xx) -----
    /// 拒否: リソース不足 (0xa700〜0xa7ff)
    ///
    /// DIMSEサービスユーザーが複合SOPインスタンスの保存をリソース不足のために実行できなかったことを示す。
    ///
    /// なお、C-STOREを利用するStorage Service ClassとNon-Patient Object Storage Service Classがステータスの具体的な値を定めている。
    /// - <https://dicom.nema.org/medical/dicom/2025c/output/chtml/part04/sect_B.2.3.html#table_B.2-1>
    /// - <https://dicom.nema.org/medical/dicom/2025c/output/chtml/part04/sect_GG.4.2.html#table_GG.4-1>
    OutOfResources(code::OutOfResources),

    /// エラー: データセットがSOPクラスと一致しない (0xa900〜0xa9ff)
    ///
    /// データセットがSOPクラスと一致しないため、DIMSEサービスユーザーが複合SOPインスタンスを保存できなかったことを示す。
    ///
    /// なお、C-STOREを利用するStorage Service ClassとNon-Patient Object Storage Service Classがステータスの具体的な値を定めている。
    /// - <https://dicom.nema.org/medical/dicom/2025c/output/chtml/part04/sect_B.2.3.html#table_B.2-1>
    /// - <https://dicom.nema.org/medical/dicom/2025c/output/chtml/part04/sect_GG.4.2.html#table_GG.4-1>
    DataSetDoesNotMatchSopClass(code::DataSetDoesNotMatchSopClass),

    /// エラー: 理解できない (0xc000〜0xcfff)
    ///
    /// DIMSEサービスユーザーが特定のデータ要素を理解できないため、複合SOPインスタンスを保存できなかったことを示す。
    ///
    /// なお、C-STOREを利用するStorage Service ClassとNon-Patient Object Storage Service Classがステータスの具体的な値を定めている。
    /// - <https://dicom.nema.org/medical/dicom/2025c/output/chtml/part04/sect_B.2.3.html#table_B.2-1>
    /// - <https://dicom.nema.org/medical/dicom/2025c/output/chtml/part04/sect_GG.4.2.html#table_GG.4-1>
    CannotUnderstand(code::CannotUnderstand),

    /// 不正なSOPインスタンス (0x0117)
    ///
    /// 指定されたSOPインスタンスUIDがUID構築ルールに違反していることを示す。
    InvalidSopInstance,

    /// 拒否: SOPクラスがサポートされていない (0x0122)
    ///
    /// SOPクラスがサポートされていないため、DIMSEサービスユーザーが複合SOPインスタンスを保存できなかったことを示す。
    SopClassNotSupported,

    /// 拒否: 認証されていない (0x0124)
    ///
    /// ピアDIMSEサービスユーザーが複合SOPインスタンスの保存を許可されていなかったことを示す。
    NotAuthorized,

    /// 重複した呼び出し (0x0210)
    ///
    /// 指定された`Message ID (0000,0110)`が他の通知または操作に割り当てられていることを示す。
    DuplicateInvocation,

    /// 認識されない操作 (0x0211)
    ///
    /// DIMSEサービスユーザー間で合意された操作のいずれでもない操作が指定されたことを示す。
    UnrecognizedOperation,

    /// 誤った引数 (0x0212)
    ///
    /// DIMSEサービスユーザー間のアソシエーションで使用することに合意されていないパラメータが指定されたことを示す。
    MistypedArgument,

    /// その他の失敗
    ///
    /// 上記のどれにも当てはまらない失敗ステータス。将来の拡張のために予約されている。
    OtherFailure(code::OtherFailure),
}

impl TryFrom<u16> for Status {
    type Error = String;

    fn try_from(val: u16) -> Result<Self, Self::Error> {
        match val {
            // 成功 (0000)
            0x0000 => Ok(Status::Success),
            // 警告 (0001 or 0Bxxx or 0107 or 0116)
            v if v == 0x0001 || (0xb000..=0xbfff).contains(&v) || v == 0x0107 || v == 0x0116 => {
                Ok(Status::Warning(code::Warning(v)))
            }
            // 失敗 (Axxx or Cxxx or 01xx (0107と0116を除く) or 02xx)
            v if (0xa000..=0xafff).contains(&v)
                || (0xc000..=0xcfff).contains(&v)
                || ((0x0100..=0x01ff).contains(&v) && v != 0x0107 && v != 0x0116)
                || (0x0200..=0x02ff).contains(&v) =>
            {
                match v {
                    0x0117 => Ok(Status::InvalidSopInstance),
                    0x0122 => Ok(Status::SopClassNotSupported),
                    0x0124 => Ok(Status::NotAuthorized),
                    0x0210 => Ok(Status::DuplicateInvocation),
                    0x0211 => Ok(Status::UnrecognizedOperation),
                    0x0212 => Ok(Status::MistypedArgument),
                    v if (0xa700..=0xa7ff).contains(&v) => {
                        Ok(Status::OutOfResources(code::OutOfResources(v)))
                    }
                    v if (0xa900..=0xa9ff).contains(&v) => Ok(Status::DataSetDoesNotMatchSopClass(
                        code::DataSetDoesNotMatchSopClass(v),
                    )),
                    v if (0xc000..=0xcfff).contains(&v) => {
                        Ok(Status::CannotUnderstand(code::CannotUnderstand(v)))
                    }
                    _ => Ok(Status::OtherFailure(code::OtherFailure(v))),
                }
            }
            _ => Err(format!(
                "C-STOREで定義されていないステータスコードです (コード={val:#06X})"
            )),
        }
    }
}

impl From<Status> for u16 {
    fn from(val: Status) -> Self {
        match val {
            Status::Success => 0x0000,
            Status::Warning(code) => code.0,
            Status::OutOfResources(code) => code.0,
            Status::DataSetDoesNotMatchSopClass(code) => code.0,
            Status::CannotUnderstand(code) => code.0,
            Status::InvalidSopInstance => 0x0117,
            Status::SopClassNotSupported => 0x0122,
            Status::NotAuthorized => 0x0124,
            Status::DuplicateInvocation => 0x0210,
            Status::UnrecognizedOperation => 0x0211,
            Status::MistypedArgument => 0x0212,
            Status::OtherFailure(code) => code.0,
        }
    }
}
