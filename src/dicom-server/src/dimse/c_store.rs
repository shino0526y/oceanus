mod instance_info;

use super::save_file;
use crate::{
    DB_POOL, SERVER_AE_TITLE, STORAGE_DIR,
    constants::{IMPLEMENTATION_CLASS_UID, IMPLEMENTATION_VERSION_NAME},
    dimse::{DimseMessage, SaveFileError, c_store::instance_info::InstanceInfo},
};
use chrono::Datelike;
use dicom_lib::{
    core::{
        DataSet,
        value::value_representations::{ae::AeValue, sh::ShValue, ui::UiValue},
    },
    dictionaries::SOP_CLASS_DICTIONARY,
    file::{File, file_meta_information::FileMetaInformation},
    network::{
        CommandSet,
        dimse::c_store::{CStoreRq, CStoreRsp},
        service_class::storage::{Status, status::code::OutOfResources},
        upper_layer_protocol::pdu::a_abort::Reason,
    },
};
use sqlx::query;
use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};
use tracing::{error, info};

pub async fn handle_c_store(
    command_set: CommandSet,
    data_set: DataSet,
    dimse_message: DimseMessage,
    ae_title: &str,
) -> Result<(Vec<u8>, Vec<u8>), Reason> {
    let c_store_rq = match CStoreRq::try_from(command_set) {
        Ok(val) => val,
        Err(e) => {
            error!("C-STORE-RQのパースに失敗しました: {e}");
            return Err(Reason::InvalidPduParameterValue);
        }
    };

    let c_store_rsp = match handle_c_store_rq(
        c_store_rq,
        data_set,
        dimse_message.transfer_syntax_uid,
        ae_title,
        dimse_message.context_id,
    )
    .await
    {
        Ok(val) => val,
        Err(e) => {
            return Err(e);
        }
    };

    let command_set_to_be_sent: CommandSet = c_store_rsp.into();
    let command_set_buf = command_set_to_be_sent.into();

    Ok((command_set_buf, Vec::new()))
}

/// C-STORE-RQおよび対応するデータセットを処理し、C-STORE-RSPを生成する。
/// SCUが送信したデータが原因で保存に失敗した場合、適切なステータスを持つC-STORE-RSPを返す。
/// SCPの内部エラーが発生した場合、Reasonを返す。
async fn handle_c_store_rq(
    c_store_rq: CStoreRq,
    data_set: DataSet,
    transfer_syntax_uid: &str,
    ae_title: &str,
    context_id: u8,
) -> Result<CStoreRsp, Reason> {
    let affected_sop_class_uid = c_store_rq.affected_sop_class_uid();
    let affected_sop_instance_uid = c_store_rq.affected_sop_instance_uid();
    let file_meta_info = generate_file_meta_info(
        affected_sop_class_uid,
        affected_sop_instance_uid,
        transfer_syntax_uid,
        ae_title,
    );

    let instance_info = {
        match InstanceInfo::from_data_set(&data_set) {
            Ok(val) => val,
            Err((message, error_status)) => {
                error!("データセットからのインスタンス情報の抽出に失敗しました: {message}");

                // データセットをファイルとして保存
                let file = File::new(file_meta_info, data_set);
                let path_buf = generate_failure_path(affected_sop_instance_uid);
                if let Err(e) = save_file(file.into(), &path_buf).await {
                    error!(
                        "インスタンス情報の抽出に失敗したデータセットをファイルとして保存できませんでした: {e}"
                    );
                }

                return Ok(CStoreRsp::new(
                    c_store_rq.message_id(),
                    error_status.into(),
                    affected_sop_class_uid,
                    affected_sop_instance_uid,
                ));
            }
        }
    };

    let sop_class = SOP_CLASS_DICTIONARY
        .get(affected_sop_class_uid)
        .unwrap_or(&"Unknown SOP Class");

    let patient_id = instance_info.patient.id();
    let patient_name = {
        let name_alphabet = instance_info.patient.name_alphabet();
        let name_kanji = instance_info.patient.name_kanji();
        let name_hiragana = instance_info.patient.name_hiragana();
        format!("{name_alphabet}={name_kanji}={name_hiragana}")
    };
    let study_instance_uid = instance_info.study.instance_uid();
    let study_id = instance_info.study.id();
    let study_date_time = {
        let study_date = match instance_info.study.date() {
            Some(date) => date.to_string(),
            None => String::new(),
        };
        let study_time = match instance_info.study.time() {
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
    let accession_number = instance_info.study.accession_number();
    let series_instance_uid = instance_info.series.instance_uid();
    let modality = instance_info.series.modality();
    let series_number = match instance_info.series.number() {
        Some(number) => number.to_string(),
        None => String::new(),
    };
    let sop_instance_uid = instance_info.sop_instance.instance_uid();
    let instance_number = match instance_info.sop_instance.number() {
        Some(number) => number.to_string(),
        None => String::new(),
    };

    // データセットをファイルとして保存
    let file_buf: Vec<u8> = File::new(file_meta_info, data_set).into();
    let file_size = file_buf.len();
    let path_buf = generate_success_path(&instance_info);
    if let Err(e) = save_file(file_buf, &path_buf).await {
        error!("データセットをファイルとして保存できませんでした: {e}");
        let (SaveFileError::CreateDirError { io_error, .. }
        | SaveFileError::WriteFileError { io_error, .. }) = e;
        return match io_error.kind() {
            ErrorKind::StorageFull
            | ErrorKind::FileTooLarge
            | ErrorKind::OutOfMemory
            | ErrorKind::WriteZero => {
                // リソース不足
                Ok(CStoreRsp::new(
                    c_store_rq.message_id(),
                    Status::OutOfResources(OutOfResources::new(0xa700).unwrap()).into(),
                    affected_sop_class_uid,
                    affected_sop_instance_uid,
                ))
            }
            _ => Err(Reason::ReasonNotSpecified),
        };
    }

    // DBへ情報を保存
    if let Err(e) = save_instance_to_db(
        &instance_info,
        ae_title,
        transfer_syntax_uid,
        file_size,
        path_buf.to_str().unwrap(),
    )
    .await
    {
        error!("データベースへの情報の保存に失敗しました: {e}");
        return Err(Reason::ReasonNotSpecified);
    }

    info!(
        "[{context_id}] C-STORE - {sop_class} (患者ID=\"{patient_id}\", 患者氏名=\"{patient_name}\", 検査インスタンスUID=\"{study_instance_uid}\", 検査ID=\"{study_id}\" 検査日時=\"{study_date_time}\", 受付番号=\"{accession_number}\", シリーズインスタンスUID=\"{series_instance_uid}\", モダリティ=\"{modality}\", シリーズ番号={series_number}, SOPインスタンスUID=\"{sop_instance_uid}\", インスタンス番号={instance_number})"
    );

    Ok(CStoreRsp::new(
        c_store_rq.message_id(),
        Status::Success.into(),
        affected_sop_class_uid,
        affected_sop_instance_uid,
    ))
}

fn generate_file_meta_info(
    affected_sop_class_uid: &str,
    affected_sop_instance_uid: &str,
    transfer_syntax_uid: &str,
    ae_title: &str,
) -> FileMetaInformation {
    FileMetaInformation::new(
        UiValue::from_string(affected_sop_class_uid).unwrap(),
        UiValue::from_string(affected_sop_instance_uid).unwrap(),
        UiValue::from_string(transfer_syntax_uid).unwrap(),
        UiValue::from_string(IMPLEMENTATION_CLASS_UID).unwrap(),
        Some(ShValue::from_string(IMPLEMENTATION_VERSION_NAME).unwrap()),
        None,
        Some(AeValue::from_string(ae_title).unwrap()),
        Some(AeValue::from_string(SERVER_AE_TITLE.get().unwrap()).unwrap()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
}

fn generate_success_path(info: &InstanceInfo) -> PathBuf {
    match info.study.date() {
        Some(date) => Path::new(STORAGE_DIR.get().unwrap())
            .join(format!("{:04}", date.year()))
            .join(format!("{:02}", date.month()))
            .join(format!("{:02}", date.day()))
            .join(info.study.instance_uid())
            .join(info.series.instance_uid())
            .join(format!("{}.dcm", info.sop_instance.instance_uid())),
        None => Path::new(STORAGE_DIR.get().unwrap())
            .join("unknown_date")
            .join(info.study.instance_uid())
            .join(info.series.instance_uid())
            .join(format!("{}.dcm", info.sop_instance.instance_uid())),
    }
}

fn generate_failure_path(affected_sop_instance_uid: &str) -> PathBuf {
    Path::new(STORAGE_DIR.get().unwrap())
        .join("failures")
        .join(format!("{affected_sop_instance_uid}.dcm"))
}

async fn save_instance_to_db(
    instance_info: &InstanceInfo,
    ae_title: &str,
    transfer_syntax_uid: &str,
    size: usize,
    path: &str,
) -> Result<(), String> {
    assert!(
        size <= 2147483647,
        "サイズは0から2147483647の範囲である必要があります"
    );

    let mut transaction = DB_POOL
        .get()
        .unwrap()
        .begin()
        .await
        .map_err(|e| format!("トランザクションの開始に失敗しました: {e}"))?;

    // AEのUUIDを取得
    let ae_uuid = query!(
        "SELECT uuid FROM application_entities WHERE title = $1",
        ae_title
    )
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|e| format!("AEの取得に失敗しました: {e}"))?
    .map(|r| r.uuid)
    .ok_or_else(|| format!("送信元AE \"{ae_title}\" が登録されていません"))?;

    query!(
        r#"
        INSERT INTO patients (id, name_alphabet, name_kanji, name_hiragana, birth_date, sex, created_by, created_at, updated_by, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, now(), $7, now())
        ON CONFLICT (id) DO NOTHING
        "#, // 患者についてはもともと登録されていた情報を真とみなす
        instance_info.patient.id(),
        instance_info.patient.name_alphabet(),
        instance_info.patient.name_kanji(),
        instance_info.patient.name_hiragana(),
        instance_info.patient.birth_date(),
        instance_info
            .patient
            .sex()
            .as_ref()
            .map(|s| s.to_iso_5218())
            .unwrap_or(0), // not known
        ae_uuid,
    )
    .execute(&mut *transaction)
    .await
    .map_err(|e| format!("患者情報の保存に失敗しました: {e}"))?;

    query!(
        r#"
        INSERT INTO studies (patient_id, instance_uid, id, study_date, study_time, accession_number, created_by, created_at, updated_by, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, now(), $7, now())
        ON CONFLICT (instance_uid) DO UPDATE SET
            patient_id = EXCLUDED.patient_id,
            id = EXCLUDED.id,
            study_date = EXCLUDED.study_date,
            study_time = EXCLUDED.study_time,
            accession_number = EXCLUDED.accession_number,
            updated_by = EXCLUDED.updated_by,
            updated_at = EXCLUDED.updated_at
        WHERE studies.patient_id IS DISTINCT FROM EXCLUDED.patient_id
           OR studies.id IS DISTINCT FROM EXCLUDED.id
           OR studies.study_date IS DISTINCT FROM EXCLUDED.study_date
           OR studies.study_time IS DISTINCT FROM EXCLUDED.study_time
           OR studies.accession_number IS DISTINCT FROM EXCLUDED.accession_number
        "#,
        instance_info.patient.id(),
        instance_info.study.instance_uid(),
        instance_info.study.id(),
        instance_info.study.date(),
        instance_info.study.time(),
        instance_info.study.accession_number(),
        ae_uuid,
    )
    .execute(&mut *transaction)
    .await
    .map_err(|e| format!("検査情報の保存に失敗しました: {e}"))?;

    query!(
        r#"
        INSERT INTO series (study_instance_uid, instance_uid, modality, series_number, created_by, created_at, updated_by, updated_at)
        VALUES ($1, $2, $3, $4, $5, now(), $5, now())
        ON CONFLICT (instance_uid) DO UPDATE SET
            study_instance_uid = EXCLUDED.study_instance_uid,
            modality = EXCLUDED.modality,
            series_number = EXCLUDED.series_number,
            updated_by = EXCLUDED.updated_by,
            updated_at = EXCLUDED.updated_at
        WHERE series.study_instance_uid IS DISTINCT FROM EXCLUDED.study_instance_uid
           OR series.modality IS DISTINCT FROM EXCLUDED.modality
           OR series.series_number IS DISTINCT FROM EXCLUDED.series_number
        "#,
        instance_info.study.instance_uid(),
        instance_info.series.instance_uid(),
        instance_info.series.modality(),
        instance_info.series.number(),
        ae_uuid,
    )
    .execute(&mut *transaction)
    .await
    .map_err(|e| format!("シリーズ情報の保存に失敗しました: {e}"))?;

    query!(
        r#"
        INSERT INTO sop_instances (series_instance_uid, class_uid, instance_uid, transfer_syntax_uid, size, path, created_by, created_at, updated_by, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, now(), $7, now())
        ON CONFLICT (instance_uid) DO UPDATE SET
            series_instance_uid = EXCLUDED.series_instance_uid,
            class_uid = EXCLUDED.class_uid,
            transfer_syntax_uid = EXCLUDED.transfer_syntax_uid,
            size = EXCLUDED.size,
            path = EXCLUDED.path,
            updated_by = EXCLUDED.updated_by,
            updated_at = EXCLUDED.updated_at
        WHERE sop_instances.series_instance_uid IS DISTINCT FROM EXCLUDED.series_instance_uid
           OR sop_instances.class_uid IS DISTINCT FROM EXCLUDED.class_uid
           OR sop_instances.transfer_syntax_uid IS DISTINCT FROM EXCLUDED.transfer_syntax_uid
           OR sop_instances.size IS DISTINCT FROM EXCLUDED.size
           OR sop_instances.path IS DISTINCT FROM EXCLUDED.path
        "#,
        instance_info.series.instance_uid(),
        instance_info.sop_instance.class_uid(),
        instance_info.sop_instance.instance_uid(),
        transfer_syntax_uid,
        size as i32,
        path,
        ae_uuid,
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
