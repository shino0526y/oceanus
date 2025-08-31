pub mod implementation_class_uid;
pub mod implementation_version_name;
pub mod maximum_length;

pub use implementation_class_uid::ImplementationClassUid;
pub use implementation_version_name::ImplementationVersionName;
pub use maximum_length::MaximumLength;

use crate::network::upper_layer_protocol::pdu::{
    ItemType, PduReadError, a_associate::INVALID_ITEM_LENGTH_ERROR_MESSAGE,
};

pub(crate) const ITEM_TYPE: u8 = 0x50;

// Maximum Length Application PDU NotificationとImplementation Identification Notificationに対応しているが、それ以外には対応していない。
// 対応に迫られたら実装する。
pub struct UserInformation {
    length: u16,
    maximum_length: Option<MaximumLength>,
    implementation_class_uid: ImplementationClassUid,
    implementation_version_name: Option<ImplementationVersionName>,
}

impl UserInformation {
    pub fn size(&self) -> usize {
        4 + self.length as usize
    }

    pub fn length(&self) -> u16 {
        self.length
    }

    pub fn maximum_length(&self) -> Option<&MaximumLength> {
        self.maximum_length.as_ref()
    }

    pub fn implementation_class_uid(&self) -> &ImplementationClassUid {
        &self.implementation_class_uid
    }

    pub fn implementation_version_name(&self) -> Option<&ImplementationVersionName> {
        self.implementation_version_name.as_ref()
    }

    pub fn new(
        maximum_length: Option<MaximumLength>,
        implementation_class_uid: ImplementationClassUid,
        implementation_version_name: Option<ImplementationVersionName>,
    ) -> Self {
        let length = (maximum_length
            .as_ref()
            .map_or(0, |maximum_length| maximum_length.size())
            + implementation_class_uid.size()
            + implementation_version_name
                .as_ref()
                .map_or(0, |implementation_version_name| {
                    implementation_version_name.size()
                })) as u16;

        Self {
            length,
            maximum_length,
            implementation_class_uid,
            implementation_version_name,
        }
    }

    pub async fn read_from_stream(
        buf_reader: &mut tokio::io::BufReader<impl tokio::io::AsyncRead + Unpin>,
        length: u16,
    ) -> Result<Self, PduReadError> {
        use tokio::io::AsyncReadExt;

        let mut offset = 0;
        let mut maximum_length = Option::None;
        let mut implementation_class_uid = Option::None;
        let mut implementation_version_name = Option::None;
        while offset < length as usize {
            if offset + 4 > length as usize {
                // オフセット + Sub-Itemヘッダ（Item-type, Reserved, Item-length）の長さ が全体の長さを超えている場合
                return Err(PduReadError::InvalidPduParameterValue {
                    message: INVALID_ITEM_LENGTH_ERROR_MESSAGE.to_string(),
                });
            }

            let sub_item_type = ItemType::read_from_stream(buf_reader).await?;
            offset += 1;
            buf_reader.read_u8().await?; // Reserved
            offset += 1;
            let sub_item_length = buf_reader.read_u16().await?;
            offset += 2;

            match sub_item_type {
                ItemType::MaximumLengthSubItem => {
                    maximum_length = {
                        let maximum_length = MaximumLength::read_from_stream(
                            buf_reader,
                            sub_item_length,
                        )
                        .await
                        .map_err(|e| match e {
                            PduReadError::IoError(_) => e,
                            PduReadError::InvalidPduParameterValue { message } => {
                                PduReadError::InvalidPduParameterValue {
                                    message: format!(
                                        "Maximum Length Sub-Itemのパースに失敗しました: {message}"
                                    ),
                                }
                            }
                            _ => panic!(),
                        })?;
                        offset += maximum_length.length() as usize;

                        Some(maximum_length)
                    }
                }
                ItemType::ImplementationClassUidSubItem => {
                    implementation_class_uid = {
                        let implementation_class_uid = ImplementationClassUid::read_from_stream(
                            buf_reader,
                            sub_item_length,
                        )
                        .await
                        .map_err(|e| match e {
                            PduReadError::IoError(_) => e,
                            PduReadError::InvalidPduParameterValue { message } => {
                                PduReadError::InvalidPduParameterValue {
                                    message: format!(
                                        "Implementation Class UID Sub-Itemのパースに失敗しました: {message}"
                                    ),
                                }
                            }
                            _ => panic!(),
                        })?;
                        offset += implementation_class_uid.length() as usize;

                        Some(implementation_class_uid)
                    }
                }
                ItemType::ImplementationVersionNameSubItem => {
                    implementation_version_name = {
                        let implementation_version_name =
                            ImplementationVersionName::read_from_stream(buf_reader, sub_item_length).await.map_err(|e| match e {
                                PduReadError::IoError(_) => e,
                                PduReadError::InvalidPduParameterValue { message } => PduReadError::InvalidPduParameterValue{
                                        message: format!(
                                            "Implementation Version Name Sub-Itemのパースに失敗しました: {message}"
                                        ),
                                },
                                _ => panic!(),
                            })?;
                        offset += implementation_version_name.length() as usize;

                        Some(implementation_version_name)
                    }
                }
                ItemType::AsynchronousOperationsWindowSubItem
                | ItemType::ScpScuRoleSelectionSubItem
                | ItemType::SopClassExtendedNegotiationSubItem
                | ItemType::SopClassCommonExtendedNegotiationSubItem
                // FIXME: A-ASSOCIATE-RQのUser Identity Sub-ItemとA-ASSOCIATE-ACのUser Identity Sub-Itemは別物なので、別々に扱う
                | ItemType::UserIdentitySubItemInAAssociateRq
                | ItemType::UserIdentitySubItemInAAssociateAc => {
                    // TODO: 対応しないサブアイテムの処理。暫定対応として、バイト列をそのまま出力している。
                    let mut buf = vec![0; sub_item_length as usize];
                    buf_reader.read_exact(&mut buf).await?;
                    offset += buf.len();

                    tracing::debug!(
                        "未対応のSub-Itemが存在します (Item-type=0x{:02X} バイト列=[{}])",
                        sub_item_type as u8,
                        buf.iter()
                            .map(|b| format!("0x{b:02X}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
                _ => {
                    return Err(PduReadError::UnexpectedPduParameter(sub_item_type));
                }
            }
        }

        if offset != length as usize {
            return Err(PduReadError::InvalidPduParameterValue {
                message: format!(
                    "Item-lengthと実際の読み取りバイト数が一致しません (Item-length={length} 読み取りバイト数={offset})"
                ),
            });
        }

        let implementation_class_uid =
            implementation_class_uid.ok_or_else(|| PduReadError::InvalidPduParameterValue {
                message: "Implementation Class UID Sub-Itemが存在しません".to_string(),
            })?;

        Ok(Self {
            length,
            maximum_length,
            implementation_class_uid,
            implementation_version_name,
        })
    }
}

impl From<UserInformation> for Vec<u8> {
    fn from(val: UserInformation) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(val.size());

        bytes.push(ITEM_TYPE);
        bytes.push(0); // Reserved
        bytes.extend(val.length.to_be_bytes());

        if let Some(maximum_length) = val.maximum_length {
            bytes.append(&mut maximum_length.into());
        }

        bytes.append(&mut val.implementation_class_uid.into());

        if let Some(implementation_version_name) = val.implementation_version_name {
            bytes.append(&mut implementation_version_name.into());
        }

        bytes
    }
}
