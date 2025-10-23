use {
    crate::{
        sign::{
            client::{AttestationCallback, MaybeVerifiedRequest},
            client_errors::RequestError,
            incoming::HandleError,
            priority_future::PriorityReceiver,
            protocol_types::{
                SessionRequestJsonRpc, SessionRequestJsonRpcResponse,
            },
            relay_url::ConnectionOptions,
            storage::Storage,
            utils::{DecryptedHash, EncryptedHash},
            VerifyContext,
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

// Wrapper types to prevent mixing up sent vs next message IDs
/// A message ID that has been sent over the websocket
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SentMessageId(u64);

impl SentMessageId {
    fn new(id: u64) -> Self {
        Self(id)
    }

    fn get(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for SentMessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The next available message ID for sending
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NextMessageId(u64);

impl NextMessageId {
    fn new(id: u64) -> Self {
        Self(id)
    }

    fn get(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for NextMessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
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

// Common return type for both connection functions
type ConnectOutput = (
    NextMessageId, // next_message_id (next available ID for sending)
    tokio::sync::mpsc::UnboundedReceiver<IncomingMessage>,
    ConnectWebSocket,
);

// Connect to relay, optionally sending a prepared initial request
// Returns (next_message_id, rx, ws)
// - next_message_id is the next available ID for future messages
// TODO: When relay supports it, initial_req will be sent as a query param in the URL
async fn connect(
    relay_url: String,
    project_id: ProjectId,
    key: &SigningKey,
    topics: Vec<Topic>,
    initial_req: Option<PreparedMessage>,
    cleanup_rx: tokio_util::sync::CancellationToken,
    probe_group: Option<String>,
) -> Result<ConnectOutput, ConnectError> {
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

    tracing::debug!(group = probe_group.clone(), probe = "relay_connect_begin");

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
        tracing::debug!(
            group = probe_group.clone(),
            probe = "relay_connect_success"
        );

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
                        tracing::debug!("sending websocket message: {message}");
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

        let mut message_id = NextMessageId::new(MIN_RPC_ID);

        if !topics.is_empty() {
            // TODO batch this extra request together with the initial connection

            let payload_request = Payload::Request(Request::new(
                MessageId::new(message_id.get()),
                Params::BatchSubscribe(BatchSubscribe { topics }),
            ));
            message_id = NextMessageId::new(message_id.get() + 1);
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
                                if id == MessageId::new(message_id.get()) {
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

        // Send the initial request if provided
        if let Some(prepared) = initial_req {
            send_prepared_message(
                &(outgoing_tx.clone(), close_tx.clone()),
                &prepared,
            )?;
            message_id = prepared.next_message_id;
        }

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

        let mut message_id = NextMessageId::new(MIN_RPC_ID);

        if !topics.is_empty() {
            // TODO batch this extra request together with the initial connection

            let payload_request = Payload::Request(Request::new(
                MessageId::new(message_id.get()),
                Params::BatchSubscribe(BatchSubscribe { topics }),
            ));
            message_id = NextMessageId::new(message_id.get() + 1);
            let serialized =
                serde_json::to_string(&payload_request).map_err(|e| {
                    ConnectError::ShouldNeverHappen(format!(
                        "Failed to serialize request: {e}"
                    ))
                })?;
            tracing::debug!("sending websocket message: {serialized}");
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
                                if id == MessageId::new(message_id.get()) {
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

        // Send the initial request if provided
        let ws_handle = (
            ws.clone(),
            Closures {
                on_close: on_close_closure,
                on_error: on_error_closure,
                on_message: onmessage_closure,
            },
        );

        if let Some(prepared) = initial_req {
            send_prepared_message(&ws_handle, &prepared)?;
            message_id = prepared.next_message_id;
        }

        Ok((message_id, on_incomingmessage_rx, ws_handle))
    }
}

// Represents a prepared websocket message ready to be sent
#[derive(Clone)]
struct PreparedMessage {
    sent_message_id: SentMessageId,
    next_message_id: NextMessageId,
    serialized: String,
}

// Prepare a websocket message (allocates ID and serializes)
fn prepare_websocket_message(
    next_message_id: NextMessageId,
    params: Params,
) -> Result<PreparedMessage, ConnectError> {
    let sent_id = next_message_id.get();
    let request =
        Payload::Request(Request::new(MessageId::new(sent_id), params));
    let serialized = serde_json::to_string(&request)
        .map_err(|e| ConnectError::ShouldNeverHappen(e.to_string()))?;

    Ok(PreparedMessage {
        sent_message_id: SentMessageId::new(sent_id),
        next_message_id: NextMessageId::new(sent_id + 1),
        serialized,
    })
}

// Send a prepared message over the websocket
fn send_prepared_message(
    ws: &ConnectWebSocket,
    prepared: &PreparedMessage,
) -> Result<(), ConnectError> {
    #[cfg(target_arch = "wasm32")]
    {
        ws.0.send_with_str(&prepared.serialized).map_err(|e| {
            ConnectError::ConnectFail(
                e.as_string().unwrap_or("unknown error".to_string()),
            )
        })?;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        ws.0.send(prepared.serialized.clone())
            .map_err(|e| ConnectError::ConnectFail(e.to_string()))?;
    }

    tracing::debug!(
        "sent websocket message with message_id={}",
        prepared.sent_message_id
    );
    Ok(())
}

// Helper to spawn attestation fetch task and return the receiver
fn spawn_attestation_fetch(
    encrypted_id: EncryptedHash,
    decrypted_id: DecryptedHash,
    project_id: ProjectId,
) -> tokio::sync::oneshot::Receiver<String> {
    let (attestation_tx, attestation_rx) = tokio::sync::oneshot::channel();

    crate::spawn::spawn(async move {
        match crate::sign::verify::create_attestation(
            encrypted_id,
            decrypted_id,
            project_id,
        )
        .await
        {
            Ok(attestation) => {
                tracing::debug!(
                    "Attestation received: {} bytes",
                    attestation.len()
                );
                let _ = attestation_tx.send(attestation);
            }
            Err(e) => {
                tracing::error!("Failed to create attestation: {e:?}");
                let _ = attestation_tx.send(String::new());
            }
        }
    });

    attestation_rx
}

#[derive(Clone, Copy)]
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
        SentMessageId, // sent_message_id (waiting for response with this ID)
        NextMessageId, // next_message_id (for sending future messages)
        tokio::sync::mpsc::UnboundedReceiver<IncomingMessage>,
        ConnectWebSocket,
        DurableSleep,
    ),
    Backoff(BackoffState),
    ConnectRequest(
        (
            MaybeVerifiedRequest,
            tokio::sync::oneshot::Sender<Result<Response, RequestError>>,
        ),
    ),
    AwaitingConnectRequestResponse(
        SentMessageId, // sent_message_id (waiting for response with this ID)
        NextMessageId, // next_message_id (for sending future messages)
        tokio::sync::mpsc::UnboundedReceiver<IncomingMessage>,
        ConnectWebSocket,
        (tokio::sync::oneshot::Sender<Result<Response, RequestError>>,),
        DurableSleep,
    ),
    ConnectRequestAwaitingAttestation(
        NextMessageId, // next available message_id (will use this and increment)
        tokio::sync::mpsc::UnboundedReceiver<IncomingMessage>,
        ConnectWebSocket,
        AttestationCallback,
        tokio::sync::oneshot::Receiver<String>, // attestation receiver (created by state machine)
        tokio::sync::oneshot::Sender<Result<Response, RequestError>>,
    ),
    Connected(
        NextMessageId,
        tokio::sync::mpsc::UnboundedReceiver<IncomingMessage>,
        ConnectWebSocket,
    ),
    ConnectedAwaitingAttestation(
        NextMessageId, // next available message_id (will use this and increment)
        tokio::sync::mpsc::UnboundedReceiver<IncomingMessage>,
        ConnectWebSocket,
        AttestationCallback,
        tokio::sync::oneshot::Receiver<String>,
        tokio::sync::oneshot::Sender<Result<Response, RequestError>>,
    ),
    AwaitingRequestResponse(
        SentMessageId, // sent_message_id (waiting for response with this ID)
        NextMessageId, // next_message_id (for sending future messages)
        tokio::sync::mpsc::UnboundedReceiver<IncomingMessage>,
        ConnectWebSocket,
        (Params, tokio::sync::oneshot::Sender<Result<Response, RequestError>>),
        DurableSleep,
    ),
    ConnectRetryRequest(
        (Params, tokio::sync::oneshot::Sender<Result<Response, RequestError>>),
    ),
    AwaitingConnectRetryRequestResponse(
        SentMessageId, // sent_message_id (waiting for response with this ID)
        NextMessageId, // next_message_id (for sending future messages)
        tokio::sync::mpsc::UnboundedReceiver<IncomingMessage>,
        ConnectWebSocket,
        tokio::sync::oneshot::Sender<Result<Response, RequestError>>,
        DurableSleep,
    ),
}

impl std::fmt::Debug for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ConnectionState::Idle => "Idle",
            ConnectionState::Poisoned => "Poisoned",
            ConnectionState::MaybeReconnect(_) => "MaybeReconnect(_)",
            ConnectionState::ConnectSubscribe(_) => "ConnectSubscribe(_)",
            ConnectionState::AwaitingSubscribeResponse(_, _, _, _, _, _) => {
                "AwaitingSubscribeResponse(_, _, _, _, _, _)"
            }
            ConnectionState::Backoff(_) => "Backoff(_)",
            ConnectionState::ConnectRequest(_) => "ConnectRequest(_)",
            ConnectionState::AwaitingConnectRequestResponse(
                _,
                _,
                _,
                _,
                _,
                _,
            ) => "AwaitingConnectRequestResponse(_, _, _, _, _, _)",
            ConnectionState::ConnectRequestAwaitingAttestation(
                _,
                _,
                _,
                _,
                _,
                _,
            ) => "ConnectRequestAwaitingAttestation(_, _, _, _, _, _)",
            ConnectionState::Connected(_, _, _) => "Connected(_, _, _)",
            ConnectionState::ConnectedAwaitingAttestation(_, _, _, _, _, _) => {
                "ConnectedAwaitingAttestation(_, _, _, _, _, _)"
            }
            ConnectionState::AwaitingRequestResponse(_, _, _, _, _, _) => {
                "AwaitingRequestResponse(_, _, _, _, _, _)"
            }
            ConnectionState::ConnectRetryRequest(_) => "ConnectRetryRequest(_)",
            ConnectionState::AwaitingConnectRetryRequestResponse(
                _,
                _,
                _,
                _,
                _,
                _,
            ) => "AwaitingConnectRetryRequestResponse(_, _, _, _, _, _)",
        };
        write!(f, "{name}")
    }
}

// TODO rename to something more generic, e.g. IncomingEvent
// TODO refactor this architecutre (Topic is also passed outside this enum, why duplicate it here? Also what exact values are needed and why (JSON RPC ID, etc.))
#[derive(Debug)]
pub enum IncomingSessionMessage {
    SessionRequest(SessionRequestJsonRpc, VerifyContext),
    Disconnect(u64, Topic),
    SessionEvent(Topic, String, serde_json::Value, String),
    SessionUpdate(u64, Topic, crate::sign::protocol_types::SettleNamespaces),
    SessionExtend(u64, Topic),
    SessionConnect(u64, Topic),
    SessionReject(u64, Topic),
    SessionRequestResponse(u64, Topic, SessionRequestJsonRpcResponse),
}

// MaybeVerifiedRequest is now defined in client.rs and imported via the parent module

#[allow(clippy::too_many_arguments)]
pub async fn connect_loop_state_machine(
    relay_url: String,
    project_id: ProjectId,
    key: SigningKey,
    session_store: Arc<dyn Storage>,
    http_client: reqwest::Client,
    session_request_tx: tokio::sync::mpsc::UnboundedSender<(
        Topic,
        IncomingSessionMessage,
    )>,
    request_rx: tokio::sync::mpsc::UnboundedReceiver<(
        MaybeVerifiedRequest,
        tokio::sync::oneshot::Sender<Result<Response, RequestError>>,
    )>,
    mut online_rx: tokio::sync::mpsc::UnboundedReceiver<()>,
    cleanup_rx: tokio_util::sync::CancellationToken,
    probe_group: Option<String>,
) {
    let (irn_subscription_ack_tx, mut irn_subscription_ack_rx) =
        tokio::sync::mpsc::unbounded_channel();
    let (priority_request_tx, priority_request_rx) =
        tokio::sync::mpsc::unbounded_channel();
    let mut request_rx = PriorityReceiver::new(priority_request_rx, request_rx);

    let handle_irn_subscription = {
        let session_store = session_store.clone();
        let http_client = http_client.clone();
        let session_request_tx = session_request_tx.clone();
        let irn_subscription_ack_tx = irn_subscription_ack_tx.clone();
        move |id: MessageId,
              sub_msg: Subscription,
              priority_request_tx: tokio::sync::mpsc::UnboundedSender<(
            MaybeVerifiedRequest,
            tokio::sync::oneshot::Sender<Result<Response, RequestError>>,
        )>| {
            let session_store = session_store.clone();
            let http_client = http_client.clone();
            let session_request_tx = session_request_tx.clone();
            let irn_subscription_ack_tx = irn_subscription_ack_tx.clone();
            async move {
                let result = crate::sign::incoming::handle(
                    session_store,
                    http_client.clone(),
                    sub_msg,
                    session_request_tx,
                    priority_request_tx.clone(),
                )
                .await;
                // TODO relay only listens for these types of ACKs for 4s. We should handle longer processing times e.g. via batchFetch in-case of long network latency or other factors
                match result {
                    Ok(()) => {
                        if let Err(e) = irn_subscription_ack_tx.send(id) {
                            tracing::debug!(
                                "Failed to send subscription ack: {e}"
                            );
                        }
                    }
                    Err(e) => match e {
                        HandleError::Temporary(e) => {
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
                        HandleError::Peer(e) => {
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
                        HandleError::AlreadyHandled => {
                            tracing::debug!("Already handled, ignoring");
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
        tracing::debug!("connect state: {state:?}");
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
                let prepared = prepare_websocket_message(
                    NextMessageId::new(MIN_RPC_ID),
                    Params::BatchSubscribe(BatchSubscribe { topics }),
                );
                let prepared = match prepared {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::error!(
                            "Failed to prepare batch subscribe: {e:?}"
                        );
                        state = ConnectionState::Backoff(backoff_state);
                        continue;
                    }
                };

                let connect_res = connect(
                    relay_url.clone(),
                    project_id.clone(),
                    &key,
                    vec![],
                    Some(prepared.clone()),
                    cleanup_rx.clone(),
                    probe_group.clone(),
                )
                .await;
                match connect_res {
                    Ok((next_message_id, on_incomingmessage_rx, ws)) => {
                        ConnectionState::AwaitingSubscribeResponse(
                            backoff_state,
                            prepared.sent_message_id,
                            next_message_id,
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
                sent_message_id,
                next_message_id,
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
                                                sent_message_id,
                                                next_message_id,
                                                on_incomingmessage_rx,
                                                ws,
                                                sleep,
                                            )
                                        }
                                        Payload::Response(response) => {
                                            if id == MessageId::new(sent_message_id.get()) {
                                                ConnectionState::Connected(next_message_id, on_incomingmessage_rx, ws)
                                            } else {
                                                tracing::warn!("ignoring message response in AwaitingSubscribeResponse state: {:?}", response);
                                                ConnectionState::AwaitingSubscribeResponse(
                                                    backoff_state,
                                                    sent_message_id,
                                                    next_message_id,
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

                // Handle MaybeVerifiedRequest
                let next_state = if let MaybeVerifiedRequest::Unverified(
                    params,
                ) = request
                {
                    // Prepare the message first (allocates ID)
                    let prepared = prepare_websocket_message(
                        NextMessageId::new(MIN_RPC_ID),
                        params,
                    )
                    .expect(
                        "Failed to serialize Params - this should never happen",
                    );

                    // Connect with the prepared request
                    let connect_res = connect(
                        relay_url.clone(),
                        project_id.clone(),
                        &key,
                        topics,
                        Some(prepared.clone()),
                        cleanup_rx.clone(),
                        probe_group.clone(),
                    )
                    .await;

                    match connect_res {
                        Ok((next_message_id, on_incomingmessage_rx, ws)) => {
                            ConnectionState::AwaitingConnectRequestResponse(
                                prepared.sent_message_id,
                                next_message_id,
                                on_incomingmessage_rx,
                                ws,
                                (response_tx,),
                                crate::time::durable_sleep(REQUEST_TIMEOUT),
                            )
                        }
                        Err(e) => {
                            if let Err(e) =
                                response_tx.send(Err(e.clone().into()))
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
                                ConnectError::Cleanup => break,
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
                } else if let MaybeVerifiedRequest::Verified(
                    encrypted_id,
                    decrypted_id,
                    callback,
                ) = request
                {
                    // PARALLEL EXECUTION: Spawn attestation AND connect websocket at same time
                    let attestation_rx = spawn_attestation_fetch(
                        encrypted_id,
                        decrypted_id,
                        project_id.clone(),
                    );

                    // Start websocket connection immediately (parallel with attestation)
                    let connect_res = connect(
                        relay_url.clone(),
                        project_id.clone(),
                        &key,
                        topics,
                        None,
                        cleanup_rx.clone(),
                        probe_group.clone(),
                    )
                    .await;

                    match connect_res {
                        Ok((next_message_id, on_incomingmessage_rx, ws)) => {
                            // Websocket connected! Now wait for attestation
                            ConnectionState::ConnectRequestAwaitingAttestation(
                                next_message_id,
                                on_incomingmessage_rx,
                                ws,
                                callback,
                                attestation_rx,
                                response_tx,
                            )
                        }
                        Err(e) => {
                            if let Err(e) =
                                response_tx.send(Err(e.clone().into()))
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
                                ConnectError::Cleanup => break,
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
                } else {
                    // This should never happen - request should be either Unverified or Verified
                    tracing::error!(
                        "Unexpected request type in ConnectRequest handler"
                    );
                    ConnectionState::Idle
                };
                next_state
            }
            ConnectionState::AwaitingConnectRequestResponse(
                sent_message_id,
                next_message_id,
                mut on_incomingmessage_rx,
                ws,
                (response_tx,),
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
                                                sent_message_id,
                                                next_message_id,
                                                on_incomingmessage_rx,
                                                ws,
                                                (response_tx,),
                                                sleep,
                                            )
                                        }
                                        Payload::Response(response) => {
                                            if id == MessageId::new(sent_message_id.get()) {
                                                if let Err(e) =
                                                    response_tx.send(Ok(response))
                                                {
                                                    tracing::warn!("Failed to send response: {e:?}");
                                                }
                                                ConnectionState::Connected(next_message_id, on_incomingmessage_rx, ws)
                                            } else {
                                                tracing::warn!("ignoring message response in AwaitingSubscribeResponse state: {:?}", response);
                                                ConnectionState::AwaitingConnectRequestResponse(
                                                    sent_message_id,
                                                    next_message_id,
                                                    on_incomingmessage_rx,
                                                    ws,
                                                    (response_tx,),
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
            ConnectionState::ConnectRequestAwaitingAttestation(
                message_id,
                mut on_incomingmessage_rx,
                ws,
                callback,
                mut receiver,
                response_tx,
            ) => {
                // Websocket is already connected, waiting for attestation to arrive
                tracing::debug!(
                    "Waiting for attestation while websocket is connected"
                );

                // TODO avoid select! as it doesn't guarantee that all branches are covered (it will panic otherwise)
                tokio::select! {
                    Ok(attestation) = &mut receiver => {
                        tracing::debug!("Attestation received in ConnectRequestAwaitingAttestation state");
                        let params = callback(attestation);

                        // Prepare and send the request
                        let prepared = prepare_websocket_message(message_id, params)
                            .expect("Failed to serialize Params - this should never happen");

                        match send_prepared_message(&ws, &prepared) {
                            Ok(()) => {
                                tracing::debug!("Sent verified request with attestation, message_id={}", prepared.sent_message_id);
                                ConnectionState::AwaitingConnectRequestResponse(
                                    prepared.sent_message_id,
                                    prepared.next_message_id,
                                    on_incomingmessage_rx,
                                    ws,
                                    (response_tx,),
                                    crate::time::durable_sleep(REQUEST_TIMEOUT),
                                )
                            }
                            Err(e) => {
                                tracing::error!("Failed to send websocket message: {e:?}");
                                let _ = response_tx.send(Err(RequestError::Offline));
                                ConnectionState::MaybeReconnect(None)
                            }
                        }
                    }
                    message = on_incomingmessage_rx.recv() => {
                        if let Some(message) = message {
                            match message {
                                IncomingMessage::Close(reason) => {
                                    // Handle disconnect before attestation arrives
                                    match reason {
                                        CloseReason::InvalidAuth => {
                                            let _ = response_tx.send(Err(RequestError::InvalidAuth));
                                            ConnectionState::Poisoned
                                        }
                                        CloseReason::Error(e) => {
                                            tracing::debug!("Websocket closed while awaiting attestation: {e}");
                                            let _ = response_tx.send(Err(RequestError::Offline));
                                            ConnectionState::MaybeReconnect(None)
                                        }
                                    }
                                }
                                IncomingMessage::Message(_) => {
                                    // Unexpected message, stay in same state
                                    tracing::warn!("Unexpected message while awaiting attestation");
                                    ConnectionState::ConnectRequestAwaitingAttestation(
                                        message_id, on_incomingmessage_rx, ws, callback, receiver, response_tx
                                    )
                                }
                            }
                        } else {
                            let _ = response_tx.send(Err(RequestError::Offline));
                            ConnectionState::MaybeReconnect(None)
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
                    }
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
                            match request {
                                MaybeVerifiedRequest::Unverified(params) => {
                                    // Send immediately for unverified requests
                                    let prepared = prepare_websocket_message(message_id, params.clone())
                                        .expect("Failed to serialize Params - this should never happen");

                                    match send_prepared_message(&ws, &prepared) {
                                        Ok(()) => {
                                            ConnectionState::AwaitingRequestResponse(
                                                prepared.sent_message_id,
                                                prepared.next_message_id,
                                                on_incomingmessage_rx,
                                                ws,
                                                (params, response_tx),
                                                crate::time::durable_sleep(REQUEST_TIMEOUT),
                                            )
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to send websocket message: {e:?}");
                                            let _ = response_tx.send(Err(RequestError::Offline));
                                            ConnectionState::MaybeReconnect(None)
                                        }
                                    }
                                }
                                MaybeVerifiedRequest::Verified(encrypted_id, decrypted_id, callback) => {
                                    // Spawn attestation fetch and transition to awaiting state
                                    let attestation_rx = spawn_attestation_fetch(encrypted_id, decrypted_id, project_id.clone());

                                    // Transition to awaiting attestation (will create params once it arrives)
                                    ConnectionState::ConnectedAwaitingAttestation(
                                        message_id,
                                        on_incomingmessage_rx,
                                        ws,
                                        callback,
                                        attestation_rx,
                                        response_tx,
                                    )
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
                            tracing::debug!("sending websocket message: {serialized}");
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
            ConnectionState::ConnectedAwaitingAttestation(
                message_id,
                mut on_incomingmessage_rx,
                ws,
                callback,
                mut attestation_receiver,
                response_tx,
            ) => {
                // Already connected, waiting for attestation before sending request
                tracing::debug!("Waiting for attestation in Connected state");

                tokio::select! {
                    Ok(attestation) = &mut attestation_receiver => {
                        tracing::debug!("Attestation received in ConnectedAwaitingAttestation state");
                        let params = callback(attestation);

                        // Prepare and send the request with attestation
                        let prepared = prepare_websocket_message(message_id, params.clone())
                            .expect("Failed to serialize Params - this should never happen");

                        match send_prepared_message(&ws, &prepared) {
                            Ok(()) => {
                                tracing::debug!("Sent verified request with attestation, message_id={}", prepared.sent_message_id);
                                ConnectionState::AwaitingRequestResponse(
                                    prepared.sent_message_id,
                                    prepared.next_message_id,
                                    on_incomingmessage_rx,
                                    ws,
                                    (params, response_tx),
                                    crate::time::durable_sleep(REQUEST_TIMEOUT),
                                )
                            }
                            Err(e) => {
                                tracing::error!("Failed to send websocket message: {e:?}");
                                let _ = response_tx.send(Err(RequestError::Offline));
                                ConnectionState::MaybeReconnect(None)
                            }
                        }
                    }
                    message = on_incomingmessage_rx.recv() => {
                        if let Some(message) = message {
                            match message {
                                IncomingMessage::Close(reason) => {
                                    // Handle disconnect before attestation arrives
                                    match reason {
                                        CloseReason::InvalidAuth => {
                                            let _ = response_tx.send(Err(RequestError::InvalidAuth));
                                            ConnectionState::Poisoned
                                        }
                                        CloseReason::Error(e) => {
                                            tracing::debug!("Websocket closed while awaiting attestation in Connected state: {e}");
                                            let _ = response_tx.send(Err(RequestError::Offline));
                                            ConnectionState::MaybeReconnect(None)
                                        }
                                    }
                                }
                                IncomingMessage::Message(_) => {
                                    // Unexpected message, stay in same state
                                    tracing::warn!("Unexpected message while awaiting attestation in Connected state");
                                    ConnectionState::ConnectedAwaitingAttestation(
                                        message_id, on_incomingmessage_rx, ws, callback, attestation_receiver, response_tx
                                    )
                                }
                            }
                        } else {
                            let _ = response_tx.send(Err(RequestError::Offline));
                            ConnectionState::MaybeReconnect(None)
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
                sent_message_id,
                next_message_id,
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
                                                sent_message_id,
                                                next_message_id,
                                                on_incomingmessage_rx,
                                                ws,
                                                (request, response_tx),
                                                sleep,
                                            )
                                        }
                                        Payload::Response(response) => {
                                            if id == MessageId::new(sent_message_id.get()) {
                                                if let Err(e) =
                                                    response_tx.send(Ok(response))
                                                {
                                                    tracing::warn!("Failed to send response in AwaitingRequestResponse state: {e:?}");
                                                }
                                                ConnectionState::Connected(next_message_id, on_incomingmessage_rx, ws)
                                            } else {
                                                tracing::warn!("ignoring message response in AwaitingSubscribeResponse state: {:?}", response);
                                                ConnectionState::AwaitingRequestResponse(
                                                    sent_message_id,
                                                    next_message_id,
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

                // Prepare the message first (allocates ID)
                let prepared = prepare_websocket_message(
                    NextMessageId::new(MIN_RPC_ID),
                    request.clone(),
                )
                .expect(
                    "Failed to serialize Params - this should never happen",
                );

                let connect_fut = connect(
                    relay_url.clone(),
                    project_id.clone(),
                    &key,
                    topics,
                    Some(prepared.clone()),
                    cleanup_rx.clone(),
                    probe_group.clone(),
                );
                tokio::select! {
                    connect_res = connect_fut => {
                        match connect_res {
                            Ok((next_message_id, on_incomingmessage_rx, ws)) => {
                                ConnectionState::AwaitingConnectRetryRequestResponse(
                                    prepared.sent_message_id,
                                    next_message_id,
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
                sent_message_id,
                next_message_id,
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
                                                sent_message_id,
                                                next_message_id,
                                                on_incomingmessage_rx,
                                                ws,
                                                response_tx,
                                                sleep,
                                            )
                                        }
                                        Payload::Response(response) => {
                                            if id == MessageId::new(sent_message_id.get()) {
                                                if let Err(e) =
                                                    response_tx.send(Ok(response))
                                                {
                                                    tracing::warn!("Failed to send response: {e:?}");
                                                }
                                                ConnectionState::Connected(next_message_id, on_incomingmessage_rx, ws)
                                            } else {
                                                tracing::warn!("ignoring message response in AwaitingConnectRetryRequestResponse state: {:?}", response);
                                                ConnectionState::AwaitingConnectRetryRequestResponse(
                                                    sent_message_id,
                                                    next_message_id,
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
