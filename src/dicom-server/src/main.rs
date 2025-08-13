use dicom_server::dicom::network::{
    CommandSet,
    dimse::c_echo::{
        c_echo_rq::CEchoRq,
        c_echo_rsp::{CEchoRsp, Status},
    },
    upper_layer_protocol::{
        pdu::{
            AAbort, AAssociateAc, AAssociateRq, AReleaseRp, AReleaseRq, PDataTf, PduType, a_abort,
            a_associate_ac::{
                ApplicationContext, PresentationContext, UserInformation,
                presentation_context::{ResultReason, TransferSyntax},
                user_information::{
                    ImplementationClassUid, ImplementationVersionName, MaximumLength,
                },
            },
        },
        utils::command_set_converter::{
            command_set_to_p_data_tf_pdus, p_data_tf_pdus_to_command_set,
        },
    },
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

const SERVER_AE_TITLE: &str = "SERVER";
const IMPLEMENTATION_CLASS_UID: &str = "1.2.826.0.1.3680043.2.1396.999";
const IMPLEMENTATION_VERSION_NAME: &str = "Oceanus";
const MAXIMUM_LENGTH: u32 = 0;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:104").await?;
    println!("104 ポートをリッスンしています...");

    let (mut socket, addr) = listener.accept().await?;
    println!(
        "コネクションを受け入れました ({}:{})",
        addr.ip(),
        addr.port()
    );

    let a_associate_rq = {
        let mut buf_reader = tokio::io::BufReader::new(&mut socket);

        let pdu_type = PduType::try_from(buf_reader.read_u8().await?).unwrap_or_else(|e| {
            panic!("PDU タイプの変換に失敗しました: {e}");
        });
        if pdu_type != PduType::AAssociateRq {
            buf_reader.into_inner().shutdown().await?;
            panic!("A-ASSOCIATE-RQ 以外の PDU が受信されました");
        }
        buf_reader.read_u8().await?; // Reserved
        let pdu_length = buf_reader.read_u32().await?;

        match AAssociateRq::read_from_stream(&mut buf_reader, pdu_length).await {
            Ok(req) => req,
            Err(e) => {
                panic!("A-ASSOCIATE-RQ PDU のパースに失敗しました: {e:?}");
            }
        }
    };

    let called_ae_title = a_associate_rq.called_ae_title();
    let calling_ae_title = a_associate_rq.calling_ae_title();

    println!("Calling AE Title: {calling_ae_title}");
    println!("Called AE Title: {called_ae_title}");
    if called_ae_title != SERVER_AE_TITLE {
        // TODO: A_ASSOCIATE_RJを送信する
        panic!("サーバーの AE タイトルとクライアントの AE タイトルが一致しません");
    }

    let application_context = ApplicationContext::new("1.2.840.10008.3.1.1.1.1");
    let presentation_contexts = a_associate_rq
        .presentation_contexts()
        .iter()
        .map(|presentation_context| {
            PresentationContext::new(
                presentation_context.context_id(),
                if presentation_context.abstract_syntax().name() == "1.2.840.10008.1.1" {
                    ResultReason::Acceptance
                } else {
                    ResultReason::AbstractSyntaxNotSupported
                },
                TransferSyntax::new("1.2.840.10008.1.2").unwrap(),
            )
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

    println!("A-ASSOCIATE-AC PDU を送信します");
    {
        let bytes: Vec<u8> = a_associate_ac.into();
        socket.write_all(&bytes).await?;
    }

    let p_data_tf = {
        let mut buf_reader = tokio::io::BufReader::new(&mut socket);

        let pdu_type = PduType::try_from(buf_reader.read_u8().await?).unwrap_or_else(|e| {
            panic!("PDU タイプの変換に失敗しました: {e}");
        });
        match pdu_type {
            PduType::PDataTf => {
                buf_reader.read_u8().await?; // Reserved
                let pdu_length = buf_reader.read_u32().await?;

                PDataTf::read_from_stream(&mut buf_reader, pdu_length)
                    .await
                    .unwrap_or_else(|e| {
                        panic!("P-DATA-TF PDU のパースに失敗しました: {e}");
                    })
            }
            PduType::AAbort => {
                handle_abort(buf_reader).await?;
                panic!("A-ABORT PDU が受信されました");
            }
            _ => {
                abort(&mut socket, a_abort::Reason::UnexpectedPdu).await?;
                panic!("P-DATA-TF 以外の PDU が受信されました");
            }
        }
    };
    println!("P-DATA-TF PDU を受信しました");

    let presentation_context_id = p_data_tf.presentation_data_values()[0].presentation_context_id();
    let c_echo_rq = {
        let command_set = p_data_tf_pdus_to_command_set(&[p_data_tf]).unwrap_or_else(|e| {
            panic!("コマンドセットのパースに失敗しました: {e}");
        });

        CEchoRq::try_from(command_set).unwrap_or_else(|e| {
            panic!("C-ECHO-RQ のパースに失敗しました: {e}");
        })
    };

    // TODO: エラー処理
    let c_echo_rsp = CEchoRsp::new(c_echo_rq.message_id(), Status::Success);
    println!("C-ECHO-RSP を送信します");
    {
        let maximum_length = a_associate_rq
            .user_information()
            .maximum_length()
            .map_or(0, |maximum_length| maximum_length.maximum_length());

        let command_set: CommandSet = c_echo_rsp.into();
        let p_data_tf_pdus =
            command_set_to_p_data_tf_pdus(&command_set, presentation_context_id, maximum_length);
        for p_data_tf in p_data_tf_pdus {
            let bytes: Vec<u8> = (&p_data_tf).into();
            socket.write_all(&bytes).await?;
        }
    }

    // A-RELEASE-RQ PDUの受信
    {
        let mut buf_reader = tokio::io::BufReader::new(&mut socket);

        let pdu_type = PduType::try_from(buf_reader.read_u8().await?).unwrap_or_else(|e| {
            panic!("PDU タイプの変換に失敗しました: {e}");
        });
        match pdu_type {
            PduType::AReleaseRq => {
                buf_reader.read_u8().await?; // Reserved
                let pdu_length = buf_reader.read_u32().await?;

                match AReleaseRq::read_from_stream(&mut buf_reader, pdu_length).await {
                    Ok(req) => req,
                    Err(e) => {
                        panic!("A-RELEASE-RQ PDU のパースに失敗しました: {e:?}");
                    }
                }
            }
            PduType::AAbort => {
                handle_abort(buf_reader).await?;
                panic!("A-ABORT PDU が受信されました");
            }
            _ => {
                abort(&mut socket, a_abort::Reason::UnexpectedPdu).await?;
                panic!("A-RELEASE-RQ 以外の PDU が受信されました");
            }
        }
    };

    let a_release_rp = AReleaseRp::new();
    println!("A-RELEASE-RP を送信します");
    {
        let bytes: Vec<u8> = a_release_rp.into();
        socket.write_all(&bytes).await?;
    }

    socket.shutdown().await?;

    Ok(())
}

/// A-ABORT PDU を受信し、処理する
///
/// 具体的には以下を行う。
/// - ログの出力
/// - 通信の切断
async fn handle_abort(mut buf_reader: tokio::io::BufReader<&mut TcpStream>) -> std::io::Result<()> {
    buf_reader.read_u8().await?; // Reserved
    let pdu_length = buf_reader.read_u32().await?;

    match AAbort::read_from_stream(&mut buf_reader, pdu_length).await {
        Ok(a_abort) => {
            eprintln!(
                "A-ABORT PDU を受信しました (source={:02X}, reason={:02X})",
                a_abort.source() as u8,
                a_abort.reason() as u8
            );
        }
        Err(e) => {
            eprintln!("A-ABORT PDU のパースに失敗しました: {e}");
        }
    };

    buf_reader.into_inner().shutdown().await?;

    Ok(())
}

/// A-ABORT PDU を送信し、通信を切断する
async fn abort(socket: &mut tokio::net::TcpStream, reason: a_abort::Reason) -> std::io::Result<()> {
    let a_abort = AAbort::new(a_abort::Source::Provider, reason);

    let bytes: Vec<u8> = a_abort.into();
    socket.write_all(&bytes).await?;
    socket.shutdown().await?;

    Ok(())
}
