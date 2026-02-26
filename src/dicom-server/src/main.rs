mod args;
mod constants;
mod dimse;

use crate::{
    args::Args,
    constants::*,
    dimse::{DimseMessage, handle_dimse_message},
};
use clap::Parser;
use dicom_lib::{
    constants::{sop_class_uids::VERIFICATION, transfer_syntax_uids::IMPLICIT_VR_LITTLE_ENDIAN},
    network::{
        command_set::utils::generate_p_data_tf_pdus,
        upper_layer_protocol::{
            AReleaseRqReception, PDataTfReception,
            pdu::{
                AAssociateAc, AAssociateRq, PDataTf, PduReadError,
                a_abort::{self, Source},
                a_associate_ac::{
                    self, ApplicationContext, UserInformation,
                    presentation_context::{ResultReason, TransferSyntax},
                    user_information::{
                        ImplementationClassUid, ImplementationVersionName, MaximumLength,
                    },
                },
                a_associate_rj::{
                    self, SourceAndReason,
                    source::{service_provider_acse, service_user},
                },
                a_associate_rq,
                p_data_tf::PresentationDataValue,
            },
            receive_a_associate_rq, receive_a_release_rq, receive_p_data_tf, send_a_abort,
            send_a_associate_ac, send_a_associate_rj, send_a_release_rp, send_p_data_tf,
        },
    },
};
use dotenvy::dotenv;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions, query};
use std::{
    collections::{HashMap, HashSet},
    io::{ErrorKind, IsTerminal},
    net::Ipv4Addr,
    path::{Path, PathBuf},
    process::exit,
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::BufReader,
    net::{TcpListener, TcpStream, lookup_host},
    spawn,
};
use tracing::{Instrument, Level, debug, error, info, level_filters::LevelFilter, span, warn};
use tracing_subscriber::fmt::time::LocalTime;

static CONNECTION_COUNTER: AtomicU64 = AtomicU64::new(1);
static SERVER_AE_TITLE: OnceLock<String> = OnceLock::new();
static DB_POOL: OnceLock<Pool<Postgres>> = OnceLock::new();
static STORAGE_DIR: OnceLock<PathBuf> = OnceLock::new();

#[tokio::main]
async fn main() {
    // 環境変数の読み込み（失敗しても無視）
    let _ = dotenv();
    // コマンドライン引数の解析
    let args = Args::parse();
    SERVER_AE_TITLE.set(args.ae_title).unwrap();
    STORAGE_DIR // ストレージ先ディレクトリはデータディレクトリの直下の`dicom`ディレクトリとする
        .set(Path::new(&args.data_dir).join("dicom"))
        .unwrap();

    print!(
        r"
 ██████╗  ██████╗███████╗ █████╗ ███╗   ██╗██╗   ██╗███████╗
██╔═══██╗██╔════╝██╔════╝██╔══██╗████╗  ██║██║   ██║██╔════╝
██║   ██║██║     █████╗  ███████║██╔██╗ ██║██║   ██║███████╗
██║   ██║██║     ██╔══╝  ██╔══██║██║╚██╗██║██║   ██║╚════██║
╚██████╔╝╚██████╗███████╗██║  ██║██║ ╚████║╚██████╔╝███████║
 ╚═════╝  ╚═════╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚══════╝
{:>60}

",
        env!("CARGO_PKG_VERSION")
    );

    // ログ設定
    {
        let is_tty = std::io::stdout().is_terminal();
        let log_level_filter: LevelFilter = args.log_level.into();

        tracing_subscriber::fmt()
            .with_ansi(is_tty)
            .with_timer(LocalTime::rfc_3339())
            .with_max_level(log_level_filter)
            .with_target(false)
            .init();
    }

    // DB 接続
    match PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&args.database_url)
        .await
    {
        Ok(pool) => {
            DB_POOL.set(pool).unwrap();
            debug!("データベースに接続しました");
        }
        Err(e) => {
            error!("データベースへの接続に失敗しました: {e}");
            exit(1);
        }
    }

    let listener = {
        match TcpListener::bind((Ipv4Addr::UNSPECIFIED, args.port)).await {
            Ok(val) => val,
            Err(e) => {
                error!(
                    "通信の待ち受けに失敗しました (ポート番号={}): {e}",
                    args.port
                );
                exit(1);
            }
        }
    };
    info!(
        "サーバーが起動しました (AEタイトル=\"{}\" ポート番号={})",
        SERVER_AE_TITLE.get().unwrap(),
        args.port
    );

    let shutdown = shutdown_signal();
    tokio::pin!(shutdown);

    loop {
        let (socket, addr) = tokio::select! {
            _ = &mut shutdown => {
                break;
            }
            res = listener.accept() => {
                match res {
                    Ok(val) => val,
                    Err(e) => {
                        error!("接続の受け入れに失敗しました: {e}");
                        continue;
                    }
                }
            }
        };
        let connection_id = CONNECTION_COUNTER.fetch_add(1, Ordering::Relaxed);

        spawn(async move {
            use Instrument;
            handle_association(socket)
                .instrument(span!(
                    Level::INFO,
                    "connection",
                    ID = connection_id,
                    IP = format!("{}", addr.ip()),
                    Port = addr.port()
                ))
                .await;
        });
    }
}

async fn handle_association(mut socket: TcpStream) {
    let mut buf_reader = BufReader::new(&mut socket);

    let (a_associate_rq, mut context_id_to_dimse_message) =
        match handle_association_establishment(&mut buf_reader).await {
            Some(val) => val,
            None => return,
        };
    let maximum_length = a_associate_rq
        .user_information()
        .maximum_length()
        .map_or(0, |maximum_length| maximum_length.maximum_length());

    // サービス処理
    let accepted_context_ids = context_id_to_dimse_message
        .keys()
        .copied()
        .collect::<HashSet<_>>();
    loop {
        // P-DATA-TFの受信
        let p_data_tf = {
            let reception = match receive_p_data_tf(&mut buf_reader).await {
                Ok(val) => val,
                Err(e) => {
                    error!("P-DATA-TFの受信に失敗しました: {e}");
                    if !matches!(e, PduReadError::IoError(_)) {
                        abort(&mut buf_reader, a_abort::Reason::from(e)).await;
                    }
                    return;
                }
            };

            match reception {
                PDataTfReception::PDataTf(val) => val,
                PDataTfReception::AReleaseRq(_) => {
                    debug!("A-RELEASE-RQを受信しました");
                    release(&mut buf_reader).await;
                    return;
                }
                PDataTfReception::AAbort(a_abort) => {
                    debug!(
                        "A-ABORTを受信しました: (Source={:02X} Reason={:02X})",
                        a_abort.source() as u8,
                        a_abort.reason() as u8
                    );
                    return;
                }
            }
        };
        debug!("P-DATA-TFを受信しました");

        // 受信したP-DATA-TFからDIMSEメッセージを復元し、Presentation Context IDごとに処理
        // TODO: 並列化
        let pdvs = PDataTf::extract_presentation_data_values(p_data_tf);
        for pdv in pdvs {
            let context_id = pdv.presentation_context_id();
            if !accepted_context_ids.contains(&context_id) {
                error!(
                    "受信したP-DATA-TFにアソシエーションで受諾していないPresentation Context IDが含まれています (ContextID={context_id})"
                );
                abort(&mut buf_reader, a_abort::Reason::InvalidPduParameterValue).await;
                return;
            } else if !context_id_to_dimse_message.contains_key(&context_id) {
                error!(
                    "受信したP-DATA-TFにすでに処理済みのPresentation Context IDが含まれています (ContextID={context_id})"
                );
                abort(&mut buf_reader, a_abort::Reason::InvalidPduParameterValue).await;
                return;
            }

            let dimse_message = context_id_to_dimse_message.get_mut(&context_id).unwrap();
            let is_command = pdv.is_command();
            let is_last = pdv.is_last();
            let fragment = &mut PresentationDataValue::extract_fragment(pdv);
            if is_command {
                dimse_message.command_set_buf.append(fragment);
                dimse_message.is_command_received = is_last;
            } else {
                dimse_message.data_set_buf.append(fragment);
                dimse_message.is_data_received = is_last;
            }

            if !(dimse_message.is_command_received && dimse_message.is_data_received) {
                continue;
            }

            // DIMSEメッセージを取り出し、空のDIMSEメッセージを生成し登録しなおす。
            // これにより、同じPresentation Context IDで複数のDIMSEメッセージを処理できるようにする。
            let dimse_message = context_id_to_dimse_message.remove(&context_id).unwrap();
            context_id_to_dimse_message.insert(
                context_id,
                generate_empty_dimse_message(
                    context_id,
                    &dimse_message.abstract_syntax_uid,
                    dimse_message.transfer_syntax_uid,
                ),
            );

            let (command_set_buf, data_set_buf) = match handle_dimse_message(
                dimse_message,
                a_associate_rq.calling_ae_title(),
            )
            .await
            {
                Ok(val) => val,
                Err(reason) => {
                    abort(&mut buf_reader, reason).await;
                    return;
                }
            };

            // P-DATA-TFの送信
            {
                let p_data_tf_pdus = generate_p_data_tf_pdus(
                    context_id,
                    command_set_buf,
                    data_set_buf,
                    maximum_length,
                );

                match send_p_data_tf(&mut buf_reader.get_mut(), p_data_tf_pdus).await {
                    Ok(()) => {}
                    Err(e) => {
                        error!("P-DATA-TFの送信に失敗しました: {e}");
                        return;
                    }
                }
                debug!("P-DATA-TFを送信しました");
            }
        }

        if context_id_to_dimse_message.is_empty() {
            break;
        }
    }

    handle_association_release(&mut buf_reader).await;

    info!("アソシエーションを正常に終了しました");
}

async fn handle_association_establishment(
    buf_reader: &mut BufReader<&mut TcpStream>,
) -> Option<(AAssociateRq, HashMap<u8, DimseMessage>)> {
    // A-ASSOCIATE-RQの受信
    let a_associate_rq = match receive_a_associate_rq(buf_reader).await {
        Ok(val) => val,
        Err(e) => {
            if let PduReadError::IoError(io_err) = &e
                && io_err.kind() == ErrorKind::UnexpectedEof
            {
                // `nc -z`等のヘルスチェックによる接続終了を想定し、debugログにとどめる
                debug!("TCP接続がデータ受信前に閉じられました");
                return None;
            }

            error!("A-ASSOCIATE-RQの受信に失敗しました: {e}");
            return None;
        }
    };
    let called_ae_title = a_associate_rq.called_ae_title();
    let calling_ae_title = a_associate_rq.calling_ae_title();
    debug!(
        "A-ASSOCIATE-RQを受信しました (呼出元=\"{calling_ae_title}\" 宛先=\"{called_ae_title}\")"
    );

    // アソシエーション要求を受諾するか判定し、拒否する場合はA-ASSOCIATE-RJを送信して終了する
    if called_ae_title != SERVER_AE_TITLE.get().unwrap() {
        warn!(
            "アソシエーション要求を拒否しました (呼出元=\"{calling_ae_title}\" 宛先=\"{called_ae_title}\" 理由=宛先AEタイトル不一致)",
        );
        reject_association(
            buf_reader,
            a_associate_rj::Result::RejectedPermanent,
            SourceAndReason::ServiceUser(service_user::Reason::CalledAeTitleNotRecognized),
        )
        .await;
        return None;
    }

    match query!(
        "SELECT host FROM application_entities WHERE title = $1",
        calling_ae_title
    )
    .fetch_one(DB_POOL.get().unwrap())
    .await
    {
        Ok(application_entity) => {
            let mut host_addresses = {
                match lookup_host((application_entity.host.as_str(), 0)).await {
                    Ok(addresses) => addresses,
                    Err(e) => {
                        warn!(
                            "アソシエーション要求を拒否しました (呼出元=\"{calling_ae_title}\" 宛先=\"{called_ae_title}\" 理由=ホスト名の解決に失敗): {e}",
                        );
                        reject_association(
                            buf_reader,
                            a_associate_rj::Result::RejectedTransient,
                            SourceAndReason::ServiceProviderAcse(
                                service_provider_acse::Reason::NoReasonGiven,
                            ),
                        )
                        .await;
                        return None;
                    }
                }
            };
            let peer_address = buf_reader.get_ref().peer_addr().unwrap();
            let is_matched = host_addresses.any(|addr| addr.ip() == peer_address.ip());

            if !is_matched {
                warn!(
                    "アソシエーション要求を拒否しました (呼出元=\"{calling_ae_title}\" 宛先=\"{called_ae_title}\" 理由=呼出元AEタイトル不明)",
                );
                reject_association(
                    buf_reader,
                    a_associate_rj::Result::RejectedPermanent,
                    SourceAndReason::ServiceUser(service_user::Reason::CallingAeTitleNotRecognized),
                )
                .await;
                return None;
            }
        }
        Err(_) => {
            warn!(
                "アソシエーション要求を拒否しました (呼出元=\"{calling_ae_title}\" 宛先=\"{called_ae_title}\" 理由=呼出元AEタイトル不明)",
            );
            reject_association(
                buf_reader,
                a_associate_rj::Result::RejectedPermanent,
                SourceAndReason::ServiceUser(service_user::Reason::CallingAeTitleNotRecognized),
            )
            .await;
            return None;
        }
    }

    // A-ASSOCIATE-ACの送信
    let mut context_id_to_dimse_message = HashMap::new();
    {
        let application_context = ApplicationContext::new("1.2.840.10008.3.1.1.1.1");
        let presentation_contexts = a_associate_rq
            .presentation_contexts()
            .iter()
            .map(|presentation_context| {
                if !is_abstract_syntax_supported(presentation_context) {
                    a_associate_ac::PresentationContext::new(
                        presentation_context.context_id(),
                        ResultReason::AbstractSyntaxNotSupported,
                        TransferSyntax::new(IMPLICIT_VR_LITTLE_ENDIAN).unwrap(),
                    )
                } else if !is_transfer_syntax_supported(presentation_context) {
                    a_associate_ac::PresentationContext::new(
                        presentation_context.context_id(),
                        ResultReason::TransferSyntaxesNotSupported,
                        TransferSyntax::new(IMPLICIT_VR_LITTLE_ENDIAN).unwrap(),
                    )
                } else {
                    let transfer_syntax_uid = choose_transfer_syntax_uid(presentation_context);
                    let pc = a_associate_ac::PresentationContext::new(
                        presentation_context.context_id(),
                        ResultReason::Acceptance,
                        TransferSyntax::new(transfer_syntax_uid).unwrap(),
                    );

                    {
                        let context_id = pc.context_id();
                        let abstract_syntax = presentation_context.abstract_syntax().name();

                        context_id_to_dimse_message.insert(
                            context_id,
                            generate_empty_dimse_message(
                                context_id,
                                abstract_syntax,
                                transfer_syntax_uid,
                            ),
                        );
                    }

                    pc
                }
            })
            .collect::<Vec<_>>();
        let user_information = UserInformation::new(
            Some(MaximumLength::new(MAXIMUM_LENGTH)),
            ImplementationClassUid::new(IMPLEMENTATION_CLASS_UID).unwrap(),
            Some(ImplementationVersionName::new(IMPLEMENTATION_VERSION_NAME).unwrap()),
        );
        let a_associate_ac = AAssociateAc::new(
            1,
            called_ae_title,
            calling_ae_title,
            application_context,
            presentation_contexts,
            user_information,
        )
        .unwrap();

        if let Err(e) = send_a_associate_ac(&mut buf_reader.get_mut(), a_associate_ac).await {
            error!("A-ASSOCIATE-ACの送信に失敗しました: {e}");
            return None;
        };
    }
    debug!("A-ASSOCIATE-ACを送信しました");

    if context_id_to_dimse_message.is_empty() {
        warn!(
            "アソシエーション要求について受諾可能なプレゼンテーションコンテキストがありません (呼出元=\"{calling_ae_title}\")"
        );
        return None;
    }
    info!("アソシエーション要求を受諾しました (呼出元=\"{calling_ae_title}\")");

    Some((a_associate_rq, context_id_to_dimse_message))
}

fn generate_empty_dimse_message(
    context_id: u8,
    abstract_syntax_uid: &str,
    transfer_syntax_uid: &'static str,
) -> DimseMessage {
    let is_data_received = match abstract_syntax_uid {
        VERIFICATION => true, // C-ECHOはデータセットを使用しないため、すでにデータセットを受信したものとみなす
        _ => false,
    };

    DimseMessage {
        context_id,
        abstract_syntax_uid: abstract_syntax_uid.to_string(),
        transfer_syntax_uid,
        command_set_buf: Vec::new(),
        data_set_buf: Vec::new(),
        is_command_received: false,
        is_data_received,
    }
}

fn is_abstract_syntax_supported(
    presentation_context: &a_associate_rq::PresentationContext,
) -> bool {
    SUPPORTED_ABSTRACT_SYNTAX_UIDS.contains(&presentation_context.abstract_syntax().name())
}

fn is_transfer_syntax_supported(
    presentation_context: &a_associate_rq::PresentationContext,
) -> bool {
    SUPPORTED_TRANSFER_SYNTAX_UIDS
        .iter()
        .any(|transfer_syntax| {
            presentation_context
                .transfer_syntaxes()
                .iter()
                .any(|ts| ts.name() == *transfer_syntax)
        })
}

fn choose_transfer_syntax_uid(
    presentation_context: &a_associate_rq::PresentationContext,
) -> &'static str {
    let transfer_syntax_uids = presentation_context
        .transfer_syntaxes()
        .iter()
        .map(|uid| uid.name())
        .collect::<Vec<_>>();

    // サポートされている転送構文UIDの中から最初にマッチしたもの（優先度が高いもの）を取り出す
    let uid = SUPPORTED_TRANSFER_SYNTAX_UIDS
        .iter()
        .find(|uid| transfer_syntax_uids.contains(uid));

    // 取り出した転送構文UIDを返す
    // 見つからなかった場合はImplicit VR Little Endian（デフォルトの転送構文UID）を返す
    uid.unwrap_or(&IMPLICIT_VR_LITTLE_ENDIAN)
}

async fn reject_association(
    buf_reader: &mut BufReader<&mut TcpStream>,
    result: a_associate_rj::Result,
    source_and_reason: SourceAndReason,
) {
    match send_a_associate_rj(&mut buf_reader.get_mut(), result, source_and_reason).await {
        Ok(()) => debug!("A-ASSOCIATE-RJを送信しました"),
        Err(e) => error!("A-ASSOCIATE-RJの送信に失敗しました: {e}"),
    }
}

async fn handle_association_release(buf_reader: &mut BufReader<&mut TcpStream>) {
    let reception = match receive_a_release_rq(buf_reader).await {
        Ok(val) => val,
        Err(e) => {
            error!("A-RELEASE-RQの受信に失敗しました: {e}");
            if !matches!(e, PduReadError::IoError(_)) {
                abort(buf_reader, a_abort::Reason::from(e)).await;
            }
            return;
        }
    };
    if let AReleaseRqReception::AAbort(a_abort) = reception {
        debug!(
            "A-ABORTを受信しました: (Source={:02X} Reason={:02X})",
            a_abort.source() as u8,
            a_abort.reason() as u8
        );
        return;
    }
    debug!("A-RELEASE-RQを受信しました");

    release(buf_reader).await;
}

async fn release(buf_reader: &mut BufReader<&mut TcpStream>) {
    match send_a_release_rp(&mut buf_reader.get_mut()).await {
        Ok(()) => debug!("A-RELEASE-RPを送信しました"),
        Err(e) => error!("A-RELEASE-RPの送信に失敗しました: {e}"),
    }
}

async fn abort(buf_reader: &mut BufReader<&mut TcpStream>, reason: a_abort::Reason) {
    match send_a_abort(&mut buf_reader.get_mut(), Source::Provider, reason).await {
        Ok(()) => debug!("A-ABORTを送信しました"),
        Err(e) => error!("A-ABORTの送信に失敗しました: {e}"),
    }
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let mut sigterm =
            signal(SignalKind::terminate()).expect("SIGTERMハンドラの登録に失敗しました");
        let mut sigint =
            signal(SignalKind::interrupt()).expect("SIGINTハンドラの登録に失敗しました");

        tokio::select! {
            _ = sigterm.recv() => info!("SIGTERMを受信しました"),
            _ = sigint.recv() => info!("SIGINTを受信しました"),
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c()
            .await
            .expect("Ctrl+Cハンドラの登録に失敗しました");
        info!("Ctrl+Cを受信しました");
    }

    info!("サーバーを停止します");
}
