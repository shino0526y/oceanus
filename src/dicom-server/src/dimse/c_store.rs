mod instance_info;

use crate::{
    DB_POOL, SERVER_AE_TITLE, STORAGE_DIR,
    constants::{EXPLICIT_VR_BIG_ENDIAN, IMPLEMENTATION_CLASS_UID, IMPLEMENTATION_VERSION_NAME},
    dimse::{
        DimseMessage, buf_to_command_set, buf_to_data_set, c_store::instance_info::InstanceInfo,
    },
};
use chrono::Datelike;
use dicom_lib::{
    constants::transfer_syntax_uids::IMPLICIT_VR_LITTLE_ENDIAN,
    core::{
        Encoding,
        value::value_representations::{ae::AeValue, sh::ShValue, ui::UiValue},
    },
    dictionaries::SOP_CLASS_DICTIONARY,
    file::{File, file_meta_information::FileMetaInformation},
    network::{
        CommandSet,
        dimse::c_store::{CStoreRq, CStoreRsp, c_store_rsp::Status},
        upper_layer_protocol::pdu::a_abort::Reason,
    },
};
use sqlx::query;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{error, info};

async fn save_instance_to_db(
    instance_info: &InstanceInfo,
    ae_title: &str,
    path: &str,
) -> Result<(), String> {
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
        "#, // 患者についてはもともと登録されていた情報を真とみなす
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
        ON CONFLICT (instance_uid) DO UPDATE SET
            patient_id = EXCLUDED.patient_id,
            id = EXCLUDED.id,
            study_date = EXCLUDED.study_date,
            study_time = EXCLUDED.study_time,
            accession_number = EXCLUDED.accession_number,
            ae_title = EXCLUDED.ae_title
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
        ON CONFLICT (instance_uid) DO UPDATE SET
            study_instance_uid = EXCLUDED.study_instance_uid,
            modality = EXCLUDED.modality,
            series_number = EXCLUDED.series_number
        "#,
        instance_info.study.instance_uid,
        instance_info.series.instance_uid,
        instance_info.series.modality,
        instance_info.series.number,
    )
    .execute(&mut *transaction)
    .await
    .map_err(|e| format!("シリーズ情報の保存に失敗しました: {e}"))?;

    query!(
        r#"
        INSERT INTO sop_instances (series_instance_uid, class_uid, instance_uid, path)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (instance_uid) DO UPDATE SET
            series_instance_uid = EXCLUDED.series_instance_uid,
            class_uid = EXCLUDED.class_uid,
            path = EXCLUDED.path
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
    mut dimse_message: DimseMessage,
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

    let affected_sop_class_uid = c_store_rq.affected_sop_class_uid();
    let affected_sop_instance_uid = c_store_rq.affected_sop_instance_uid();
    let transfer_syntax_uid = dimse_message.transfer_syntax_uid;
    let file_meta_info = generate_file_meta_info(
        affected_sop_class_uid,
        affected_sop_instance_uid,
        transfer_syntax_uid,
        ae_title,
    );

    let data_set_received = {
        let encoding = match transfer_syntax_uid {
            IMPLICIT_VR_LITTLE_ENDIAN => Encoding::ImplicitVrLittleEndian,
            EXPLICIT_VR_BIG_ENDIAN => {
                unimplemented!("Explicit VR Big Endianのサポートは未実装です")
            }
            _ => {
                // 暗黙的VRリトルエンディアンと明示的VRビッグエンディアン以外の転送構文に対応するエンコーディングは明示的VRリトルエンディアン
                Encoding::ExplicitVrLittleEndian
            }
        };

        match buf_to_data_set(dimse_message.data_set_buf.as_ref(), encoding) {
            Ok(val) => val,
            Err(e) => {
                // ファイルに保存
                let mut buf: Vec<u8> = file_meta_info.into();
                buf.append(&mut dimse_message.data_set_buf);
                let path = generate_failure_path(affected_sop_instance_uid);
                save_file_to_storage(buf, &path).await.unwrap_or_else(|e| {
                    let path = path.to_str().unwrap();
                    error!("パースに失敗したデータセットをファイルとして保存できませんでした (パス=\"{path}\")): {e}");
                });

                return Err(e);
            }
        }
    };

    let instance_info = {
        let info = InstanceInfo::from_data_set(&data_set_received);
        if let Err(e) = &info {
            error!("データセットからのインスタンス情報の抽出に失敗しました: {e}");

            // データセットをファイルとして保存
            let file = File::new(file_meta_info, data_set_received);
            let path = generate_failure_path(affected_sop_instance_uid);
            if let Err(e) = save_file_to_storage(file.into(), &path).await {
                let path = path.to_str().unwrap();
                error!(
                    "インスタンス情報の抽出に失敗したデータセットをファイルとして保存できませんでした (パス=\"{path}\")): {e}"
                );
            }

            return Err(Reason::InvalidPduParameterValue);
        }

        info.unwrap()
    };

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

    // データセットをファイルとして保存
    let file = File::new(file_meta_info, data_set_received);
    let path = generate_success_path(&instance_info);
    if let Err(e) = save_file_to_storage(file.into(), &path).await {
        let path = path.to_str().unwrap();
        error!("データセットをファイルとして保存できませんでした (パス=\"{path}\"): {e}");
        return Err(Reason::ReasonNotSpecified);
    }
    // DBへ情報を保存
    if let Err(e) = save_instance_to_db(&instance_info, ae_title, path.to_str().unwrap()).await {
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
        affected_sop_instance_uid,
    );

    let command_set_to_be_sent: CommandSet = c_store_rsp.into();
    let command_set_buf = command_set_to_be_sent.into();

    Ok((command_set_buf, Vec::new()))
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
    match info.study.date {
        Some(date) => Path::new(STORAGE_DIR.get().unwrap())
            .join(format!("{:04}", date.year()))
            .join(format!("{:02}", date.month()))
            .join(format!("{:02}", date.day()))
            .join(&info.study.instance_uid)
            .join(&info.series.instance_uid)
            .join(format!("{}.dcm", info.sop_instance.instance_uid)),
        None => Path::new(STORAGE_DIR.get().unwrap())
            .join("unknown_date")
            .join(&info.study.instance_uid)
            .join(&info.series.instance_uid)
            .join(format!("{}.dcm", info.sop_instance.instance_uid)),
    }
}

fn generate_failure_path(affected_sop_instance_uid: &str) -> PathBuf {
    Path::new(STORAGE_DIR.get().unwrap())
        .join("failures")
        .join(format!("{}.dcm", affected_sop_instance_uid))
}

async fn save_file_to_storage(buf: Vec<u8>, path: &Path) -> Result<(), String> {
    // ディレクトリが存在しない場合は作成
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await.map_err(|e| {
            format!(
                "ディレクトリの作成に失敗しました (パス=\"{}\"): {e}",
                parent.to_str().unwrap()
            )
        })?;
    }

    // ファイルに書き込み
    fs::write(path, buf).await.map_err(|e| {
        format!(
            "ファイルの書き込みに失敗しました (パス=\"{}\"): {e}",
            path.to_str().unwrap()
        )
    })?;
    Ok(())
}
