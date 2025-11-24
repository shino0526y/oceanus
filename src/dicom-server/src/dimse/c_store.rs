mod instance_info;

use crate::{
    constants::EXPLICIT_VR_BIG_ENDIAN,
    dimse::{
        DimseMessage, buf_to_command_set, buf_to_data_set, c_store::instance_info::InstanceInfo,
    },
};
use dicom_lib::{
    constants::transfer_syntax_uids::IMPLICIT_VR_LITTLE_ENDIAN,
    core::Encoding,
    dictionaries::SOP_CLASS_DICTIONARY,
    network::{
        CommandSet,
        dimse::c_store::{CStoreRq, CStoreRsp, c_store_rsp::Status},
        upper_layer_protocol::pdu::a_abort::Reason,
    },
};
use tracing::{error, info};

pub fn handle_c_store(dimse_message: DimseMessage) -> Result<(Vec<u8>, Vec<u8>), Reason> {
    let command_set_received = buf_to_command_set(dimse_message.command_set_buf)?;

    let c_store_rq = match CStoreRq::try_from(command_set_received) {
        Ok(val) => val,
        Err(e) => {
            error!("C-STORE-RQのパースに失敗しました: {e}");
            return Err(Reason::InvalidPduParameterValue);
        }
    };

    let data_set_received = {
        let encoding = match dimse_message._transfer_syntax_uid {
            IMPLICIT_VR_LITTLE_ENDIAN => Encoding::ImplicitVrLittleEndian,
            EXPLICIT_VR_BIG_ENDIAN => {
                unimplemented!("Explicit VR Big Endianのサポートは未実装です")
            }
            _ => {
                // 暗黙的VRリトルエンディアンと明示的VRビッグエンディアン以外の転送構文に対応するエンコーディングは明示的VRリトルエンディアン
                Encoding::ExplicitVrLittleEndian
            }
        };
        buf_to_data_set(dimse_message.data_set_buf, encoding)?
    };
    let instance_info = InstanceInfo::from_data_set(&data_set_received).map_err(|e| {
        error!("データセットからのインスタンス情報の抽出に失敗しました: {e}");
        Reason::InvalidPduParameterValue
    })?;

    let affected_sop_class_uid = c_store_rq.affected_sop_class_uid();
    let sop_class = SOP_CLASS_DICTIONARY
        .get(affected_sop_class_uid)
        .unwrap_or(&"Unknown SOP Class");

    let patient_id = &instance_info.patient.id;
    let patient_name = {
        let family_name_alphabet = &instance_info.patient.family_name_alphabet;
        let given_name_alphabet = &instance_info.patient.given_name_alphabet;
        let family_name_kanji = &instance_info.patient.family_name_kanji;
        let given_name_kanji = &instance_info.patient.given_name_kanji;
        let family_name_hiragana = &instance_info.patient.family_name_hiragana;
        let given_name_hiragana = &instance_info.patient.given_name_hiragana;
        format!(
            "{}^{}={}^{}={}^{}",
            family_name_alphabet,
            given_name_alphabet,
            family_name_kanji,
            given_name_kanji,
            family_name_hiragana,
            given_name_hiragana
        )
    };
    let study_instance_uid = &instance_info.study.instance_uid;
    let study_id = &instance_info.study.id;
    let study_date_time = {
        let study_date = match &instance_info.study.date {
            Some(date) => date.to_string(),
            None => "".to_string(),
        };
        let study_time = match &instance_info.study.time {
            Some(time) => time.to_string(),
            None => "".to_string(),
        };
        match (!study_date.is_empty(), !study_time.is_empty()) {
            (true, true) => format!("{}T{}", study_date, study_time),
            (true, false) => study_date,
            (false, true) => study_time,
            (false, false) => "".to_string(),
        }
    };
    let accession_number = &instance_info.study.accession_number;
    let series_instance_uid = &instance_info.series.instance_uid;
    let modality = &instance_info.series.modality;
    let series_number = match &instance_info.series.number {
        Some(number) => number.to_string(),
        None => "".to_string(),
    };
    let sop_instance_uid = &instance_info.sop_instance.instance_uid;
    let instance_number = match &instance_info.sop_instance.number {
        Some(number) => number.to_string(),
        None => "".to_string(),
    };

    info!(
        "[{}] C-STORE - {sop_class} (患者ID=\"{patient_id}\", 患者氏名=\"{patient_name}\", 検査インスタンスUID=\"{study_instance_uid}\", 検査ID=\"{study_id}\" 検査日時=\"{study_date_time}\", 受付番号=\"{accession_number}\", シリーズインスタンスUID=\"{series_instance_uid}\", モダリティ=\"{modality}\", シリーズ番号={series_number}, SOPインスタンスUID=\"{sop_instance_uid}\", インスタンス番号={instance_number})",
        dimse_message.context_id,
    );

    let c_store_rsp = CStoreRsp::new(
        c_store_rq.message_id(),
        Status::Success,
        affected_sop_class_uid, // TODO: Stringを借用するようにする
        c_store_rq.affected_sop_instance_uid(), // TODO: Stringを借用するようにする
    );

    let command_set_to_be_sent: CommandSet = c_store_rsp.into();
    let command_set_buf = command_set_to_be_sent.into();

    Ok((command_set_buf, vec![]))
}
