pub mod sub_items;

use crate::dicom::network::pdu::a_associate::items::{INVALID_ITEM_TYPE_ERROR_MESSAGE, Item};

pub(crate) const ITEM_TYPE: u8 = 0x50;

// Maximum Length Application PDU NotificationとImplementation Identification Notificationに対応しているが、それ以外には対応していない。
// 対応に迫られたら実装する。
pub struct UserInformation {
    length: u16,
    maximum_length: Option<sub_items::MaximumLength>,
    implementation_class_uid: sub_items::ImplementationClassUid,
    implementation_version_name: Option<sub_items::ImplementationVersionName>,
}

impl UserInformation {
    pub fn size(&self) -> usize {
        4 + self.length as usize
    }

    pub fn length(&self) -> u16 {
        self.length
    }

    pub fn maximum_length(&self) -> Option<&sub_items::MaximumLength> {
        self.maximum_length.as_ref()
    }

    pub fn implementation_class_uid(&self) -> &sub_items::ImplementationClassUid {
        &self.implementation_class_uid
    }

    pub fn implementation_version_name(&self) -> Option<&sub_items::ImplementationVersionName> {
        self.implementation_version_name.as_ref()
    }

    pub fn new(
        maximum_length: Option<sub_items::MaximumLength>,
        implementation_class_uid: sub_items::ImplementationClassUid,
        implementation_version_name: Option<sub_items::ImplementationVersionName>,
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
}

impl TryFrom<&[u8]> for UserInformation {
    type Error = String;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let item = Item::try_from(bytes)?;
        if item.item_type != ITEM_TYPE {
            return Err(INVALID_ITEM_TYPE_ERROR_MESSAGE.to_string());
        }

        let mut offset = 0;
        let mut maximum_length = Option::None;
        let mut implementation_class_uid = Option::None;
        let mut implementation_version_name = Option::None;
        while offset < item.data.len() {
            let sub_item_type = item.data[offset];
            match sub_item_type {
                sub_items::maximum_length::ITEM_TYPE => {
                    maximum_length = {
                        let maximum_length = sub_items::MaximumLength::try_from(
                            &item.data[offset..],
                        )
                        .map_err(|message| {
                            format!("Maximum Length Sub-Item のパースに失敗しました: {message}")
                        })?;
                        offset += maximum_length.size();
                        Some(maximum_length)
                    }
                }
                sub_items::implementation_class_uid::ITEM_TYPE => {
                    implementation_class_uid = {
                        let implementation_class_uid = sub_items::ImplementationClassUid::try_from(
                            &item.data[offset..],
                        )
                        .map_err(|message| {
                            format!(
                                "Implementation Class UID Sub-Item のパースに失敗しました: {message}"
                            )
                        })?;
                        offset += implementation_class_uid.size();
                        Some(implementation_class_uid)
                    }
                }
                sub_items::implementation_version_name::ITEM_TYPE => {
                    implementation_version_name = {
                        let implementation_version_name =
                            sub_items::ImplementationVersionName::try_from(&item.data[offset..])
                                .map_err(|message| {
                                    format!(
                                        "Implementation Version Name Sub-Item のパースに失敗しました: {message}"
                                    )
                                })?;
                        offset += implementation_version_name.size();
                        Some(implementation_version_name)
                    }
                }
                _ => {
                    // TODO: 対応しないサブアイテムの処理。暫定対応として、バイト列をそのまま出力している。
                    println!("未対応の Sub-Item (Item-type=0x{sub_item_type:02X}): [");
                    let sub_item = Item::try_from(&item.data[offset..])?;
                    for i in 0..sub_item.data.len() {
                        print!("0x{:02X} ", sub_item.data[i]);
                    }
                    println!("]");
                    offset += 4 + sub_item.data.len();
                }
            }
        }

        if implementation_class_uid.is_none() {
            return Err("Implementation Class UID Sub-Item が存在しません".to_string());
        }
        let implementation_class_uid = implementation_class_uid.unwrap();

        Ok(UserInformation {
            length: item.length,
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
