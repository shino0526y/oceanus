mod args;

use crate::args::Args;
use clap::Parser;
use dicom_lib::{
    constants::{sop_class_uids::VERIFICATION, transfer_syntax_uids::IMPLICIT_VR_LITTLE_ENDIAN},
    network::{
        dimse::c_echo::{
            c_echo_rq::CEchoRq,
            c_echo_rsp::{CEchoRsp, Status},
        },
        upper_layer_protocol::{
            pdu::{
                self, AReleaseRqReception, PDataTfReception,
                a_abort::{Reason, Source},
                a_associate_ac::{
                    ApplicationContext, PresentationContext, UserInformation,
                    presentation_context::{ResultReason, TransferSyntax},
                    user_information::{
                        ImplementationClassUid, ImplementationVersionName, MaximumLength,
                    },
                },
                receive_a_associate_rq, receive_a_release_rq, receive_p_data_tf,
                send_a_associate_ac, send_a_release_rp, send_p_data_tf,
            },
            utils::command_set_converter::{
                command_set_to_p_data_tf_pdus, p_data_tf_pdus_to_command_set,
            },
        },
    },
};
use std::{
    io::IsTerminal,
    net::Ipv4Addr,
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    },
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

const MAXIMUM_LENGTH: u32 = 0;

static CONNECTION_COUNTER: AtomicU64 = AtomicU64::new(1);
static SERVER_AE_TITLE: OnceLock<String> = OnceLock::new();

#[tokio::main]
async fn main() -> std::io::Result<()> {
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

    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, args.port)).await?;
    info!(
        "サーバーが起動しました (AEタイトル=\"{}\" ポート番号={})",
        SERVER_AE_TITLE.get().unwrap(),
        args.port
    );

    loop {
        let (socket, addr) = listener.accept().await?;
        let connection_id = CONNECTION_COUNTER.fetch_add(1, Ordering::Relaxed);

        tokio::spawn(async move {
            use Instrument;

            handle_connection(socket)
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

async fn handle_connection(mut socket: TcpStream) {
    let mut buf_reader = BufReader::new(&mut socket);

    // A-ASSOCIATE-RQの受信
    let a_associate_rq = match receive_a_associate_rq(&mut buf_reader).await {
        Ok(val) => val,
        Err(e) => {
            error!("A-ASSOCIATE-RQの受信に失敗しました: {e:?}");
            return;
        }
    };
    let called_ae_title = a_associate_rq.called_ae_title();
    let calling_ae_title = a_associate_rq.calling_ae_title();
    debug!(
        "A-ASSOCIATE-RQを受信しました (送信元=\"{calling_ae_title}\" 宛先=\"{called_ae_title}\")"
    );

    let presentation_contexts = a_associate_rq.presentation_contexts();
    let is_supported = presentation_contexts
        .iter()
        .any(|presentation_context| presentation_context.abstract_syntax().name() == VERIFICATION);

    // TODO: A_ASSOCIATE_RJを送信する
    if called_ae_title != SERVER_AE_TITLE.get().unwrap() {
        warn!(
            "アソシエーション要求を拒否しました (送信元=\"{calling_ae_title}\" 宛先=\"{called_ae_title}\" 理由=AEタイトル不一致)",
        );
        panic!("サーバーのAEタイトルとクライアントのAEタイトルが一致しません");
    } else if !is_supported {
        warn!(
            "アソシエーション要求を拒否しました (送信元=\"{calling_ae_title}\" 理由=サポートされていない抽象構文)",
        );
        panic!("サポートされていない抽象構文が指定されました");
    } else {
        info!("アソシエーション要求を受諾しました (送信元=\"{calling_ae_title}\")",);
    }

    // A-ASSOCIATE-ACの送信
    {
        let application_context = ApplicationContext::new("1.2.840.10008.3.1.1.1.1");
        let presentation_contexts = presentation_contexts
            .iter()
            .map(|presentation_context| {
                PresentationContext::new(
                    presentation_context.context_id(),
                    if presentation_context.abstract_syntax().name() == VERIFICATION {
                        ResultReason::Acceptance
                    } else {
                        ResultReason::AbstractSyntaxNotSupported
                    },
                    TransferSyntax::new(IMPLICIT_VR_LITTLE_ENDIAN).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        let user_information = UserInformation::new(
            Some(MaximumLength::new(MAXIMUM_LENGTH)),
            ImplementationClassUid::new(IMPLEMENTATION_CLASS_UID).unwrap(),
            Some(ImplementationVersionName::new(IMPLEMENTATION_VERSION_NAME).unwrap()),
        );

        match send_a_associate_ac(
            &mut buf_reader.get_mut(),
            called_ae_title,
            calling_ae_title,
            application_context,
            presentation_contexts,
            user_information,
        )
        .await
        {
            Ok(()) => {}
            Err(e) => {
                error!("A-ASSOCIATE-ACの送信に失敗しました: {e:?}");
                return;
            }
        };
    }
    debug!("A-ASSOCIATE-ACを送信しました");

    // P-DATA-TFの受信
    let p_data_tf = {
        let reception = match receive_p_data_tf(&mut buf_reader).await {
            Ok(val) => val,
            Err(e) => {
                error!("P-DATA-TFの受信に失敗しました: {e:?}");
                let reason = Reason::from(e);
                send_a_abort(&mut buf_reader, reason).await;
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
    debug!("P-DATA-TFを受信しました");

    // 受信したP-DATA-TFからコマンドセットを生成する
    let presentation_context_id = p_data_tf.presentation_data_values()[0].presentation_context_id();
    let command_set_received = match p_data_tf_pdus_to_command_set(&[p_data_tf]) {
        Ok(val) => val,
        Err(e) => {
            error!("コマンドセットのパースに失敗しました: {e}");
            let reason = Reason::InvalidPduParameterValue;
            send_a_abort(&mut buf_reader, reason).await;
            return;
        }
    };

    let c_echo_rq = match CEchoRq::try_from(command_set_received) {
        Ok(val) => val,
        Err(e) => {
            error!("C-ECHO-RQのパースに失敗しました: {e}");
            let reason = Reason::InvalidPduParameterValue;
            send_a_abort(&mut buf_reader, reason).await;
            return;
        }
    };
    debug!("  -> C-ECHO-RQ",);

    let c_echo_rsp = CEchoRsp::new(c_echo_rq.message_id(), Status::Success);
    debug!("  <- C-ECHO-RSP",);

    // 送信するP-DATA-TFのためのコマンドセットを生成する
    let command_set_to_be_sent = c_echo_rsp.into();

    // P-DATA-TFの送信
    {
        let maximum_length = a_associate_rq
            .user_information()
            .maximum_length()
            .map_or(0, |maximum_length| maximum_length.maximum_length());
        let p_data_tf_pdus = command_set_to_p_data_tf_pdus(
            &command_set_to_be_sent,
            presentation_context_id,
            maximum_length,
        );

        match send_p_data_tf(&mut buf_reader.get_mut(), &p_data_tf_pdus).await {
            Ok(()) => {}
            Err(e) => {
                error!("P-DATA-TFの送信に失敗しました: {e:?}");
                return;
            }
        }
        debug!("P-DATA-TFを送信しました");
    }

    // A-RELEASE-RQの受信
    {
        let reception = match receive_a_release_rq(&mut buf_reader).await {
            Ok(val) => val,
            Err(e) => {
                error!("A-RELEASE-RQの受信に失敗しました: {e:?}");
                let reason = Reason::from(e);
                send_a_abort(&mut buf_reader, reason).await;
                return;
            }
        };

        match reception {
            AReleaseRqReception::AReleaseRq(val) => val,
            AReleaseRqReception::AAbort(a_abort) => {
                debug!(
                    "A-ABORTを受信しました: (Source={:02X} Reason={:02X})",
                    a_abort.source() as u8,
                    a_abort.reason() as u8
                );
                return;
            }
        }
    };
    debug!("A-RELEASE-RQを受信しました");

    // A-RELEASE-RPの送信
    match send_a_release_rp(buf_reader.get_mut()).await {
        Ok(()) => {}
        Err(e) => {
            error!("A-RELEASE-RPの送信に失敗しました: {e:?}");
            return;
        }
    }
    debug!("A-RELEASE-RPを送信しました");

    info!("Verificationサービスを正常に完了しました");
}

async fn send_a_abort(buf_reader: &mut BufReader<&mut TcpStream>, reason: Reason) {
    match pdu::send_a_abort(&mut buf_reader.get_mut(), Source::Provider, reason).await {
        Ok(()) => debug!("A-ABORTを送信しました"),
        Err(e) => error!("A-ABORTの送信に失敗しました: {e:?}"),
    }
}
