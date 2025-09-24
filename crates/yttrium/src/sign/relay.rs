use {
    crate::{
        sign::{
            client_errors::RequestError,
            incoming::HandleError,
            priority_future::PriorityReceiver,
            protocol_types::{
                SessionRequestJsonRpc, SessionRequestJsonRpcResponse,
            },
            relay_url::ConnectionOptions,
            storage::Storage,
        },
        time::DurableSleep,
    },
    relay_rpc::{
        auth::ed25519_dalek::{Signer, SigningKey},
        domain::{DecodedClientId, MessageId, ProjectId, Topic},
        jwt::{JwtBasicClaims, JwtHeader},
        rpc::{
            BatchSubscribe, Params, Payload, Request, Response, Subscription,
            SuccessfulResponse,
        },
    },
    std::{sync::Arc, time::Duration},
};
#[cfg(not(target_arch = "wasm32"))]
use {
    futures::{SinkExt, StreamExt},
    tokio_tungstenite::{connect_async, tungstenite::Message},
};

const MIN_RPC_ID: u64 = 1000000000; // MessageId::MIN is private
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, PartialEq)]
enum IncomingMessage {
    Close(CloseReason),
    Message(Payload),
}

#[derive(Debug, PartialEq)]
enum CloseReason {
    InvalidAuth,
    Error(String),
}

#[cfg(target_arch = "wasm32")]
type ConnectWebSocket = (web_sys::WebSocket, Closures);
#[cfg(target_arch = "wasm32")]
struct Closures {
    #[allow(dead_code)]
    on_close: wasm_bindgen::closure::Closure<dyn Fn(web_sys::CloseEvent)>,
    #[allow(dead_code)]
    on_error: wasm_bindgen::closure::Closure<dyn Fn(web_sys::Event)>,
    #[allow(dead_code)]
    on_message: wasm_bindgen::closure::Closure<dyn Fn(web_sys::MessageEvent)>,
}

#[cfg(not(target_arch = "wasm32"))]
type ConnectWebSocket = (
    tokio::sync::mpsc::UnboundedSender<String>,
    tokio::sync::mpsc::UnboundedSender<()>,
);

#[derive(Debug, Clone)]
enum ConnectError {
    /// Network, timeout, or other temporary connection errors
    ConnectFail(String),
    /// Connect aborted because of a request to cleanup
    Cleanup,
    /// Connect aborted because of invalid auth
    InvalidAuth,
    /// An error that shouldn't happen (e.g. JSON serializing constant values)
    ShouldNeverHappen(String),
}

impl From<ConnectError> for RequestError {
    fn from(error: ConnectError) -> Self {
        match error {
            ConnectError::ConnectFail(e) => RequestError::Internal(e),
            ConnectError::Cleanup => RequestError::Cleanup,
            ConnectError::InvalidAuth => RequestError::InvalidAuth,
            ConnectError::ShouldNeverHappen(e) => {
                RequestError::ShouldNeverHappen(e)
            }
        }
    }
}

impl From<CloseReason> for ConnectError {
    fn from(reason: CloseReason) -> Self {
        match reason {
            CloseReason::InvalidAuth => ConnectError::InvalidAuth,
            CloseReason::Error(reason) => ConnectError::ConnectFail(reason),
        }
    }
}

async fn connect(
    relay_url: String,
    project_id: ProjectId,
    key: &SigningKey,
    topics: Vec<Topic>,
    initial_req: Params,
    cleanup_rx: tokio_util::sync::CancellationToken,
) -> Result<
    (
        u64,
        tokio::sync::mpsc::UnboundedReceiver<IncomingMessage>,
        ConnectWebSocket,
    ),
    ConnectError,
> {
    let url = {
        let encoder = &data_encoding::BASE64URL_NOPAD;
        let claims = {
            let data = JwtBasicClaims {
                iss: DecodedClientId::from_key(&key.verifying_key()).into(),
                sub: "http://example.com".to_owned(),
                aud: relay_url.clone(),
                iat: crate::time::SystemTime::now()
                    .duration_since(crate::time::UNIX_EPOCH)
                    .map_err(|e| {
                        ConnectError::ShouldNeverHappen(e.to_string())
                    })?
                    .as_secs() as i64,
                exp: Some(
                    crate::time::SystemTime::now()
                        .duration_since(crate::time::UNIX_EPOCH)
                        .map_err(|e| {
                            ConnectError::ShouldNeverHappen(e.to_string())
                        })?
                        .as_secs() as i64
                        + 60 * 60,
                ),
            };

            encoder.encode(
                serde_json::to_string(&data)
                    .map_err(|e| {
                        ConnectError::ShouldNeverHappen(e.to_string())
                    })?
                    .as_bytes(),
            )
        };
        let header = encoder.encode(
            serde_json::to_string(&JwtHeader::default())
                .map_err(|e| ConnectError::ShouldNeverHappen(e.to_string()))?
                .as_bytes(),
        );
        let message = format!("{header}.{claims}");
        let signature = {
            let data = key.sign(message.as_bytes());
            encoder.encode(&data.to_bytes())
        };
        let auth = format!("{message}.{signature}");

        let conn_opts =
            ConnectionOptions::new(project_id, auth).with_address(&relay_url);
        conn_opts
            .as_url()
            .map_err(|e| ConnectError::ConnectFail(e.to_string()))?
            .to_string()
    };

    #[cfg(not(target_arch = "wasm32"))]
    {
        let connect_fut = connect_async(url);
        let (mut ws_stream, _response) = tokio::select! {
            res = connect_fut => res.map_err(|e| ConnectError::ConnectFail(e.to_string()))?,
            _ = cleanup_rx.cancelled() => return Err(ConnectError::Cleanup),
            _ = crate::time::sleep(REQUEST_TIMEOUT) => {
                return Err(ConnectError::ConnectFail("Timeout connecting to relay".to_string()));
            }
        };

        let (outgoing_tx, mut outgoing_rx) =
            tokio::sync::mpsc::unbounded_channel::<String>();
        let (close_tx, mut close_rx) =
            tokio::sync::mpsc::unbounded_channel::<()>();
        let (on_incomingmessage_tx, mut on_incomingmessage_rx) =
            tokio::sync::mpsc::unbounded_channel();

        crate::spawn::spawn(async move {
            loop {
                use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;

                tokio::select! {
                    Some(message) = outgoing_rx.recv() => {
                        if let Err(e) = ws_stream.send(Message::Text(message.into())).await {
                            tracing::debug!("Failed to send outgoing message: {e}");
                            break;
                        }
                    }
                    _ = close_rx.recv() => {
                        if let Err(e) = ws_stream.close(None).await {
                            tracing::debug!("Failed to close WebSocket (for cleanup): {e}");
                        }
                        tracing::debug!("tungstenite connection loop ending (cleanup)");
                        break;
                    }
                    Some(message) = ws_stream.next() => {
                        let n = message
                            .map_err(|e| {
                                RequestError::Internal(format!(
                                    "WebSocket stream error: {e}"
                                ))
                            })
                            .expect("WebSocket stream error");
                        #[allow(clippy::single_match)]
                        match n {
                            Message::Text(message) => {
                                let result = serde_json::from_str::<Payload>(&message);
                                match result {
                                    Ok(payload) => {
                                        if let Err(e) = on_incomingmessage_tx.send(IncomingMessage::Message(payload)) {
                                            tracing::debug!("Failed to send incoming message: {e}");
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!("Failed to parse payload: {e}");
                                    }
                                }

                            }
                            Message::Close(close_event) => {
                                tracing::debug!("websocket onclose: {:?}", close_event);
                                if let Some(close_event) = close_event {
                                    if close_event.code == CloseCode::Iana(3000) {
                                        tracing::error!("Invalid auth: {}", close_event.reason);
                                        if let Err(e) = on_incomingmessage_tx.send(IncomingMessage::Close(CloseReason::InvalidAuth)) {
                                            tracing::debug!("Failed to send invalid auth close event: {e}");
                                        }
                                    } else {
                                        #[allow(clippy::collapsible_else_if)]
                                        if let Err(e) = on_incomingmessage_tx.send(IncomingMessage::Close(CloseReason::Error(format!("{}: {}", close_event.code, close_event.reason)))) {
                                            tracing::debug!("Failed to send close event (error): {e}");
                                        }
                                    }
                                } else {
                                    #[allow(clippy::collapsible_else_if)]
                                    if let Err(e) = on_incomingmessage_tx.send(IncomingMessage::Close(CloseReason::Error("Unknown close event".to_string()))) {
                                        tracing::debug!("Failed to send close event (unknown): {e}");
                                    }
                                }
                            }
                            e => tracing::debug!("ignoring tungstenite message: {:?}", e),
                        }
                    }
                    else => {
                        tracing::debug!("tungstenite connection loop ending (else)");
                        break;
                    }
                }
            }
            tracing::debug!("tungstenite connection loop ending");
        });

        let mut message_id = MIN_RPC_ID;

        if !topics.is_empty() {
            // TODO batch this extra request together with the initial connection

            let payload_request = Payload::Request(Request::new(
                MessageId::new(message_id),
                Params::BatchSubscribe(BatchSubscribe { topics }),
            ));
            message_id += 1;
            let serialized = serde_json::to_string(&payload_request)
                .map_err(|e| ConnectError::ShouldNeverHappen(e.to_string()))?;
            outgoing_tx
                .send(serialized)
                .map_err(|e| ConnectError::ConnectFail(e.to_string()))?;

            let mut durable_sleep = crate::time::durable_sleep(REQUEST_TIMEOUT);

            loop {
                let incoming_message = tokio::select! {
                    Some(message) = on_incomingmessage_rx.recv() => message,
                    _ = cleanup_rx.cancelled() => {
                        if let Err(e) = close_tx.send(()) {
                            tracing::debug!("Failed to close WebSocket (for cleanup): {e}");
                        }
                        return Err(ConnectError::Cleanup);
                    },
                    _ = durable_sleep.recv() => {
                        if let Err(e) = close_tx.send(()) {
                            tracing::debug!("Failed to close WebSocket (for timeout): {e}");
                        }
                        return Err(ConnectError::ConnectFail("Timeout waiting for batch subscribe response".to_string()));
                    }
                };

                match incoming_message {
                    IncomingMessage::Close(reason) => return Err(reason.into()),
                    IncomingMessage::Message(payload) => {
                        let id = payload.id();
                        match payload {
                            Payload::Request(request) => {
                                tracing::warn!("unexpected message request in connect() function: {:?}", request);
                            }
                            Payload::Response(response) => {
                                if id == MessageId::new(message_id) {
                                    // success, no-op
                                    break;
                                } else {
                                    tracing::warn!("unexpected message response in connect() function: {:?}", response);
                                }
                            }
                        }
                    }
                }
            }
        }

        // TODO this will soon be moved to initial WebSocket request
        let request = Payload::Request(Request::new(
            MessageId::new(message_id),
            initial_req,
        ));
        let serialized = serde_json::to_string(&request)
            .map_err(|e| ConnectError::ShouldNeverHappen(e.to_string()))?;
        outgoing_tx
            .send(serialized)
            .map_err(|e| ConnectError::ConnectFail(e.to_string()))?;

        Ok((message_id, on_incomingmessage_rx, (outgoing_tx, close_tx)))
    }

    #[cfg(target_arch = "wasm32")]
    {
        use {
            wasm_bindgen::{prelude::Closure, JsCast},
            web_sys::{CloseEvent, Event, MessageEvent},
        };

        let ws = web_sys::WebSocket::new(&url).map_err(|e| {
            ConnectError::ShouldNeverHappen(format!(
                "Failed to create WebSocket: {e:?}"
            ))
        })?;

        let (on_incomingmessage_tx, mut on_incomingmessage_rx) =
            tokio::sync::mpsc::unbounded_channel();

        let on_close_closure = Closure::wrap(Box::new({
            let on_incomingmessage_tx = on_incomingmessage_tx.clone();
            move |event: CloseEvent| {
                tracing::debug!("websocket onclose: {:?}", event);

                if event.code() == 3000 {
                    tracing::error!("Invalid auth: {}", event.reason());
                    if let Err(e) = on_incomingmessage_tx
                        .send(IncomingMessage::Close(CloseReason::InvalidAuth))
                    {
                        tracing::debug!(
                            "OK: Failed to send invalid auth close event: {e}"
                        );
                    }
                } else {
                    // TODO rename CloseReason::Error? It's not necessesarly an error but a "normal" close event.
                    if let Err(e) = on_incomingmessage_tx.send(
                        IncomingMessage::Close(CloseReason::Error(format!(
                            "{}: {}",
                            event.code(),
                            event.reason()
                        ))),
                    ) {
                        tracing::debug!("OK: Failed to send close event: {e}");
                    }
                }
            }
        })
            as Box<dyn Fn(CloseEvent)>);
        ws.set_onclose(Some(on_close_closure.as_ref().unchecked_ref()));

        let on_error_closure = Closure::wrap(Box::new({
            let on_incomingmessage_tx = on_incomingmessage_tx.clone();
            move |event: Event| {
                tracing::debug!(
                    "websocket onerror: {:?} {:?}",
                    event.as_string(),
                    event,
                );
                if let Err(e) = on_incomingmessage_tx.send(
                    IncomingMessage::Close(CloseReason::Error(
                        event
                            .as_string()
                            .unwrap_or_else(|| "unknown error".to_string()),
                    )),
                ) {
                    tracing::debug!(
                        "OK: Failed to send close even (error handler): {e}"
                    );
                }
            }
        }) as Box<dyn Fn(Event)>);
        ws.set_onerror(Some(on_error_closure.as_ref().unchecked_ref()));

        let onmessage_closure =
            Closure::wrap(Box::new(move |event: MessageEvent| {
                if let Some(message) = event.data().as_string() {
                    tracing::debug!("websocket onmessage: {:?}", message);
                    let result = serde_json::from_str::<Payload>(&message);
                    match result {
                        Ok(payload) => {
                            let _ = on_incomingmessage_tx
                                .clone()
                                .send(IncomingMessage::Message(payload))
                                .ok();
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse payload: {e}");
                        }
                    }
                } else {
                    tracing::warn!(
                        "received non-string JsValue for WS onmessage"
                    )
                }
            }) as Box<dyn Fn(MessageEvent)>);
        ws.set_onmessage(Some(onmessage_closure.as_ref().unchecked_ref()));

        let (tx_open, mut rx_open) = tokio::sync::mpsc::channel(1);
        let onopen_closure = Closure::wrap(Box::new(move |_event: Event| {
            let tx_open = tx_open.clone();
            crate::spawn::spawn(async move {
                let _ = tx_open.send(()).await.ok();
            });
        }) as Box<dyn Fn(Event)>);
        ws.set_onopen(Some(onopen_closure.as_ref().unchecked_ref()));

        tracing::debug!("awaiting onopen");

        let sleep = crate::time::sleep(REQUEST_TIMEOUT);
        tokio::select! {
            _ = rx_open.recv() => {
                // no-op
            }
            _ = sleep => {
                return Err(ConnectError::ConnectFail("Timeout waiting for onopen".to_string()));
            }
            _ = cleanup_rx.cancelled() => {
                return Err(ConnectError::Cleanup);
            }
        }

        ws.set_onopen(None);
        tracing::debug!("onopen received");

        let mut message_id = MIN_RPC_ID;

        if !topics.is_empty() {
            // TODO batch this extra request together with the initial connection

            let payload_request = Payload::Request(Request::new(
                MessageId::new(message_id),
                Params::BatchSubscribe(BatchSubscribe { topics }),
            ));
            message_id += 1;
            let serialized =
                serde_json::to_string(&payload_request).map_err(|e| {
                    ConnectError::ShouldNeverHappen(format!(
                        "Failed to serialize request: {e}"
                    ))
                })?;
            ws.send_with_str(&serialized).map_err(|e| {
                ConnectError::ConnectFail(format!(
                    "Failed to send batch subscribe request: {}",
                    e.as_string().unwrap_or("unknown error".to_string())
                ))
            })?;

            let mut durable_sleep = crate::time::durable_sleep(REQUEST_TIMEOUT);

            loop {
                let incoming_message = tokio::select! {
                    Some(message) = on_incomingmessage_rx.recv() => message,
                    _ = cleanup_rx.cancelled() => {
                        if let Err(e) = ws.close() {
                            tracing::debug!("Failed to close WebSocket (for cleanup): {}", e.as_string().unwrap_or("unknown error".to_string()));
                        }
                        return Err(ConnectError::Cleanup);
                    },
                    _ = durable_sleep.recv() => {
                        if let Err(e) = ws.close() {
                            tracing::debug!("Failed to close WebSocket (for timeout): {}", e.as_string().unwrap_or("unknown error".to_string()));
                        }
                        return Err(ConnectError::ConnectFail("Timeout waiting for batch subscribe response".to_string()));
                    }
                };

                match incoming_message {
                    IncomingMessage::Close(reason) => return Err(reason.into()),
                    IncomingMessage::Message(payload) => {
                        let id = payload.id();
                        match payload {
                            Payload::Request(request) => {
                                tracing::debug!("ignoring unexpected message request in batch subscribe connection: {:?}", request);
                            }
                            Payload::Response(response) => {
                                if id == MessageId::new(message_id) {
                                    // success, no-op
                                    break;
                                } else {
                                    tracing::debug!("ignoring unexpected message response in batch subscribe connection: {:?}", response);
                                }
                            }
                        }
                    }
                }
            }
        }

        // TODO this will soon be moved to initial WebSocket request
        let request = Payload::Request(Request::new(
            MessageId::new(message_id),
            initial_req,
        ));
        let serialized = serde_json::to_string(&request)
            .map_err(|e| ConnectError::ShouldNeverHappen(e.to_string()))?;
        ws.send_with_str(&serialized).map_err(|e| {
            ConnectError::ConnectFail(
                e.as_string().unwrap_or("unknown error".to_string()),
            )
        })?;

        Ok((
            message_id,
            on_incomingmessage_rx,
            (
                ws,
                Closures {
                    on_close: on_close_closure,
                    on_error: on_error_closure,
                    on_message: onmessage_closure,
                },
            ),
        ))
    }
}

struct BackoffState {
    attempt: usize,
}

enum ConnectionState {
    Idle,
    Poisoned,
    MaybeReconnect(Option<BackoffState>),
    ConnectSubscribe(BackoffState),
    AwaitingSubscribeResponse(
        BackoffState,
        u64,
        tokio::sync::mpsc::UnboundedReceiver<IncomingMessage>,
        ConnectWebSocket,
        DurableSleep,
    ),
    Backoff(BackoffState),
    ConnectRequest(
        (Params, tokio::sync::oneshot::Sender<Result<Response, RequestError>>),
    ),
    AwaitingConnectRequestResponse(
        u64,
        tokio::sync::mpsc::UnboundedReceiver<IncomingMessage>,
        ConnectWebSocket,
        (Params, tokio::sync::oneshot::Sender<Result<Response, RequestError>>),
        DurableSleep,
    ),
    Connected(
        u64,
        tokio::sync::mpsc::UnboundedReceiver<IncomingMessage>,
        ConnectWebSocket,
    ),
    AwaitingRequestResponse(
        u64,
        tokio::sync::mpsc::UnboundedReceiver<IncomingMessage>,
        ConnectWebSocket,
        (Params, tokio::sync::oneshot::Sender<Result<Response, RequestError>>),
        DurableSleep,
    ),
    ConnectRetryRequest(
        (Params, tokio::sync::oneshot::Sender<Result<Response, RequestError>>),
    ),
    AwaitingConnectRetryRequestResponse(
        u64,
        tokio::sync::mpsc::UnboundedReceiver<IncomingMessage>,
        ConnectWebSocket,
        tokio::sync::oneshot::Sender<Result<Response, RequestError>>,
        DurableSleep,
    ),
}

// TODO rename to something more generic, e.g. IncomingEvent
// TODO refactor this architecutre (Topic is also passed outside this enum, why duplicate it here? Also what exact values are needed and why (JSON RPC ID, etc.))
#[derive(Debug)]
pub enum IncomingSessionMessage {
    SessionRequest(SessionRequestJsonRpc),
    Disconnect(u64, Topic),
    SessionEvent(Topic, String, serde_json::Value, String),
    SessionUpdate(u64, Topic, crate::sign::protocol_types::SettleNamespaces),
    SessionExtend(u64, Topic),
    SessionConnect(u64, Topic),
    SessionReject(u64, Topic),
    SessionRequestResponse(u64, Topic, SessionRequestJsonRpcResponse),
}

#[allow(clippy::too_many_arguments)]
pub async fn connect_loop_state_machine(
    relay_url: String,
    project_id: ProjectId,
    key: SigningKey,
    session_store: Arc<dyn Storage>,
    session_request_tx: tokio::sync::mpsc::UnboundedSender<(
        Topic,
        IncomingSessionMessage,
    )>,
    request_rx: tokio::sync::mpsc::UnboundedReceiver<(
        Params,
        tokio::sync::oneshot::Sender<Result<Response, RequestError>>,
    )>,
    mut online_rx: tokio::sync::mpsc::UnboundedReceiver<()>,
    cleanup_rx: tokio_util::sync::CancellationToken,
) {
    let (irn_subscription_ack_tx, mut irn_subscription_ack_rx) =
        tokio::sync::mpsc::unbounded_channel();
    let (priority_request_tx, priority_request_rx) =
        tokio::sync::mpsc::unbounded_channel();
    let mut request_rx = PriorityReceiver::new(priority_request_rx, request_rx);

    let handle_irn_subscription = {
        let session_store = session_store.clone();
        let session_request_tx = session_request_tx.clone();
        let irn_subscription_ack_tx = irn_subscription_ack_tx.clone();
        move |id: MessageId,
              sub_msg: Subscription,
              priority_request_tx: tokio::sync::mpsc::UnboundedSender<(
            Params,
            tokio::sync::oneshot::Sender<Result<Response, RequestError>>,
        )>| {
            let session_store = session_store.clone();
            let session_request_tx = session_request_tx.clone();
            let irn_subscription_ack_tx = irn_subscription_ack_tx.clone();
            async move {
                let result = crate::sign::incoming::handle(
                    session_store,
                    sub_msg,
                    session_request_tx,
                    priority_request_tx.clone(),
                );
                match result {
                    Ok(()) => {
                        if let Err(e) = irn_subscription_ack_tx.send(id) {
                            tracing::debug!(
                                "Failed to send subscription ack: {e}"
                            );
                        }
                    }
                    Err(e) => match e {
                        HandleError::Internal(e) => {
                            tracing::error!(
                                "Error handling IRN subscription: {e}"
                            );
                        }
                        HandleError::Dropped(e) => {
                            tracing::debug!(
                                "Dropped message subscription: {e}"
                            );
                            if let Err(e) = irn_subscription_ack_tx.send(id) {
                                tracing::debug!(
                                    "Failed to send subscription ack: {e}"
                                );
                            }
                            // TODO consider unsubscribing from topic
                        }
                        HandleError::Client(e) => {
                            tracing::error!(
                                "Error handling IRN subscription: {e}"
                            );
                            // TODO consider sending RPC error back to sender
                            if let Err(e) = irn_subscription_ack_tx.send(id) {
                                tracing::debug!(
                                    "Failed to send subscription ack: {e}"
                                );
                            }
                        }
                    },
                }
            }
        }
    };

    let mut state = ConnectionState::Idle;
    loop {
        state = match state {
            ConnectionState::Idle => {
                // TODO avoid select! as it doesn't guarantee that `else` branch exists (it will panic otherwise)
                tokio::select! {
                    Some(request) = request_rx.recv() => ConnectionState::ConnectRequest(request),
                    Some(()) = online_rx.recv() => ConnectionState::MaybeReconnect(None),
                    _ = cleanup_rx.cancelled() => break,
                    else => break,
                }
            }
            ConnectionState::Poisoned => {
                // TODO avoid select! as it doesn't guarantee that all branches are covered (it will panic otherwise)
                tokio::select! {
                    Some((_params, response_tx)) = request_rx.recv() => {
                        if let Err(e) =
                            response_tx.send(Err(RequestError::InvalidAuth))
                        {
                            tracing::debug!("Failed to send error response: {e:?}");
                        }
                        ConnectionState::Idle
                    }
                    _ = cleanup_rx.cancelled() => break,
                    else => break,
                }
            }
            ConnectionState::MaybeReconnect(backoff_state) => {
                let all_topics = session_store.get_all_topics();
                let all_topics = match all_topics {
                    Ok(topics) => topics,
                    Err(e) => {
                        tracing::error!("Storage error, exiting loop: {e}");
                        return;
                    }
                };
                if all_topics.is_empty() {
                    ConnectionState::Idle
                } else {
                    ConnectionState::ConnectSubscribe(
                        backoff_state.unwrap_or(BackoffState { attempt: 0 }),
                    )
                }
            }
            ConnectionState::ConnectSubscribe(backoff_state) => {
                let topics = session_store.get_all_topics();
                let topics = match topics {
                    Ok(topics) => topics,
                    Err(e) => {
                        tracing::error!("Storage error, exiting loop: {e}");
                        return;
                    }
                };
                let connect_res = connect(
                    relay_url.clone(),
                    project_id.clone(),
                    &key,
                    vec![],
                    Params::BatchSubscribe(BatchSubscribe { topics }),
                    cleanup_rx.clone(),
                )
                .await;
                match connect_res {
                    Ok((message_id, on_incomingmessage_rx, ws)) => {
                        ConnectionState::AwaitingSubscribeResponse(
                            backoff_state,
                            message_id,
                            on_incomingmessage_rx,
                            ws,
                            crate::time::durable_sleep(REQUEST_TIMEOUT),
                        )
                    }
                    Err(e) => match e {
                        ConnectError::ConnectFail(reason) => {
                            tracing::debug!(
                                "ConnectSubscribe failed: {reason}"
                            );
                            ConnectionState::Backoff(backoff_state)
                        }
                        ConnectError::Cleanup => {
                            break;
                        }
                        ConnectError::InvalidAuth => ConnectionState::Poisoned,
                        ConnectError::ShouldNeverHappen(reason) => {
                            tracing::error!("ConnectSubscribe should never happen: {reason}");
                            ConnectionState::Backoff(backoff_state)
                        }
                    },
                }
            }
            ConnectionState::AwaitingSubscribeResponse(
                backoff_state,
                message_id,
                mut on_incomingmessage_rx,
                ws,
                mut sleep,
            ) => {
                // TODO avoid select! as it doesn't guarantee that all branches are covered (it will panic otherwise)
                tokio::select! {
                    message = on_incomingmessage_rx.recv() => {
                        if let Some(message) = message {
                            match message {
                                IncomingMessage::Close(reason) => {
                                    match reason {
                                        CloseReason::InvalidAuth => {
                                            ConnectionState::Poisoned
                                        }
                                        CloseReason::Error(reason) => {
                                            tracing::debug!("AwaitingSubscribeResponse: CloseReason::Error: {reason}");
                                            ConnectionState::Backoff(backoff_state)
                                        }
                                    }
                                }
                                IncomingMessage::Message(payload) => {
                                    let id = payload.id();
                                    match payload {
                                        Payload::Request(request) => {
                                            tracing::warn!("ignoring message request in AwaitingSubscribeResponse state: {:?}", request);
                                            ConnectionState::AwaitingSubscribeResponse(
                                                backoff_state,
                                                message_id,
                                                on_incomingmessage_rx,
                                                ws,
                                                sleep,
                                            )
                                        }
                                        Payload::Response(response) => {
                                            if id == MessageId::new(message_id) {
                                                ConnectionState::Connected(message_id, on_incomingmessage_rx, ws)
                                            } else {
                                                tracing::warn!("ignoring message response in AwaitingSubscribeResponse state: {:?}", response);
                                                ConnectionState::AwaitingSubscribeResponse(
                                                    backoff_state,
                                                    message_id,
                                                    on_incomingmessage_rx,
                                                    ws,
                                                    sleep,
                                                )
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            ConnectionState::Backoff(backoff_state)
                        }
                    },
                    Some(()) = sleep.recv() => ConnectionState::Backoff(backoff_state),
                    _ = cleanup_rx.cancelled() => {
                        #[cfg(target_arch = "wasm32")]
                        if let Err(e) = ws.0.close() {
                            tracing::debug!("Failed to close websocket: {e:?}");
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        if let Err(e) = ws.1.send(()) {
                            tracing::debug!("Failed to send close event: {e}");
                        }
                        break;
                    }
                }
            }
            ConnectionState::Backoff(backoff_state) => {
                let attempt = backoff_state.attempt;
                const BACKOFF_VALUES: [u64; 4] = [1000, 1000, 2000, 5000];
                let sleep = crate::time::sleep(Duration::from_millis(
                    BACKOFF_VALUES[attempt],
                ));

                // TODO avoid select! as it doesn't guarantee that all branches are covered (it will panic otherwise)
                tokio::select! {
                    Some(req) = request_rx.recv() => ConnectionState::ConnectRequest(req),
                    () = sleep => {
                        let next_attempt = (attempt + 1).min(BACKOFF_VALUES.len() - 1);
                        ConnectionState::MaybeReconnect(Some(BackoffState { attempt: next_attempt }))
                    }
                    _ = cleanup_rx.cancelled() => break,
                    else => break,
                }
            }
            ConnectionState::ConnectRequest((request, response_tx)) => {
                let topics = session_store.get_all_topics();
                let topics = match topics {
                    Ok(topics) => topics,
                    Err(e) => {
                        tracing::error!("Storage error, exiting loop: {e}");
                        return;
                    }
                };
                let connect_res = connect(
                    relay_url.clone(),
                    project_id.clone(),
                    &key,
                    topics,
                    request.clone(),
                    cleanup_rx.clone(),
                )
                .await;
                match connect_res {
                    Ok((message_id, on_incomingmessage_rx, ws)) => {
                        ConnectionState::AwaitingConnectRequestResponse(
                            message_id,
                            on_incomingmessage_rx,
                            ws,
                            (request, response_tx),
                            crate::time::durable_sleep(REQUEST_TIMEOUT),
                        )
                    }
                    Err(e) => {
                        if let Err(e) = response_tx.send(Err(e.clone().into()))
                        {
                            tracing::warn!(
                                "Failed to send error response: {e:?}"
                            );
                        }
                        match e {
                            ConnectError::ConnectFail(reason) => {
                                tracing::debug!(
                                    "ConnectRequest failed: {reason}"
                                );
                                ConnectionState::MaybeReconnect(None)
                            }
                            ConnectError::Cleanup => {
                                break;
                            }
                            ConnectError::InvalidAuth => {
                                ConnectionState::Poisoned
                            }
                            ConnectError::ShouldNeverHappen(reason) => {
                                tracing::error!("ConnectRequest should never happen: {reason}");
                                ConnectionState::Idle
                            }
                        }
                    }
                }
            }
            ConnectionState::AwaitingConnectRequestResponse(
                message_id,
                mut on_incomingmessage_rx,
                ws,
                (request, response_tx),
                mut sleep,
            ) => {
                // TODO avoid select! as it doesn't guarantee that all branches are covered (it will panic otherwise)
                tokio::select! {
                    message = on_incomingmessage_rx.recv() => {
                        if let Some(message) = message {
                            match message {
                                IncomingMessage::Close(reason) => {
                                    match reason {
                                        CloseReason::InvalidAuth => {
                                            if let Err(e) =
                                                response_tx.send(Err(RequestError::InvalidAuth))
                                            {
                                                tracing::warn!("Failed to send error response: {e:?}");
                                            }
                                            ConnectionState::Poisoned
                                        }
                                        CloseReason::Error(reason) => {
                                            tracing::debug!("AwaitingConnectRequestResponse: CloseReason::Error: {reason}");
                                            if let Err(e) =
                                                response_tx.send(Err(RequestError::Offline))
                                            {
                                                tracing::warn!("Failed to send error response: {e:?}");
                                            }
                                            ConnectionState::MaybeReconnect(None)
                                        }
                                    }
                                }
                                IncomingMessage::Message(payload) => {
                                    let id = payload.id();
                                    match payload {
                                        Payload::Request(payload_request) => {
                                            tracing::warn!("ignoring message request in AwaitingSubscribeResponse state: {:?}", payload_request);
                                            ConnectionState::AwaitingConnectRequestResponse(
                                                message_id,
                                                on_incomingmessage_rx,
                                                ws,
                                                (request, response_tx),
                                                sleep,
                                            )
                                        }
                                        Payload::Response(response) => {
                                            if id == MessageId::new(message_id) {
                                                if let Err(e) =
                                                    response_tx.send(Ok(response))
                                                {
                                                    tracing::warn!("Failed to send response: {e:?}");
                                                }
                                                ConnectionState::Connected(message_id, on_incomingmessage_rx, ws)
                                            } else {
                                                tracing::warn!("ignoring message response in AwaitingSubscribeResponse state: {:?}", response);
                                                ConnectionState::AwaitingConnectRequestResponse(
                                                    message_id,
                                                    on_incomingmessage_rx,
                                                    ws,
                                                    (request, response_tx),
                                                    sleep,
                                                )
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            if let Err(e) =
                                response_tx.send(Err(RequestError::Offline))
                            {
                                tracing::warn!("Failed to send error response: {e:?}");
                            }
                            ConnectionState::MaybeReconnect(None)
                        }
                    },
                    Some(()) = sleep.recv() => {
                        if let Err(e) =
                            response_tx.send(Err(RequestError::Offline))
                        {
                            tracing::warn!("Failed to send error response: {e:?}");
                        }
                        ConnectionState::MaybeReconnect(None)
                    }
                    _ = cleanup_rx.cancelled() => {
                        #[cfg(target_arch = "wasm32")]
                        if let Err(e) = ws.0.close() {
                            tracing::debug!("Failed to close websocket: {e:?}");
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        if let Err(e) = ws.1.send(()) {
                            tracing::debug!("Failed to send close event: {e}");
                        }
                        break;
                    },
                }
            }
            ConnectionState::Connected(
                message_id,
                mut on_incomingmessage_rx,
                ws,
            ) => {
                // TODO avoid select! as it doesn't guarantee that all branches are covered (it will panic otherwise)
                tokio::select! {
                    message = on_incomingmessage_rx.recv() => {
                        if let Some(message) = message {
                            match message {
                                IncomingMessage::Close(reason) => {
                                    if reason == CloseReason::InvalidAuth {
                                        tracing::warn!("server misbehaved: invalid auth in Connected state");
                                        ConnectionState::Poisoned
                                    } else {
                                        ConnectionState::MaybeReconnect(None)
                                    }
                                }
                                IncomingMessage::Message(payload) => {
                                    let id = payload.id();
                                    match payload {
                                        Payload::Request(request) => {
                                            #[allow(clippy::single_match)]
                                            match request.params {
                                                Params::Subscription(
                                                    sub_msg
                                                ) => {
                                                    handle_irn_subscription(
                                                        id,
                                                        sub_msg,
                                                        priority_request_tx.clone(),
                                                    )
                                                    .await;
                                                }
                                                _ => tracing::warn!("ignoring message request in Connected state: {:?}", request),
                                            }
                                            ConnectionState::Connected(
                                                message_id,
                                                on_incomingmessage_rx,
                                                ws,
                                            )
                                        }
                                        Payload::Response(response) => {
                                            tracing::warn!("ignoring message response in Connected state: {:?}", response);
                                            ConnectionState::Connected(
                                                message_id,
                                                on_incomingmessage_rx,
                                                ws,
                                            )
                                        }
                                    }
                                }
                            }
                        } else {
                            ConnectionState::MaybeReconnect(None)
                        }
                    }
                    request = request_rx.recv() => {
                        if let Some((request, response_tx)) = request {
                            let message_id = message_id + 1;
                            let payload_request = Payload::Request(Request::new(
                                MessageId::new(message_id),
                                request.clone(),
                            ));
                            let serialized = serde_json::to_string(&payload_request)
                                .map_err(|e| {
                                    RequestError::ShouldNeverHappen(format!(
                                        "Failed to serialize request: {e}"
                                    ))
                                })
                                .expect("TODO");
                            #[cfg(target_arch = "wasm32")]
                            ws.0.send_with_str(&serialized).expect("TODO");
                            #[cfg(not(target_arch = "wasm32"))]
                            ws.0.send(serialized).expect("TODO");

                            ConnectionState::AwaitingRequestResponse(
                                message_id,
                                on_incomingmessage_rx,
                                ws,
                                (request, response_tx),
                                crate::time::durable_sleep(REQUEST_TIMEOUT),
                            )
                        } else {
                            #[cfg(target_arch = "wasm32")]
                            if let Err(e) = ws.0.close() {
                                tracing::debug!("Failed to close websocket: {e:?}");
                            }
                            #[cfg(not(target_arch = "wasm32"))]
                            if let Err(e) = ws.1.send(()) {
                                tracing::debug!("Failed to send close event: {e}");
                            }
                            break;
                        }
                    }
                    id = irn_subscription_ack_rx.recv() => {
                        if let Some(id) = id {
                            let request = Payload::Response(Response::Success(
                                SuccessfulResponse {
                                    id,
                                    result: serde_json::to_value(true)
                                        .expect("TODO"),
                                    jsonrpc: "2.0".to_string().into(),
                                },
                            ));
                            let serialized = serde_json::to_string(&request)
                                .map_err(|e| {
                                    RequestError::ShouldNeverHappen(format!(
                                        "Failed to serialize request: {e}"
                                    ))
                                })
                                .expect("TODO");
                            #[cfg(target_arch = "wasm32")]
                            ws.0.send_with_str(&serialized).expect("TODO");
                            #[cfg(not(target_arch = "wasm32"))]
                            ws.0.send(serialized).expect("TODO");

                            ConnectionState::Connected(
                                message_id,
                                on_incomingmessage_rx,
                                ws,
                            )
                        } else {
                            #[cfg(target_arch = "wasm32")]
                            if let Err(e) = ws.0.close() {
                                tracing::debug!("Failed to close websocket: {e:?}");
                            }
                            #[cfg(not(target_arch = "wasm32"))]
                            if let Err(e) = ws.1.send(()) {
                                tracing::debug!("Failed to send close event: {e}");
                            }
                            break;
                        }
                    }
                    _ = cleanup_rx.cancelled() => {
                        #[cfg(target_arch = "wasm32")]
                        if let Err(e) = ws.0.close() {
                            tracing::debug!("Failed to close websocket: {e:?}");
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        if let Err(e) = ws.1.send(()) {
                            tracing::debug!("Failed to send close event: {e}");
                        }
                        break;
                    },
                }
            }
            ConnectionState::AwaitingRequestResponse(
                message_id,
                mut on_incomingmessage_rx,
                ws,
                (request, response_tx),
                mut sleep,
            ) => {
                // TODO avoid select! as it doesn't guarantee that all branches are covered (it will panic otherwise)
                tokio::select! {
                    message = on_incomingmessage_rx.recv() => {
                        if let Some(message) = message {
                            match message {
                                IncomingMessage::Close(reason) => {
                                    if reason == CloseReason::InvalidAuth {
                                        tracing::warn!("server misbehaved: invalid auth in Connected state");
                                        if let Err(e) =
                                            response_tx.send(Err(RequestError::InvalidAuth))
                                        {
                                            tracing::warn!("Failed to send error response: {e:?}");
                                        }
                                        ConnectionState::Poisoned
                                    } else {
                                        ConnectionState::ConnectRetryRequest((request, response_tx))
                                    }
                                }
                                IncomingMessage::Message(payload) => {
                                    let id = payload.id();
                                    match payload {
                                        Payload::Request(payload_request) => {
                                            #[allow(clippy::single_match)]
                                            match payload_request.params {
                                                Params::Subscription(
                                                    sub_msg
                                                ) => {
                                                    handle_irn_subscription(
                                                        id,
                                                        sub_msg,
                                                        priority_request_tx.clone(),
                                                    )
                                                    .await;
                                                }
                                                _ => tracing::warn!("ignoring message request in AwaitingRequestResponse state: {:?}", payload_request),
                                            }
                                            ConnectionState::AwaitingRequestResponse(
                                                message_id,
                                                on_incomingmessage_rx,
                                                ws,
                                                (request, response_tx),
                                                sleep,
                                            )
                                        }
                                        Payload::Response(response) => {
                                            if id == MessageId::new(message_id) {
                                                if let Err(e) =
                                                    response_tx.send(Ok(response))
                                                {
                                                    tracing::warn!("Failed to send response in AwaitingRequestResponse state: {e:?}");
                                                }
                                                ConnectionState::Connected(message_id, on_incomingmessage_rx, ws)
                                            } else {
                                                tracing::warn!("ignoring message response in AwaitingSubscribeResponse state: {:?}", response);
                                                ConnectionState::AwaitingRequestResponse(
                                                    message_id,
                                                    on_incomingmessage_rx,
                                                    ws,
                                                    (request, response_tx),
                                                    sleep,
                                                )
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            #[cfg(target_arch = "wasm32")]
                            if let Err(e) = ws.0.close() {
                                tracing::debug!("Failed to close websocket: {e:?}");
                            }
                            #[cfg(not(target_arch = "wasm32"))]
                            if let Err(e) = ws.1.send(()) {
                                tracing::debug!("Failed to send close event: {e}");
                            }
                            break;
                        }
                    }
                    Some(()) = sleep.recv() => ConnectionState::ConnectRetryRequest((request, response_tx)),
                    _ = cleanup_rx.cancelled() => {
                        #[cfg(target_arch = "wasm32")]
                        if let Err(e) = ws.0.close() {
                            tracing::debug!("Failed to close websocket: {e:?}");
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        if let Err(e) = ws.1.send(()) {
                            tracing::debug!("Failed to send close event: {e}");
                        }
                        break;
                    },
                }
            }
            ConnectionState::ConnectRetryRequest((request, response_tx)) => {
                let topics = session_store.get_all_topics();
                let topics = match topics {
                    Ok(topics) => topics,
                    Err(e) => {
                        tracing::error!("Storage error, exiting loop: {e}");
                        return;
                    }
                };
                let connect_fut = connect(
                    relay_url.clone(),
                    project_id.clone(),
                    &key,
                    topics,
                    request,
                    cleanup_rx.clone(),
                );
                tokio::select! {
                    connect_res = connect_fut => {
                        match connect_res {
                            Ok((message_id, on_incomingmessage_rx, ws)) => {
                                ConnectionState::AwaitingConnectRetryRequestResponse(
                                    message_id,
                                    on_incomingmessage_rx,
                                    ws,
                                    response_tx,
                                    crate::time::durable_sleep(REQUEST_TIMEOUT),
                                )
                            }
                            Err(e) => {
                                if let Err(e) = response_tx.send(Err(e.clone().into())) {
                                    tracing::warn!(
                                        "Failed to send error response: {e:?}"
                                    );
                                }
                                match e {
                                    ConnectError::ConnectFail(reason) => {
                                        tracing::debug!("ConnectRequest failed: {reason}");
                                        ConnectionState::MaybeReconnect(None)
                                    }
                                    ConnectError::Cleanup => {
                                        break;
                                    }
                                    ConnectError::InvalidAuth => {
                                        ConnectionState::Poisoned
                                    }
                                    ConnectError::ShouldNeverHappen(reason) => {
                                        tracing::error!("ConnectRequest should never happen: {reason}");
                                        ConnectionState::MaybeReconnect(None)
                                    }
                                }
                            }
                        }
                    }
                    _ = cleanup_rx.cancelled() => break,
                }
            }
            ConnectionState::AwaitingConnectRetryRequestResponse(
                message_id,
                mut on_incomingmessage_rx,
                ws,
                response_tx,
                mut sleep,
            ) => {
                // TODO avoid select! as it doesn't guarantee that all branches are covered (it will panic otherwise)
                tokio::select! {
                    message = on_incomingmessage_rx.recv() => {
                        if let Some(message) = message {
                            match message {
                                IncomingMessage::Close(reason) => {
                                    if reason == CloseReason::InvalidAuth {
                                        tracing::warn!("server misbehaved: invalid auth in Connected state");
                                        if let Err(e) =
                                            response_tx.send(Err(RequestError::InvalidAuth))
                                        {
                                            tracing::warn!("Failed to send error response: {e:?}");
                                        }
                                        ConnectionState::Poisoned
                                    } else {
                                        if let Err(e) =
                                            response_tx.send(Err(RequestError::Offline))
                                        {
                                            tracing::warn!("Failed to send error response: {e:?}");
                                        }
                                        ConnectionState::MaybeReconnect(None)
                                    }
                                }
                                IncomingMessage::Message(payload) => {
                                    let id = payload.id();
                                    match payload {
                                        Payload::Request(request) => {
                                            // TODO consider handling anyway, if possible
                                            tracing::warn!("ignoring message request in AwaitingConnectRetryRequestResponse state: {:?}", request);
                                            ConnectionState::AwaitingConnectRetryRequestResponse(
                                                message_id,
                                                on_incomingmessage_rx,
                                                ws,
                                                response_tx,
                                                sleep,
                                            )
                                        }
                                        Payload::Response(response) => {
                                            if id == MessageId::new(message_id) {
                                                if let Err(e) =
                                                    response_tx.send(Ok(response))
                                                {
                                                    tracing::warn!("Failed to send response: {e:?}");
                                                }
                                                ConnectionState::Connected(message_id, on_incomingmessage_rx, ws)
                                            } else {
                                                tracing::warn!("ignoring message response in AwaitingConnectRetryRequestResponse state: {:?}", response);
                                                ConnectionState::AwaitingConnectRetryRequestResponse(
                                                    message_id,
                                                    on_incomingmessage_rx,
                                                    ws,
                                                    response_tx,
                                                    sleep,
                                                )
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            #[cfg(target_arch = "wasm32")]
                            if let Err(e) = ws.0.close() {
                                tracing::debug!("Failed to close websocket: {e:?}");
                            }
                            #[cfg(not(target_arch = "wasm32"))]
                            if let Err(e) = ws.1.send(()) {
                                tracing::debug!("Failed to send close event: {e}");
                            }
                            break;
                        }
                    }
                    Some(()) = sleep.recv() => ConnectionState::MaybeReconnect(None),
                    _ = cleanup_rx.cancelled() => {
                        #[cfg(target_arch = "wasm32")]
                        if let Err(e) = ws.0.close() {
                            tracing::debug!("Failed to close websocket: {e:?}");
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        if let Err(e) = ws.1.send(()) {
                            tracing::debug!("Failed to send close event: {e}");
                        }
                        break;
                    },
                }
            }
        };
    }

    request_rx.close();
    while let Some((_request, response_tx)) = request_rx.recv().await {
        if let Err(e) = response_tx.send(Err(RequestError::Cleanup)) {
            tracing::warn!("Failed to send cleanup error response: {e:?}");
        }
    }
}
