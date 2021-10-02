use crate::common::Endpoint;
use crate::filter::{Filter, FilterCmd, FilterCmdSend, FilterMode, FilterNotice, FilterNotifyRecv, FilterRspRecv};
use crate::protocol::{BroadcastChatMessage, Packet, RequestAction, ResponseCode};
use crate::settings::TRANSPORT_CHANNEL_LEN;
use crate::transport::{TransportCmd, TransportCmdRecv, TransportNotice, TransportNotifySend, TransportRsp};
use lazy_static::lazy_static;
use snowflake::ProcessUniqueId;
use std::future::Future;
use std::net::ToSocketAddrs;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{self, timeout_at, Instant};

lazy_static! {
    static ref CLIENT_ENDPOINT: Endpoint = Endpoint(("1.2.3.4", 5678).to_socket_addrs().unwrap().next().unwrap());
}

#[tokio::test]
async fn time_advancing_works() {
    time::pause();
    let start = Instant::now();
    time::advance(Duration::from_secs(5)).await;
    let end = Instant::now();
    assert_eq!(end - start, Duration::from_secs(5));
}

#[tokio::test]
async fn basic_server_filter_flow() {
    let (
        transport_notice_tx,
        mut transport_cmd_rx,
        filter_cmd_tx,
        _filter_rsp_rx,
        _filter_notify_rx,
        logged_in_resp_pkt_tid,
        filter_shutdown_watcher,
    ) = setup_server().await;

    // The logged_in_resp_pkt_tid is the transport ID of the LoggedIn packet sent by the server's
    // filter layer down to the transport layer, for sending to the client. Once the server's
    // filter layer receives acknowledgement from the client, the transport layer should receive a
    // DropPacket transport command with that transport ID.
    let action = RequestAction::KeepAlive {
        latest_response_ack: 1, // Must match the sequence sent in last Response server sent to this client (LoggedIn)
    };
    let packet = Packet::Request {
        sequence: 2,
        response_ack: Some(1), // Must match the sequence sent in last Response server sent to this client (LoggedIn)
        cookie: Some("fakecookie".to_owned()),
        action,
    };
    transport_notice_tx
        .send(TransportNotice::PacketDelivery {
            endpoint: *CLIENT_ENDPOINT,
            packet,
        })
        .await
        .unwrap();

    // Check for DropPacket (this should happen before response comes down from app layer)
    let expiration = Instant::now() + Duration::from_secs(3);
    time::advance(Duration::from_secs(5)).await;
    let transport_cmd = timeout_at(expiration, transport_cmd_rx.recv())
        .await
        .expect("we should not have timed out getting a transport cmd from filter layer");
    let transport_cmd = transport_cmd.expect("should have gotten a TransportCmd from Filter");
    match transport_cmd {
        TransportCmd::DropPacket { endpoint, tid } => {
            assert_eq!(endpoint, *CLIENT_ENDPOINT);
            assert_eq!(tid, logged_in_resp_pkt_tid);
        }
        _ => panic!("unexpected transport command {:?}", transport_cmd),
    }

    // Shut down
    filter_cmd_tx
        .send(FilterCmd::Shutdown { graceful: false })
        .await
        .unwrap();
    filter_shutdown_watcher.await;
}

// TODO: basic_client_filter_flow

/// This is a helper to simplify setting up the filter layer in server mode with one client connection. Call like this:
///
/// ```rust
/// let (transport_notice_tx, transport_cmd_rx, filter_cmd_tx, filter_rsp_rx, filter_notify_rx, logged_in_resp_pkt_tid, filter_shutdown_watcher) = setup_server().await;
/// ```
async fn setup_server() -> (
    TransportNotifySend,
    TransportCmdRecv,
    FilterCmdSend,
    FilterRspRecv,
    FilterNotifyRecv,
    ProcessUniqueId,
    impl Future<Output = ()> + 'static,
) {
    time::pause();
    // Mock transport channels
    let (transport_cmd_tx, mut transport_cmd_rx) = mpsc::channel(TRANSPORT_CHANNEL_LEN);
    let (transport_rsp_tx, transport_rsp_rx) = mpsc::channel(TRANSPORT_CHANNEL_LEN);
    let (transport_notice_tx, transport_notice_rx) = mpsc::channel(TRANSPORT_CHANNEL_LEN);

    let (mut filter, filter_cmd_tx, filter_rsp_rx, mut filter_notify_rx) = Filter::new(
        transport_cmd_tx,
        transport_rsp_rx,
        transport_notice_rx,
        FilterMode::Server,
    );

    let filter_shutdown_watcher = filter.get_shutdown_watcher(); // No await; get the future

    // Start the filter's task in the background
    tokio::spawn(async move { filter.run().await });

    // Send a mock transport notification
    let request_action_from_client = RequestAction::Connect {
        name:           "Sheeana".to_owned(),
        client_version: "0.3.2".to_owned(),
    };
    let sequence_from_client = 1;
    let packet_from_client = Packet::Request {
        sequence:     sequence_from_client,
        response_ack: None,
        cookie:       None,
        action:       request_action_from_client.clone(),
    };
    transport_notice_tx
        .send(TransportNotice::PacketDelivery {
            endpoint: *CLIENT_ENDPOINT,
            packet:   packet_from_client,
        })
        .await
        .unwrap();

    let expiration = Instant::now() + Duration::from_secs(3);
    time::advance(Duration::from_secs(5)).await; // TODO: once we add a test for a timing out flow, we can move this to that test and the timeout_at below

    // Check that we got a filter notification
    let timeout_result = timeout_at(expiration, filter_notify_rx.recv()).await;
    let filter_notification = timeout_result.expect("we should not have timed out getting a filter notification");

    let filter_notification = filter_notification.expect("channel should not have been closed");

    // Check that the correct notification was passed up to the app layer
    match filter_notification {
        FilterNotice::NewRequestAction {
            endpoint: _endpoint,
            action: _request_action,
        } => {
            assert_eq!(*CLIENT_ENDPOINT, _endpoint);
            assert_eq!(request_action_from_client, _request_action);
        }
        _ => panic!("Unexpected filter notification: {:?}", filter_notification),
    };

    // Send a logged in message from App layer to the Filter layer we are testing here
    let resp_code_for_client = ResponseCode::LoggedIn {
        cookie:         "fakecookie".to_owned(),
        server_version: "1.2.3.4.5".to_owned(),
    };
    filter_cmd_tx
        .send(FilterCmd::SendResponseCode {
            endpoint: *CLIENT_ENDPOINT,
            code:     resp_code_for_client.clone(),
        })
        .await
        .expect("should successfully send a command from App layer down to Filter layer");

    // Check that the LoggedIn response code was sent down to the Transport layer
    let transport_cmd = transport_cmd_rx
        .recv()
        .await
        .expect("should have gotten a TransportCmd from Filter");
    let packet_to_client;
    let logged_in_resp_pkt_tid;
    match transport_cmd {
        TransportCmd::SendPackets {
            endpoint: _endpoint,
            packets,
            packet_infos,
        } => {
            assert_eq!(*CLIENT_ENDPOINT, _endpoint);
            // No need to test packet_infos
            packet_to_client = packets
                .into_iter()
                .next()
                .expect("expected at least one packet for Transport layer");
            assert_eq!(
                packet_infos.len(),
                1,
                "multiple packets sent when one was expected for LoggedIn response"
            );
            logged_in_resp_pkt_tid = packet_infos[0].tid;
        }
        _ => panic!("unexpected TransportCmd"),
    };
    match packet_to_client {
        Packet::Response {
            sequence,
            request_ack,
            code,
        } => {
            assert_eq!(sequence, 1);
            assert_eq!(request_ack, Some(sequence_from_client));
            assert_eq!(code, resp_code_for_client);
        }
        _ => panic!("expected a Packet::Response, got {:?}", packet_to_client),
    }
    (
        transport_notice_tx,
        transport_cmd_rx,
        filter_cmd_tx,
        filter_rsp_rx,
        filter_notify_rx,
        logged_in_resp_pkt_tid,
        filter_shutdown_watcher,
    )
}
