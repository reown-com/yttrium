use crate::sign::envelope_type0::{encode_envelope_type0, EnvelopeType0};
use crate::sign::protocol_types::{
    Controller, Metadata, Proposal, ProposalNamespaces, ProposalResponse,
    Relay, SessionSettle, SettleNamespace,
};
use crate::sign::relay_url::ConnectionOptions;
use crate::sign::utils::{diffie_hellman, topic_from_sym_key};
use alloy::rpc::json_rpc::{self, Id, ResponsePayload};
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{AeadCore, ChaCha20Poly1305, KeyInit, Nonce};
use data_encoding::BASE64;
use relay_rpc::auth::ed25519_dalek::{Signer, SigningKey};
use relay_rpc::domain::{DecodedClientId, Topic};
use relay_rpc::jwt::{JwtBasicClaims, JwtHeader};
use relay_rpc::rpc::{FetchMessages, FetchResponse, Params, Subscription};
use relay_rpc::{
    domain::{MessageId, ProjectId},
    rpc::{ApproveSession, Payload, Request, Response},
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::debug;
use x25519_dalek::PublicKey;
#[cfg(not(target_arch = "wasm32"))]
use {
    futures::{SinkExt, StreamExt},
    tokio_tungstenite::{connect_async, tungstenite::Message},
};

const RELAY_URL: &str = "wss://relay.walletconnect.org";

mod envelope_type0;
mod envelope_type1;
mod pairing_uri;
pub mod protocol_types;
mod relay_url;
#[cfg(test)]
mod tests;
mod utils;

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Error))]
#[error("Sign request error: {0}")]
pub enum RequestError {
    #[error("Internal: {0}")]
    Internal(String),

    #[error("Should never happen: {0}")]
    ShouldNeverHappen(String),
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Error))]
#[error("Sign next error: {0}")]
pub enum NextError {
    #[error("Internal: {0}")]
    Internal(String),
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Error))]
#[error("Sign pair error: {0}")]
pub enum PairError {
    #[error("Request error: {0}")]
    Request(RequestError),

    #[error("Internal: {0}")]
    Internal(String),

    #[error("Should never happen: {0}")]
    ShouldNeverHappen(String),
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Error))]
#[error("Sign approve error: {0}")]
pub enum ApproveError {
    #[error("Request error: {0}")]
    Request(RequestError),

    #[error("Internal: {0}")]
    Internal(String),

    #[error("Should never happen: {0}")]
    ShouldNeverHappen(String),
}

// #[cfg(target_arch = "wasm32")]
// struct WebWebSocketWrapper {
//     // rx: tokio::sync::mpsc::UnboundedReceiver<String>,
//     // tx: tokio::sync::mpsc::UnboundedSender<String>,
// }

struct WebSocketState {
    // #[cfg(not(target_arch = "wasm32"))]
    // stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    // #[cfg(target_arch = "wasm32")]
    // stream: WebWebSocketWrapper,
    // message_id: u64,
    request_tx: tokio::sync::mpsc::UnboundedSender<(
        Params,
        tokio::sync::oneshot::Sender<Response>,
    )>,
}

#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Object))]
pub struct Client {
    relay_url: String,
    project_id: ProjectId,
    websocket: Option<WebSocketState>,
    session_request_tx: tokio::sync::mpsc::UnboundedSender<String>,
    sessions: Arc<RwLock<HashMap<Topic, ApprovedSession>>>,
}

// TODO
// - separate app from wallet client
// - incoming message deduplication (RPC ID/hash)
// - random (?) request RPC ID generation
// - session expiry
// - WS reconnection & retries
//   - disconnect if no ping for 30s etc.
// - interpret relay disconnect reason
// - subscribe/fetch messages on startup - also solve that ordering problem?
// - handle connection error
// - handle project ID/JWT error

#[allow(unused)]
impl Client {
    pub fn new(
        project_id: ProjectId,
    ) -> (Self, tokio::sync::mpsc::UnboundedReceiver<String>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        (
            Self {
                relay_url: RELAY_URL.to_string(),
                project_id,
                websocket: None,
                session_request_tx: tx,
                sessions: Arc::new(RwLock::new(HashMap::new())),
            },
            rx,
        )
    }

    pub fn add_sessions(
        &self,
        sessions: impl IntoIterator<Item = ApprovedSession>,
    ) {
        let mut guard = self.sessions.write().unwrap();
        for session in sessions {
            guard.insert(topic_from_sym_key(&session.session_sym_key), session);
        }
    }

    pub fn get_sessions(&self) -> Vec<ApprovedSession> {
        let guard = self.sessions.read().unwrap();
        guard.values().cloned().collect()
    }

    /// Call this when the app and user are ready to receive session requests.
    /// Skip calling this if you intend to shortly call another SDK method, as those other methods will themselves call this.
    pub fn online(&self) {
        if !self.sessions.read().unwrap().is_empty() {
            // TODO request w/ empty request
            // the WS "layer" will add the irn_batchSubscribe automatically (as it does for all things)

            // ~~TODO actually:~~
            // Don't call irn_batchSubscribe or batchFetch at all. Do this automatically when the app calls another methods e.g. wc_approveSession
            // hmm actually, currently the relay doesn't know then when to expire an individual topic. Let's revisit this once the relay keeps track of sessions
            // for now, pass the session topics via the `subscribeTopics` param of wc_approveSession etc.
        }
    }

    pub async fn _connect(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L220
        unimplemented!()

        // TODO call `wc_proposeSession`
    }

    pub async fn pair(
        &mut self,
        uri: &str,
    ) -> Result<SessionProposal, PairError> {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L330

        // TODO parse URI
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/core/src/controllers/pairing.ts#L132
        let pairing_uri = pairing_uri::parse(uri)
            .map_err(|e| PairError::Internal(e.to_string()))?;

        // TODO immediately throw if expired - maybe not necessary if FetchMessages returns empty array?
        // note: no activatePairing
        // TODO save symkey, if necessary

        // TODO update relay method to not remove message

        let response = self
            .request::<FetchResponse>(relay_rpc::rpc::Params::FetchMessages(
                FetchMessages { topic: pairing_uri.topic.clone() },
            ))
            .await
            .map_err(|e| PairError::Internal(e.to_string()))?;

        for message in response.messages {
            if message.topic == pairing_uri.topic {
                let decoded =
                    BASE64.decode(message.message.as_bytes()).map_err(|e| {
                        PairError::Internal(format!(
                            "Failed to decode message: {e}"
                        ))
                    })?;
                let envelope =
                    envelope_type0::deserialize_envelope_type0(&decoded)
                        .map_err(|e| PairError::Internal(e.to_string()))?;
                let key = ChaCha20Poly1305::new(&pairing_uri.sym_key.into());
                let decrypted = key
                    .decrypt(&Nonce::from(envelope.iv), envelope.sb.as_slice())
                    .map_err(|e| PairError::Internal(e.to_string()))?;
                let request = serde_json::from_slice::<
                    json_rpc::Request<serde_json::Value>,
                >(&decrypted)
                .map_err(|e| {
                    PairError::Internal(format!(
                        "Failed to parse decrypted message: {e}"
                    ))
                })?;
                if request.meta.method != "wc_sessionPropose" {
                    return Err(PairError::Internal(format!(
                        "Expected wc_sessionPropose, got {}",
                        request.meta.method
                    )));
                }
                println!("rpc request: {}", request.meta.id);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&request.params).unwrap()
                );
                let proposal =
                    serde_json::from_value::<Proposal>(request.params)
                        .map_err(|e| PairError::Internal(e.to_string()))?;
                println!("{:?}", proposal);

                let proposer_public_key = hex::decode(proposal.proposer.public_key)
                    .map_err(|e| {
                        PairError::Internal(format!(
                            "Failed to decode proposer public key: {e}"
                        ))
                    })?
                    .try_into()
                    .map_err(|_| {
                        PairError::Internal(format!(
                            "Failed to convert proposer public key to fixed-size array"
                        ))
                    })?;
                println!("pairing topic: {}", proposal.pairing_topic);

                // TODO validate namespaces: https://specs.walletconnect.com/2.0/specs/clients/sign/namespaces#12-proposal-namespaces-must-not-have-chains-empty

                return Ok(SessionProposal {
                    session_proposal_rpc_id: request.meta.id,
                    pairing_topic: proposal.pairing_topic,
                    pairing_sym_key: pairing_uri.sym_key,
                    proposer_public_key,
                    requested_namespaces: proposal
                        .required_namespaces
                        .into_iter()
                        .chain(proposal.optional_namespaces.into_iter())
                        .collect(),
                });
            }
        }

        Err(PairError::Internal("No message found".to_string()))
    }

    pub async fn approve(
        &mut self,
        pairing: SessionProposal,
    ) -> Result<ApprovedSession, ApproveError> {
        // TODO params:
        // - approvedNamespaces, etc.

        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L341

        // TODO check is valid

        let mut namespaces = HashMap::new();
        for (namespace, namespace_proposal) in pairing.requested_namespaces {
            let accounts = namespace_proposal
                .chains
                .iter()
                .map(|chain| {
                    format!(
                        "{}:{}",
                        chain, "0x0000000000000000000000000000000000000000"
                    )
                })
                .collect();
            let namespace_settle = SettleNamespace {
                accounts,
                methods: namespace_proposal.methods,
                events: namespace_proposal.events,
            };
            namespaces.insert(namespace, namespace_settle);
        }
        debug!("namespaces: {:?}", namespaces);

        let self_key = x25519_dalek::StaticSecret::random();
        let self_public_key = PublicKey::from(&self_key);
        let shared_secret =
            diffie_hellman(&pairing.proposer_public_key.into(), &self_key);
        let session_topic = topic_from_sym_key(&shared_secret);
        debug!("session topic: {}", session_topic);

        let session_proposal_response = {
            let serialized =
                serde_json::to_string(&alloy::rpc::json_rpc::Response {
                    id: pairing.session_proposal_rpc_id,
                    payload: ResponsePayload::Success(ProposalResponse {
                        relay: Relay { protocol: "irn".to_string() },
                        responder_public_key: hex::encode(
                            self_public_key.to_bytes(),
                        ),
                    }) as ResponsePayload<_, ()>,
                })
                .map_err(|e| ApproveError::Internal(e.to_string()))?;

            let key = ChaCha20Poly1305::new(&pairing.pairing_sym_key.into());
            let nonce = ChaCha20Poly1305::generate_nonce()
                .map_err(|e| ApproveError::Internal(e.to_string()))?;
            let encrypted = key
                .encrypt(&nonce, serialized.as_bytes())
                .map_err(|e| ApproveError::Internal(e.to_string()))?;
            let encoded = encode_envelope_type0(&EnvelopeType0 {
                iv: nonce.into(),
                sb: encrypted,
            })
            .map_err(|e| ApproveError::Internal(e.to_string()))?;
            BASE64.encode(encoded.as_slice()).into()
        };

        let session_settlement_request = {
            let serialized =
                serde_json::to_string(&alloy::rpc::json_rpc::Request::new(
                    "wc_sessionSettle",
                    1000000010.into(), // TODO generate this
                    SessionSettle {
                        relay: Relay { protocol: "irn".to_string() },
                        namespaces,
                        controller: Controller {
                            public_key: hex::encode(self_public_key.to_bytes()),
                            metadata: Metadata {
                                name: "Reown".to_string(),
                                description: "Reown".to_string(),
                                url: "https://reown.com".to_string(),
                                icons: vec![
                                    "https://reown.com/icon.png".to_string()
                                ],
                            },
                        },
                        expiry_timestamp: crate::time::SystemTime::now()
                            .duration_since(crate::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                            + 60 * 60 * 24 * 30,
                        session_properties: serde_json::Value::Null,
                        scoped_properties: serde_json::Value::Null,
                        session_config: serde_json::Value::Null,
                    },
                ))
                .map_err(|e| ApproveError::Internal(e.to_string()))?;

            let key = ChaCha20Poly1305::new(&shared_secret.into());
            let nonce = ChaCha20Poly1305::generate_nonce()
                .map_err(|e| ApproveError::Internal(e.to_string()))?;
            let encrypted = key
                .encrypt(&nonce, serialized.as_bytes())
                .map_err(|e| ApproveError::Internal(e.to_string()))?;
            let encoded = encode_envelope_type0(&EnvelopeType0 {
                iv: nonce.into(),
                sb: encrypted,
            })
            .map_err(|e| ApproveError::Internal(e.to_string()))?;
            BASE64.encode(encoded.as_slice()).into()
        };

        let approve_session = ApproveSession {
            pairing_topic: pairing.pairing_topic,
            session_topic: session_topic.clone(),
            session_proposal_response,
            session_settlement_request,
            analytics: None,
        };

        // TODO insert session into storage

        match self
            .request::<bool>(relay_rpc::rpc::Params::ApproveSession(
                approve_session,
            ))
            .await
        {
            Ok(true) => {}
            Ok(false) => {
                // TODO remove from storage
                return Err(ApproveError::Internal(format!(
                    "Session rejected by relay"
                )));
            }
            Err(e) => {
                // TODO if error, remove from storage
                // https://github.com/reown-com/reown-kotlin/blob/1488873e0ac655bdc492ab12d8ea29b9985dd97c/protocol/sign/src/main/kotlin/com/reown/sign/engine/use_case/calls/ApproveSessionUseCase.kt#L115
                // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L476
                // consistency: ok if wallet thinks session is approved, but app never received approval
                // ideally one way we fix this via relay source of truth

                return Err(ApproveError::Request(e));
            }
        }

        let session = ApprovedSession { session_sym_key: shared_secret };
        {
            let mut sessions = self.sessions.write().unwrap();
            sessions.insert(session_topic, session.clone());
        }

        Ok(session)
    }

    pub async fn _reject(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L497
        unimplemented!()

        // TODO consider new relay method?
    }

    pub async fn _update(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L528
        unimplemented!()
    }

    pub async fn _extend(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L569
        unimplemented!()
    }

    pub async fn _request(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L601
        unimplemented!()
    }

    pub async fn _respond(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L701
        unimplemented!()
    }

    pub async fn _ping(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L727
        unimplemented!()
    }

    pub async fn _emit(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L764
        unimplemented!()
    }

    pub async fn _disconnect(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L781
        unimplemented!()
    }

    pub async fn _authenticate(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L817
        unimplemented!()
    }

    pub async fn _approve_session_authenticate(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L1123
        unimplemented!()
    }

    pub async fn _reject_session_authenticate(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L1298
        unimplemented!()
    }

    async fn request<T: DeserializeOwned>(
        &mut self,
        params: relay_rpc::rpc::Params,
    ) -> Result<T, RequestError> {
        let mut ws_state = if let Some(ws_state) = self.websocket.as_mut() {
            ws_state
        } else {
            let key = SigningKey::generate(&mut rand::thread_rng());

            let url = {
                let encoder = &data_encoding::BASE64URL_NOPAD;
                let claims = {
                    let data = JwtBasicClaims {
                        iss: DecodedClientId::from_key(&key.verifying_key())
                            .into(),
                        sub: "http://example.com".to_owned(),
                        aud: self.relay_url.clone(),
                        iat: crate::time::SystemTime::now()
                            .duration_since(crate::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64,
                        exp: Some(
                            crate::time::SystemTime::now()
                                .duration_since(crate::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs() as i64
                                + 60 * 60,
                        ),
                    };

                    encoder.encode(
                        serde_json::to_string(&data).unwrap().as_bytes(),
                    )
                };
                let header = encoder.encode(
                    serde_json::to_string(&JwtHeader::default())
                        .unwrap()
                        .as_bytes(),
                );
                let message = format!("{header}.{claims}");
                let signature = {
                    let data = key.sign(message.as_bytes());
                    encoder.encode(&data.to_bytes())
                };
                let auth = format!("{message}.{signature}");

                let conn_opts =
                    ConnectionOptions::new(self.project_id.clone(), auth)
                        .with_address(&self.relay_url);
                conn_opts.as_url().unwrap().to_string()
            };

            let session_request_tx = self.session_request_tx.clone();
            let (request_tx, mut request_rx) =
                tokio::sync::mpsc::unbounded_channel();

            let handle_session_request = {
                let sessions = self.sessions.clone();
                move |sub_msg: Subscription| {
                    let session_sym_key = {
                        let sessions = sessions.read().unwrap();
                        let session =
                            sessions.get(&sub_msg.data.topic).unwrap();
                        session.session_sym_key
                    };

                    let decoded = BASE64
                        .decode(sub_msg.data.message.as_bytes())
                        .map_err(|e| {
                            PairError::Internal(format!(
                                "Failed to decode message: {e}"
                            ))
                        })
                        .unwrap();
                    let envelope =
                        envelope_type0::deserialize_envelope_type0(&decoded)
                            .map_err(|e| PairError::Internal(e.to_string()))
                            .unwrap();
                    let key = ChaCha20Poly1305::new(&session_sym_key.into());
                    let decrypted = key
                        .decrypt(
                            &Nonce::from(envelope.iv),
                            envelope.sb.as_slice(),
                        )
                        .map_err(|e| PairError::Internal(e.to_string()))
                        .unwrap();
                    // let request = serde_json::from_slice::<
                    //     json_rpc::Request<serde_json::Value>,
                    // >(&decrypted)
                    // .map_err(|e| {
                    //     PairError::Internal(format!(
                    //         "Failed to parse decrypted message: {e}"
                    //     ))
                    // })?;

                    session_request_tx
                        .send(String::from_utf8(decrypted).unwrap())
                        .unwrap();
                }
            };

            #[cfg(not(target_arch = "wasm32"))]
            let ws_stream = {
                let (mut ws_stream, _response) = connect_async(url)
                    .await
                    .map_err(|e| RequestError::Internal(e.to_string()))?;
                let sessions = self.sessions.clone();

                crate::spawn::spawn(async move {
                    const MIN: u64 = 1000000000; // MessageId::MIN is private
                    let mut message_id = MIN;
                    let mut response_channels = HashMap::<
                        _,
                        tokio::sync::oneshot::Sender<Response>,
                    >::new();

                    loop {
                        tokio::select! {
                            Some((params, response_tx)) = request_rx.recv() => {
                                message_id += 1;
                                response_channels.insert(MessageId::new(message_id), response_tx);
                                let request = Payload::Request(Request::new(MessageId::new(message_id), params));
                                let serialized = serde_json::to_string(&request).map_err(|e| {
                                    RequestError::ShouldNeverHappen(format!(
                                        "Failed to serialize request: {e}"
                                    ))
                                }).unwrap();
                                ws_stream.send(Message::Text(serialized.into())).await.unwrap();
                            }
                            Some(message) = ws_stream.next() => {
                                let n = message
                                    .map_err(|e| {
                                        RequestError::Internal(format!(
                                            "WebSocket stream error: {e}"
                                        ))
                                    })
                                    .expect("WebSocket stream error");
                                match n {
                                    Message::Text(message) => {
                                        let payload =
                                            serde_json::from_str::<Payload>(&message)
                                                .map_err(|e| {
                                                    RequestError::Internal(format!(
                                                        "Failed to parse payload: {e}"
                                                    ))
                                                });
                                        match payload {
                                            Ok(payload) => match payload {
                                                Payload::Request(request) => {
                                                    match request.params {
                                                        Params::Subscription(
                                                            sub_msg
                                                        ) => {
                                                            handle_session_request(sub_msg);
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                                Payload::Response(response) => {
                                                    if let Some(response_tx) =
                                                        response_channels
                                                            .remove(&response.id())
                                                    {
                                                        if let Err(e) =
                                                            response_tx.send(response)
                                                        {
                                                            web_sys::console::log_1(&format!("Failed to send response: {e:?}").into());
                                                        }
                                                    }
                                                }
                                            },
                                            Err(e) => {
                                                // no-op
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                });
            };

            #[cfg(target_arch = "wasm32")]
            let ws_stream = {
                use wasm_bindgen::{prelude::Closure, JsCast};
                use web_sys::{Event, MessageEvent};

                let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
                let (tx_send, mut rx_send) =
                    tokio::sync::mpsc::unbounded_channel::<String>();
                wasm_bindgen_futures::spawn_local(async move {
                    let ws = web_sys::WebSocket::new(&url)
                        .map_err(|e| {
                            RequestError::Internal(format!(
                                "Failed to create WebSocket: {e:?}"
                            ))
                        })
                        .unwrap();
                    let (tx_open, mut rx_open) = tokio::sync::mpsc::channel(1);
                    let tx_open =
                        std::sync::Arc::new(tokio::sync::Mutex::new(tx_open));
                    let tx_open_clone = tx_open.clone();
                    let onopen_closure =
                        Closure::wrap(Box::new(move |_event: Event| {
                            let tx_open = tx_open_clone.clone();
                            crate::spawn::spawn(async move {
                                let tx = tx_open.lock().await;
                                tx.send(()).await.unwrap();
                                // TODO unmount handler once inited once
                            });
                        })
                            as Box<dyn Fn(Event)>);
                    ws.set_onopen(Some(
                        Box::leak(Box::new(onopen_closure))
                            .as_ref()
                            .unchecked_ref(),
                    ));
                    let onmessage_closure =
                        Closure::wrap(Box::new(move |event: MessageEvent| {
                            // TODO may not be string messages
                            let message = event.data().as_string().unwrap();
                            web_sys::console::log_1(&message.clone().into());
                            tx.send(message.clone()).unwrap();
                        })
                            as Box<dyn Fn(MessageEvent)>);
                    ws.set_onmessage(Some(
                        Box::leak(Box::new(onmessage_closure))
                            .as_ref()
                            .unchecked_ref(),
                    ));
                    web_sys::console::log_1(&"onopen".into());
                    rx_open.recv().await.unwrap();
                    ws.set_onopen(None);
                    web_sys::console::log_1(&"onopen received".into());

                    const MIN: u64 = 1000000000; // MessageId::MIN is private
                    let mut message_id = MIN;
                    let mut response_channels = HashMap::<
                        _,
                        tokio::sync::oneshot::Sender<Response>,
                    >::new();

                    loop {
                        tokio::select! {
                            Some((params, response_tx)) = request_rx.recv() => {
                                message_id += 1;
                                response_channels.insert(MessageId::new(message_id), response_tx);
                                let request = Payload::Request(Request::new(MessageId::new(message_id), params));
                                let serialized = serde_json::to_string(&request).map_err(|e| {
                                    RequestError::ShouldNeverHappen(format!(
                                        "Failed to serialize request: {e}"
                                    ))
                                }).unwrap();
                                ws.send_with_str(&serialized).unwrap();
                            }
                            Some(message) = rx.recv() => {
                                let payload = serde_json::from_str::<Payload>(&message).map_err(|e| {
                                    RequestError::Internal(format!(
                                        "Failed to parse payload: {e}"
                                    ))
                                });
                                match payload {
                                    Ok(payload) => {
                                        match payload {
                                            Payload::Request(request) => {
                                                match request.params {
                                                    Params::Subscription(
                                                        sub_msg
                                                    ) => {
                                                        handle_session_request(sub_msg);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            Payload::Response(response) => {
                                                if let Some(response_tx) = response_channels.remove(&response.id()) {
                                                    if let Err(e) = response_tx.send(response) {
                                                        web_sys::console::log_1(&format!("Failed to send response: {e:?}").into());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        // no-op
                                    }
                                }
                            }
                        }
                    }
                });
                // WebWebSocketWrapper {}
            };

            self.websocket.insert(WebSocketState {
                // stream: ws_stream,
                // message_id: MIN,
                // session_request_rx,
                request_tx,
            })
        };

        let (response_tx, response_rx) = tokio::sync::oneshot::channel();
        ws_state.request_tx.send((params, response_tx)).unwrap();
        let response = response_rx.await.unwrap();

        match response {
            Response::Success(response) => {
                Ok(serde_json::from_value(response.result).map_err(|e| {
                    RequestError::Internal(format!(
                        "Failed to parse response result: {e}"
                    ))
                })?)
            }
            Response::Error(response) => Err(RequestError::Internal(format!(
                "RPC error: {:?}",
                response.error
            ))),
        }
    }
}

// UniFFI wrapper for better API naming
#[cfg(feature = "uniffi")]
#[derive(uniffi::Object)]
pub struct SignClient {
    client: std::sync::Arc<tokio::sync::Mutex<Client>>,
}

#[cfg(feature = "uniffi")]
#[uniffi::export(async_runtime = "tokio")]
impl SignClient {
    #[uniffi::constructor]
    pub fn new(project_id: String) -> Self {
        let client = Client::new(ProjectId::from(project_id));
        Self { client: std::sync::Arc::new(tokio::sync::Mutex::new(client.0)) }
    }

    pub async fn pair(
        &self,
        uri: String,
    ) -> Result<SessionProposalFfi, PairError> {
        let proposal = {
            let mut client = self.client.lock().await;
            client.pair(&uri).await?
        };
        Ok(proposal.into())
    }

    pub async fn approve(
        &self,
        pairing: SessionProposalFfi,
    ) -> Result<ApprovedSessionFfi, ApproveError> {
        let proposal: SessionProposal = pairing.into();
        let session = {
            let mut client = self.client.lock().await;
            client.approve(proposal).await?
        };
        Ok(session.into())
    }
}

#[derive(Debug)]
pub struct SessionProposal {
    pub session_proposal_rpc_id: Id,
    pub pairing_topic: Topic,
    pub pairing_sym_key: [u8; 32],
    pub proposer_public_key: [u8; 32],
    pub requested_namespaces: ProposalNamespaces,
}

#[cfg(feature = "uniffi")]
#[derive(uniffi_macros::Record, Debug)]
pub struct SessionProposalFfi {
    pub id: String,
    pub topic: String,
    pub pairing_sym_key: Vec<u8>,
    pub proposer_public_key: Vec<u8>,
    pub requested_namespaces: std::collections::HashMap<
        String,
        crate::sign::protocol_types::ProposalNamespace,
    >,
}

#[cfg(feature = "uniffi")]
impl From<SessionProposal> for SessionProposalFfi {
    fn from(proposal: SessionProposal) -> Self {
        // Ensure both id and topic are properly converted to valid UTF-8 strings
        let id_string = match &proposal.session_proposal_rpc_id {
            Id::Number(num) => num.to_string(),
            Id::String(s) => s.clone(),
            Id::None => "null".to_string(),
        };
        
        // Be extremely defensive about topic string conversion
        let topic_string = {
            let raw_string = if let Ok(serialized) = serde_json::to_string(&proposal.pairing_topic) {
                // Remove quotes from JSON string
                serialized.trim_matches('"').to_string()
            } else {
                // Fallback to display format
                format!("{}", proposal.pairing_topic)
            };
            
            // Ensure the string is valid UTF-8 and only contains safe ASCII characters
            if raw_string.is_ascii() && raw_string.chars().all(|c| c.is_ascii_alphanumeric()) {
                raw_string
            } else {
                // If anything looks suspicious, force it to be safe ASCII hex
                // This is a defensive fallback that should never be needed
                format!("fallback_{}", hex::encode(raw_string.as_bytes()))
            }
        };
        
        Self {
            id: id_string,
            topic: topic_string,
            pairing_sym_key: proposal.pairing_sym_key.to_vec(),
            proposer_public_key: proposal.proposer_public_key.to_vec(),
            requested_namespaces: proposal.requested_namespaces,
        }
    }
}

#[cfg(feature = "uniffi")]
impl From<SessionProposalFfi> for SessionProposal {
    fn from(proposal: SessionProposalFfi) -> Self {
        use alloy::rpc::json_rpc::Id;
        let id = if let Ok(num) = proposal.id.parse::<u64>() {
            Id::Number(num)
        } else {
            Id::String(proposal.id)
        };
        // Ensure topic is properly converted from string
        let topic = Topic::from(proposal.topic);
        Self {
            session_proposal_rpc_id: id,
            pairing_topic: topic,
            pairing_sym_key: proposal.pairing_sym_key.try_into().unwrap(),
            proposer_public_key: proposal
                .proposer_public_key
                .try_into()
                .unwrap(),
            requested_namespaces: proposal.requested_namespaces,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ApprovedSession {
    pub session_sym_key: [u8; 32],
}

#[cfg(feature = "uniffi")]
#[derive(uniffi_macros::Record)]
pub struct ApprovedSessionFfi {
    pub session_sym_key: Vec<u8>,
}

#[cfg(feature = "uniffi")]
impl From<ApprovedSession> for ApprovedSessionFfi {
    fn from(session: ApprovedSession) -> Self {
        Self { session_sym_key: session.session_sym_key.to_vec() }
    }
}

#[cfg(feature = "uniffi")]
impl From<ApprovedSessionFfi> for ApprovedSession {
    fn from(session: ApprovedSessionFfi) -> Self {
        Self { session_sym_key: session.session_sym_key.try_into().unwrap() }
    }
}

#[cfg(test)]
mod conversion_tests {
    use super::*;
    use alloy::rpc::json_rpc::Id;

    #[test]
    fn test_session_proposal_conversion() {
        // Create a test SessionProposal with known values
        let test_topic = Topic::from(
            "0c814f7d2d56c0e840f75612addaa170af479b1c8499632430b41c298bf49907"
                .to_string(),
        );
        let test_id = Id::Number(1234567890);

        let session_proposal = SessionProposal {
            session_proposal_rpc_id: test_id,
            pairing_topic: test_topic.clone(),
            pairing_sym_key: [1u8; 32],
            proposer_public_key: [2u8; 32],
            requested_namespaces: std::collections::HashMap::new(),
        };

        // Convert to FFI
        let ffi_proposal: SessionProposalFfi = session_proposal.into();

        // Print the actual values to see what we get
        println!("Original topic: {:?}", test_topic);
        println!("Topic Display: {}", test_topic);
        println!("Topic Debug: {:?}", test_topic);
        println!("Topic JSON: {:?}", serde_json::to_string(&test_topic));

        println!("FFI id: {}", ffi_proposal.id);
        println!("FFI topic: {}", ffi_proposal.topic);
        println!("FFI topic bytes: {:?}", ffi_proposal.topic.as_bytes());
        println!("FFI topic len: {}", ffi_proposal.topic.len());

        // Check if the values are reasonable
        assert_eq!(ffi_proposal.id, "1234567890");
        assert!(!ffi_proposal.topic.is_empty(), "Topic should not be empty");
        assert!(ffi_proposal.topic.is_ascii(), "Topic should be ASCII");

        // The topic should be a hex string
        if ffi_proposal.topic.len() == 64 {
            assert!(
                ffi_proposal.topic.chars().all(|c| c.is_ascii_hexdigit()),
                "Topic should be a hex string, got: {}",
                ffi_proposal.topic
            );
        }
    }
}
