/// C-STORE-RSPのステータスコード
///
/// ## 参考文献
/// - <https://dicom.nema.org/medical/dicom/2025c/output/chtml/part04/sect_b.2.3.html>
/// - <https://dicom.nema.org/medical/dicom/2025c/output/chtml/part07/chapter_9.html#sect_9.1.1.1.9>
/// - <https://dicom.nema.org/medical/dicom/2025c/output/chtml/part07/chapter_C.html>
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Status {
    /// 成功 (0x0000)
    ///
    /// 合成SOPインスタンスが正常に保存されたことを示します。
    Success,

    /// 警告（Service Class に固有; 0xb000..=0xbfff）
    ///
    /// 合成SOPインスタンスの保存は行われましたが、推定されるエラーが検出されたことを示します。
    ///
    /// 例: 0xb000（データ要素の強制変換）、0xb006（要素破棄）、0xb007（データセットとSOPクラスの不一致）など。
    Warning(u16),

    /// 拒否: リソース不足（Service Class に固有; 0xa700..=0xa7ff）
    ///
    /// 実行側 DIMSE サービスユーザがリソース不足のため、合成SOPインスタンスを保存できなかったことを示します。
    RefusedOutOfResources(u16),

    /// エラー: データセットがSOPクラスと一致しない（Service Class に固有; 0xa900..=0xa9ff）
    ///
    /// データセットがSOPクラスに適合しないため、合成SOPインスタンスを保存できなかったことを示します。
    ErrorDataSetMismatch(u16),

    /// エラー: 解釈不能（Service Class に固有; 0xc000..=0xcfff）
    ///
    /// 一部のデータ要素を理解できないため、合成SOPインスタンスを保存できなかったことを示します。
    ErrorCannotUnderstand(u16),

    /// 拒否: SOPクラス非対応（0x0122）
    ///
    /// 実行側 DIMSE サービスユーザが、SOPクラスがサポートされていないため保存できなかったことを示します。
    RefusedSopClassNotSupported,

    /// 拒否: 非認可（0x0124）
    ///
    /// ピアの DIMSE サービスユーザに、合成SOPインスタンスの保存を行う権限がなかったことを示します。
    RefusedNotAuthorized,

    /// 無効なSOPインスタンス（0x0117）
    ///
    /// 指定された SOP Instance UID が UID 構成規則の違反を含意することを示します。
    InvalidSopInstance,

    /// 重複呼び出し（0x0210）
    ///
    /// 指定された Message ID (0000,0110) が他の通知または操作に割り当て済みであることを示します。
    DuplicateInvocation,

    /// 未認識の操作（0x0211）
    ///
    /// 操作が DIMSE サービスユーザ間で合意されているものではないことを示します。
    UnrecognizedOperation,

    /// 誤った引数（0x0212）
    ///
    /// 供給されたパラメータの一つが、DIMSEサービスユーザ間のアソシエーションで使用合意されていないことを示します。
    MistypedArgument,
}

impl TryFrom<u16> for Status {
    type Error = &'static str;

    fn try_from(val: u16) -> Result<Self, Self::Error> {
        match val {
            0x0000 => Ok(Status::Success),
            0x0122 => Ok(Status::RefusedSopClassNotSupported),
            0x0124 => Ok(Status::RefusedNotAuthorized),
            0x0117 => Ok(Status::InvalidSopInstance),
            0x0210 => Ok(Status::DuplicateInvocation),
            0x0211 => Ok(Status::UnrecognizedOperation),
            0x0212 => Ok(Status::MistypedArgument),
            c if (0xb000..=0xbfff).contains(&c) => Ok(Status::Warning(c)),
            c if (0xa700..=0xa7ff).contains(&c) => Ok(Status::RefusedOutOfResources(c)),
            c if (0xa900..=0xa9ff).contains(&c) => Ok(Status::ErrorDataSetMismatch(c)),
            c if (0xc000..=0xcfff).contains(&c) => Ok(Status::ErrorCannotUnderstand(c)),
            _ => Err("不正なステータスコードです"),
        }
    }
}

impl From<Status> for u16 {
    fn from(val: Status) -> Self {
        match val {
            Status::Success => 0x0000,
            Status::RefusedSopClassNotSupported => 0x0122,
            Status::RefusedNotAuthorized => 0x0124,
            Status::InvalidSopInstance => 0x0117,
            Status::DuplicateInvocation => 0x0210,
            Status::UnrecognizedOperation => 0x0211,
            Status::MistypedArgument => 0x0212,
            Status::Warning(v) => v,
            Status::RefusedOutOfResources(v) => v,
            Status::ErrorDataSetMismatch(v) => v,
            Status::ErrorCannotUnderstand(v) => v,
        }
    }
}
