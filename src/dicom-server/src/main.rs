mod args;

use crate::args::Args;
use clap::Parser;
use dicom_lib::{
    constants::{sop_class_uids::VERIFICATION, transfer_syntax_uids::IMPLICIT_VR_LITTLE_ENDIAN},
    network::{
        command_set::utils::{command_set_to_p_data_tf_pdus, p_data_tf_pdus_to_command_set},
        dimse::c_echo::{
            c_echo_rq::CEchoRq,
            c_echo_rsp::{CEchoRsp, Status},
        },
        upper_layer_protocol::{
            AReleaseRqReception, PDataTfReception,
            pdu::{
                AAssociateAc, AAssociateRq, PduReadError,
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
            },
            receive_a_associate_rq, receive_a_release_rq, receive_p_data_tf, send_a_abort,
            send_a_associate_ac, send_a_associate_rj, send_a_release_rp, send_p_data_tf,
        },
    },
};
use dotenvy::dotenv;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::{
    io::IsTerminal,
    net::Ipv4Addr,
    process::exit,
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::BufReader,
    net::{TcpListener, TcpStream},
};
use tracing::{Instrument, Level, debug, error, info, level_filters::LevelFilter, span, warn};
use tracing_subscriber::fmt::time::LocalTime;

// <root>.<app>.<type>.<version>
// root: 1.3.6.1.4.1.64183 (https://www.iana.org/assignments/enterprise-numbers/)
// app: 1 (Oceanus)
// type: 1 (DICOM Server)
// version: x (major version)
const IMPLEMENTATION_CLASS_UID: &str =
    concat!("1.3.6.1.4.1.64183.1.1.", env!("CARGO_PKG_VERSION_MAJOR"));
const IMPLEMENTATION_VERSION_NAME: &str = concat!("OCEANUS_", env!("CARGO_PKG_VERSION")); // OCEANUS_x.y.z

const MAXIMUM_LENGTH: u32 = 0; // 制限なし
const SUPPORTED_ABSTRACT_SYNTAX_UIDS: &[&str] = &[VERIFICATION];
const SUPPORTED_TRANSFER_SYNTAX_UIDS: &[&str] = &[IMPLICIT_VR_LITTLE_ENDIAN];

static CONNECTION_COUNTER: AtomicU64 = AtomicU64::new(1);
static SERVER_AE_TITLE: OnceLock<String> = OnceLock::new();
static DB_POOL: OnceLock<Pool<Postgres>> = OnceLock::new();

#[tokio::main]
async fn main() {
    // 環境変数の読み込み（失敗しても無視）
    let _ = dotenv();
    // コマンドライン引数の解析
    let args = Args::parse();
    SERVER_AE_TITLE.set(args.ae_title).unwrap();

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

    loop {
        let (socket, addr) = {
            match listener.accept().await {
                Ok(val) => val,
                Err(e) => {
                    error!("接続の受け入れに失敗しました: {e}");
                    continue;
                }
            }
        };
        let connection_id = CONNECTION_COUNTER.fetch_add(1, Ordering::Relaxed);

        tokio::spawn(async move {
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

    let (a_associate_rq, accepted_presentation_contexts) =
        match handle_association_establishment(&mut buf_reader).await {
            Some(val) => val,
            None => return,
        };
    // TODO: 複数のPresentation Contextに対応する
    if accepted_presentation_contexts.len() > 1 {
        panic!("複数のPresentation Contextに対応していません");
    }

    // P-DATA-TFの受信
    let mut p_data_tfs = vec![];
    loop {
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

        if p_data_tf.presentation_data_values().iter().any(|pdv| {
            accepted_presentation_contexts
                .iter()
                .all(|pc| pdv.presentation_context_id() != pc.context_id())
        }) {
            warn!(
                "受信したP-DATA-TFのPresentation Context IDがアソシエーションで受諾したものに含まれていません"
            );
            abort(&mut buf_reader, a_abort::Reason::InvalidPduParameterValue).await;
            return;
        }

        let is_last = p_data_tf
            .presentation_data_values()
            .iter()
            .any(|pdv| pdv.is_last());

        p_data_tfs.push(p_data_tf);

        if is_last {
            break;
        }
    }
    debug!("P-DATA-TFを受信しました");

    let command_set_received = match p_data_tf_pdus_to_command_set(&p_data_tfs) {
        Ok(val) => val,
        Err(e) => {
            error!("コマンドセットのパースに失敗しました: {e}");
            abort(&mut buf_reader, a_abort::Reason::InvalidPduParameterValue).await;
            return;
        }
    };

    let c_echo_rq = match CEchoRq::try_from(command_set_received) {
        Ok(val) => val,
        Err(e) => {
            error!("C-ECHO-RQのパースに失敗しました: {e}");
            abort(&mut buf_reader, a_abort::Reason::InvalidPduParameterValue).await;
            return;
        }
    };

    info!(
        "[{}] Verification (MessageID={})",
        accepted_presentation_contexts[0].context_id(), // TODO: 複数のPresentation Contextに対応する
        c_echo_rq.message_id()
    );

    let c_echo_rsp = CEchoRsp::new(c_echo_rq.message_id(), Status::Success);

    let command_set_to_be_sent = c_echo_rsp.into();

    // P-DATA-TFの送信
    {
        let maximum_length = a_associate_rq
            .user_information()
            .maximum_length()
            .map_or(0, |maximum_length| maximum_length.maximum_length());
        let p_data_tf_pdus = command_set_to_p_data_tf_pdus(
            command_set_to_be_sent,
            accepted_presentation_contexts[0].context_id(), // TODO: 複数のPresentation Contextに対応する
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

    handle_association_release(&mut buf_reader).await;

    info!("アソシエーションを正常に終了しました");
}

async fn handle_association_establishment(
    buf_reader: &mut BufReader<&mut TcpStream>,
) -> Option<(AAssociateRq, Vec<a_associate_ac::PresentationContext>)> {
    // A-ASSOCIATE-RQの受信
    let a_associate_rq = match receive_a_associate_rq(buf_reader).await {
        Ok(val) => val,
        Err(e) => {
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

    match sqlx::query!(
        "SELECT host FROM application_entities WHERE title = $1",
        calling_ae_title
    )
    .fetch_one(DB_POOL.get().unwrap())
    .await
    {
        Ok(application_entity) => {
            let mut host_addresses = {
                match tokio::net::lookup_host((application_entity.host.as_str(), 0)).await {
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
    let mut accepted_presentation_contexts = vec![];
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
                    let pc = a_associate_ac::PresentationContext::new(
                        presentation_context.context_id(),
                        ResultReason::Acceptance,
                        TransferSyntax::new(IMPLICIT_VR_LITTLE_ENDIAN).unwrap(),
                    );
                    accepted_presentation_contexts.push(pc.clone());
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

    if accepted_presentation_contexts.len() > 0 {
        info!("アソシエーション要求を受諾しました (呼出元=\"{calling_ae_title}\")");
    } else {
        warn!(
            "アソシエーション要求について受諾可能なプレゼンテーションコンテキストがありません (呼出元=\"{calling_ae_title}\")"
        );
        return None;
    }

    Some((a_associate_rq, accepted_presentation_contexts))
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

    if let Err(e) = send_a_release_rp(buf_reader.get_mut()).await {
        error!("A-RELEASE-RPの送信に失敗しました: {e}");
        return;
    }
    debug!("A-RELEASE-RPを送信しました");
}

async fn abort(buf_reader: &mut BufReader<&mut TcpStream>, reason: a_abort::Reason) {
    match send_a_abort(&mut buf_reader.get_mut(), Source::Provider, reason).await {
        Ok(()) => debug!("A-ABORTを送信しました"),
        Err(e) => error!("A-ABORTの送信に失敗しました: {e}"),
    }
}
