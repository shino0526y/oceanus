mod instance_info;

use crate::{
    DB_POOL,
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
use sqlx::query;
use tracing::{error, info};

async fn save_instance_to_db(instance_info: &InstanceInfo, ae_title: &str) -> Result<(), String> {
    let mut transaction = DB_POOL
        .get()
        .unwrap()
        .begin()
        .await
        .map_err(|e| format!("トランザクションの開始に失敗しました: {e}"))?;

    query!(
        r#"
        INSERT INTO patients (id, name_alphabet, name_kanji, name_hiragana, birth_date, sex)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (id) DO NOTHING
        "#,
        instance_info.patient.id,
        instance_info.patient.name_alphabet,
        instance_info.patient.name_kanji,
        instance_info.patient.name_hiragana,
        instance_info.patient.birth_date,
        instance_info.patient.sex.as_ref().map(|s| s.to_smallint()),
    )
    .execute(&mut *transaction)
    .await
    .map_err(|e| format!("患者情報の保存に失敗しました: {e}"))?;

    query!(
        r#"
        INSERT INTO studies (patient_id, instance_uid, id, study_date, study_time, accession_number, ae_title)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (instance_uid) DO NOTHING
        "#,
        instance_info.patient.id,
        instance_info.study.instance_uid,
        instance_info.study.id,
        instance_info.study.date,
        instance_info.study.time,
        instance_info.study.accession_number,
        ae_title,
    )
    .execute(&mut *transaction)
    .await
    .map_err(|e| format!("検査情報の保存に失敗しました: {e}"))?;

    query!(
        r#"
        INSERT INTO series (study_instance_uid, instance_uid, modality, series_number)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (instance_uid) DO NOTHING
        "#,
        instance_info.study.instance_uid,
        instance_info.series.instance_uid,
        instance_info.series.modality,
        instance_info.series.number,
    )
    .execute(&mut *transaction)
    .await
    .map_err(|e| format!("シリーズ情報の保存に失敗しました: {e}"))?;

    // TODO: pathの実装
    let path = "".to_string();
    query!(
        r#"
        INSERT INTO sop_instances (series_instance_uid, class_uid, instance_uid, path)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (instance_uid) DO NOTHING
        "#,
        instance_info.series.instance_uid,
        instance_info.sop_instance.class_uid,
        instance_info.sop_instance.instance_uid,
        path,
    )
    .execute(&mut *transaction)
    .await
    .map_err(|e| format!("SOPインスタンス情報の保存に失敗しました: {e}"))?;

    transaction
        .commit()
        .await
        .map_err(|e| format!("トランザクションのコミットに失敗しました: {e}"))?;

    Ok(())
}

pub async fn handle_c_store(
    dimse_message: DimseMessage,
    ae_title: &str,
) -> Result<(Vec<u8>, Vec<u8>), Reason> {
    let command_set_received = buf_to_command_set(dimse_message.command_set_buf)?;

    let c_store_rq = match CStoreRq::try_from(command_set_received) {
        Ok(val) => val,
        Err(e) => {
            error!("C-STORE-RQのパースに失敗しました: {e}");
            return Err(Reason::InvalidPduParameterValue);
        }
    };

    let data_set_received = {
        let encoding = match dimse_message.transfer_syntax_uid {
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
        let name_alphabet = &instance_info.patient.name_alphabet;
        let name_kanji = &instance_info.patient.name_kanji;
        let name_hiragana = &instance_info.patient.name_hiragana;
        format!("{name_alphabet}={name_kanji}={name_hiragana}")
    };
    let study_instance_uid = &instance_info.study.instance_uid;
    let study_id = &instance_info.study.id;
    let study_date_time = {
        let study_date = match &instance_info.study.date {
            Some(date) => date.to_string(),
            None => String::new(),
        };
        let study_time = match &instance_info.study.time {
            Some(time) => time.to_string(),
            None => String::new(),
        };
        match (!study_date.is_empty(), !study_time.is_empty()) {
            (true, true) => format!("{}T{}", study_date, study_time),
            (true, false) => study_date,
            (false, true) => study_time,
            (false, false) => String::new(),
        }
    };
    let accession_number = &instance_info.study.accession_number;
    let series_instance_uid = &instance_info.series.instance_uid;
    let modality = &instance_info.series.modality;
    let series_number = match &instance_info.series.number {
        Some(number) => number.to_string(),
        None => String::new(),
    };
    let sop_instance_uid = &instance_info.sop_instance.instance_uid;
    let instance_number = match &instance_info.sop_instance.number {
        Some(number) => number.to_string(),
        None => String::new(),
    };

    // DBへ情報を保存
    if let Err(e) = save_instance_to_db(&instance_info, ae_title).await {
        error!("データベースへの情報の保存に失敗しました: {e}");
        return Err(Reason::ReasonNotSpecified);
    }

    info!(
        "[{}] C-STORE - {sop_class} (患者ID=\"{patient_id}\", 患者氏名=\"{patient_name}\", 検査インスタンスUID=\"{study_instance_uid}\", 検査ID=\"{study_id}\" 検査日時=\"{study_date_time}\", 受付番号=\"{accession_number}\", シリーズインスタンスUID=\"{series_instance_uid}\", モダリティ=\"{modality}\", シリーズ番号={series_number}, SOPインスタンスUID=\"{sop_instance_uid}\", インスタンス番号={instance_number})",
        dimse_message.context_id,
    );

    let c_store_rsp = CStoreRsp::new(
        c_store_rq.message_id(),
        Status::Success,
        affected_sop_class_uid,
        c_store_rq.affected_sop_instance_uid(),
    );

    let command_set_to_be_sent: CommandSet = c_store_rsp.into();
    let command_set_buf = command_set_to_be_sent.into();

    Ok((command_set_buf, vec![]))
}
