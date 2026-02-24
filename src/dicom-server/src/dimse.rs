pub mod c_echo;
pub mod c_store;

use crate::STORAGE_DIR;
use dicom_lib::{
    constants::{
        sop_class_uids::{
            COMPUTED_RADIOGRAPHY_IMAGE_STORAGE, CT_IMAGE_STORAGE,
            DIGITAL_MAMMOGRAPHY_X_RAY_IMAGE_STORAGE_FOR_PRESENTATION,
            DIGITAL_X_RAY_IMAGE_STORAGE_FOR_PRESENTATION, MR_IMAGE_STORAGE,
            SECONDARY_CAPTURE_IMAGE_STORAGE, VERIFICATION, X_RAY_ANGIOGRAPHIC_IMAGE_STORAGE,
            X_RAY_RADIOFLUOROSCOPIC_IMAGE_STORAGE,
        },
        transfer_syntax_uids::{EXPLICIT_VR_BIG_ENDIAN, IMPLICIT_VR_LITTLE_ENDIAN},
    },
    core::{DataSet, Encoding},
    network::{CommandSet, upper_layer_protocol::pdu::a_abort::Reason},
};
use std::{
    io::Cursor,
    path::{Path, PathBuf},
};
use tokio::fs;
use tracing::{error, info};

pub struct DimseMessage {
    pub context_id: u8,
    pub abstract_syntax_uid: String,
    pub transfer_syntax_uid: &'static str,
    pub command_set_buf: Vec<u8>,
    pub data_set_buf: Vec<u8>,
    pub is_command_received: bool,
    pub is_data_received: bool,
}

fn parse_command_set(buf: &[u8]) -> Result<CommandSet, Reason> {
    let mut cur = Cursor::new(buf);
    match CommandSet::read_from_cur(&mut cur) {
        Ok(val) => Ok(val),
        Err(e) => {
            error!("コマンドセットのパースに失敗しました: {e}");
            Err(Reason::InvalidPduParameterValue)
        }
    }
}

fn parse_data_set(buf: &[u8], encoding: Encoding) -> Result<DataSet, Reason> {
    let mut cur = Cursor::new(buf);
    match DataSet::read_from_cur(&mut cur, encoding) {
        Ok(val) => Ok(val),
        Err(e) => {
            error!("データセットのパースに失敗しました: {e}");
            Err(Reason::InvalidPduParameterValue)
        }
    }
}

pub async fn handle_dimse_message(
    dimse_message: DimseMessage,
    ae_title: &str,
) -> Result<(Vec<u8>, Vec<u8>), Reason> {
    let command_set = match parse_command_set(&dimse_message.command_set_buf) {
        Ok(val) => val,
        Err(e) => {
            match dump(
                dimse_message.command_set_buf,
                ae_title,
                DumpType::CommandSet,
            )
            .await
            {
                Ok(path) => {
                    info!(
                        "パースに失敗したコマンドセットをダンプファイルとして保存しました (パス=\"{}\")",
                        path.display()
                    );
                }
                Err(e) => {
                    error!(
                        "パースに失敗したコマンドセットをダンプファイルとして保存できませんでした: {}",
                        e
                    );
                }
            }
            return Err(e);
        }
    };

    match dimse_message.abstract_syntax_uid.as_str() {
        VERIFICATION => c_echo::handle_c_echo(command_set, dimse_message.context_id),
        COMPUTED_RADIOGRAPHY_IMAGE_STORAGE
        | DIGITAL_X_RAY_IMAGE_STORAGE_FOR_PRESENTATION
        | DIGITAL_MAMMOGRAPHY_X_RAY_IMAGE_STORAGE_FOR_PRESENTATION
        | CT_IMAGE_STORAGE
        | MR_IMAGE_STORAGE
        | SECONDARY_CAPTURE_IMAGE_STORAGE
        | X_RAY_ANGIOGRAPHIC_IMAGE_STORAGE
        | X_RAY_RADIOFLUOROSCOPIC_IMAGE_STORAGE => {
            let data_set = {
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

                match parse_data_set(dimse_message.data_set_buf.as_ref(), encoding) {
                    Ok(val) => val,
                    Err(e) => {
                        match dump(dimse_message.data_set_buf, ae_title, DumpType::DataSet).await {
                            Ok(path) => {
                                info!(
                                    "パースに失敗したデータセットをダンプファイルとして保存しました (パス=\"{}\")",
                                    path.display()
                                );
                            }
                            Err(e) => {
                                error!(
                                    "パースに失敗したデータセットをダンプファイルとして保存できませんでした: {}",
                                    e
                                );
                            }
                        }
                        return Err(e);
                    }
                }
            };
            c_store::handle_c_store(command_set, data_set, dimse_message, ae_title).await
        }
        _ => unreachable!(),
    }
}

enum DumpType {
    CommandSet,
    DataSet,
}

async fn dump(buf: Vec<u8>, ae_title: &str, dump_type: DumpType) -> Result<PathBuf, SaveFileError> {
    let now = chrono::Utc::now().format("%Y%m%d%H%M%S%6f").to_string();
    let dump_type = match dump_type {
        DumpType::CommandSet => "commandset",
        DumpType::DataSet => "dataset",
    };

    let path_buf = Path::new(STORAGE_DIR.get().unwrap())
        .join("dump")
        .join(format!("{now}_{ae_title}_{dump_type}.dump"));

    save_file(buf, &path_buf).await?;
    Ok(path_buf)
}

#[derive(Debug, thiserror::Error)]
enum SaveFileError {
    #[error("ディレクトリの作成に失敗しました (パス=\"{}\"): {io_error}", path_buf.display())]
    CreateDirError {
        path_buf: PathBuf,
        io_error: std::io::Error,
    },
    #[error("ファイルの書き込みに失敗しました (パス=\"{}\"): {io_error}", path_buf.display())]
    WriteFileError {
        path_buf: PathBuf,
        io_error: std::io::Error,
    },
}

async fn save_file(buf: Vec<u8>, path: &Path) -> Result<(), SaveFileError> {
    // ディレクトリが存在しない場合は作成
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| SaveFileError::CreateDirError {
                path_buf: parent.to_path_buf(),
                io_error: e,
            })?;
    }

    fs::write(path, buf)
        .await
        .map_err(|e| SaveFileError::WriteFileError {
            path_buf: path.to_path_buf(),
            io_error: e,
        })?;

    Ok(())
}
