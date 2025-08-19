#[cfg(feature = "uniffi")]
use crate::sign::ffi_types::{
    SessionFfi, SessionProposalFfi, SessionRequestJsonRpcFfi,
    SessionRequestResponseJsonRpcFfi,
};
pub use relay_rpc::{
    auth::ed25519_dalek::{SecretKey, SigningKey},
    domain::Topic,
};
use {
    crate::sign::{
        envelope_type0::{encode_envelope_type0, EnvelopeType0},
        protocol_types::{
            Controller, Metadata, ProposalJsonRpc, ProposalNamespaces,
            ProposalResponse, ProposalResponseJsonRpc, Relay,
            SessionRequestJsonRpc, SessionRequestResponseJsonRpc,
            SessionSettle, SessionSettleJsonRpc, SettleNamespace,
        },
        relay_url::ConnectionOptions,
        utils::{diffie_hellman, generate_rpc_id, topic_from_sym_key},
    },
    chacha20poly1305::{
        aead::Aead, AeadCore, ChaCha20Poly1305, KeyInit, Nonce,
    },
    data_encoding::BASE64,
    relay_rpc::{
        auth::ed25519_dalek::Signer,
        domain::{DecodedClientId, MessageId, ProjectId},
        jwt::{JwtBasicClaims, JwtHeader},
        rpc::{
            AnalyticsData, ApproveSession, BatchSubscribe, FetchMessages,
            FetchResponse, Params, Payload, Publish, Request, Response,
            Subscription, SuccessfulResponse,
        },
    },
    serde::{de::DeserializeOwned, Deserialize, Serialize},
    std::{collections::HashMap, sync::Arc, time::Duration},
    tracing::debug,
    x25519_dalek::PublicKey,
};
#[cfg(not(target_arch = "wasm32"))]
use {
    futures::{SinkExt, StreamExt},
    tokio_tungstenite::{connect_async, tungstenite::Message},
};

const RELAY_URL: &str = "wss://relay.walletconnect.org";
const MIN_RPC_ID: u64 = 1000000000; // MessageId::MIN is private

mod envelope_type0;
mod envelope_type1;
#[cfg(feature = "uniffi")]
mod ffi_types;
#[cfg(feature = "uniffi")]
mod mapper;
mod pairing_uri;
pub mod protocol_types;
mod relay_url;
#[cfg(test)]
mod tests;
mod utils;

#[derive(Debug, thiserror::Error, Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Error))]
#[error("Sign request error: {0}")]
pub enum RequestError {
    #[error("Internal: {0}")]
    Internal(String),

    #[error("Offline")]
    Offline,

    #[error("Invalid auth")]
    InvalidAuth,

    /// An error that shouldn't happen (e.g. JSON serializing constant values)
    #[error("Should never happen: {0}")]
    ShouldNeverHappen(String),

    /// An error that shouldn't happen because the relay should be behaving as expected
    #[error("Server misbehaved: {0}")]
    ServerMisbehaved(String),
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

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "uniffi", derive(uniffi_macros::Error))]
#[error("Sign respond error: {0}")]
pub enum RespondError {
    #[error("Internal: {0}")]
    Internal(String),
}

pub struct Client {
    request_tx: tokio::sync::mpsc::UnboundedSender<(
        Params,
        tokio::sync::oneshot::Sender<Result<Response, RequestError>>,
    )>,
    sessions: Arc<tokio::sync::RwLock<HashMap<Topic, Session>>>,
    online_tx: Option<tokio::sync::mpsc::UnboundedSender<()>>,
}

const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

// TODO bindings integration
// - State:
//   - is app and wallet state coupled? should we build the DApp support right now to make it easier?
//   - does deduplication happen at the irn_subscription layer (like current SDKs) or do we do it for each action e.g. update, event, etc. (remember layered state and stateless architecture)

// TODO
// - WS reconnection & retries
//   - disconnect if no ping for 30s etc. (native only)
//   - reconnect back-off w/ random jitter to prevent server overload
//   - online/offline hints
//   - background/foreground hints

// TODO
// - session pings, update, events, emit
// - session expiry & renew
//   - expire implemented simply by filtering out expired sessions in `Client::add_sessions()` ?
//     - long-lived clients might suffer here. Maybe filter each reconnect?

// TODO error improvement
// - bundle size optimization: error enums only for actionable errors higher-up
// - use a single string variant for all errors (which would be shown to users!)
// - other is internal errors we don't expect to EVER happen (so show error code instead w/ GitHub issue link)
// TODO bundle size optimization
// - flutter JSON serialization, avoid back/forth in UniFFI?
// - use native crypto utils
// TODO relay changes
// - subscribe to other sessions as part of `wc_approveSession` etc.
// - (feasible?) wc_sessionRequestRespond which ACKs the `irn_subscription` message
// - https://www.notion.so/walletconnect/Design-Doc-Sign-Client-Rust-port-2303a661771e80628bdbf07c96a97b21?source=copy_link#2303a661771e808f895acbcab46bd12a
// - don't forward ignored messages e.g. ACKing etc. do it based on client version/flag
// - binary relay encoding: bincode?
// - pings for web platforms: https://reown-inc.slack.com/archives/C04DB2EAHE3/p1754402214830549
// - share scheduled disconnect time: https://reown-inc.slack.com/archives/C04DB2EAHE3/p1754406425810959
// - Force clients off if they are not subscribed to any topics after certain interval? (opt-in with a flag?): https://reown-inc.slack.com/archives/C03RR5C0F7X/p1721459186692409?thread_ts=1712767993.823029&cid=C03RR5C0F7X

// TODO
// - Verify API
// - 1CA
// - Link Mode
// - Events SDK & Analytics/TVF
//   - Additional events for measuring latency/reconnect performance/client network environment/etc. so we can tune. E.g. "should we retry to connect?"
// - Network state hinting (offline/online)
//   - offline: don't try to reconnect, but also don't force a disconnect
//   - online: reconnect if online() was called

// TODO tests
// - memory leak slow tests, run for days?. Kill WS many times over and over again to test. Create many sessions over and over again, update sessions, session requests, etc.
// - test killing the WS, not returning request, failing to connect, etc. in various stages of the lifecycle

#[allow(unused)]
impl Client {
    pub fn new(
        project_id: ProjectId,
        key: SecretKey,
    ) -> (
        Self,
        tokio::sync::mpsc::UnboundedReceiver<(Topic, SessionRequestJsonRpc)>,
    ) {
        assert_eq!(
            project_id.value().len(),
            32,
            "Project ID must be exactly 32 characters"
        );
        // TODO validate format i.e. hex

        let sessions = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let (request_tx, request_rx) = tokio::sync::mpsc::unbounded_channel();
        let (online_tx, online_rx) = tokio::sync::mpsc::unbounded_channel();

        crate::spawn::spawn(connect_loop_state_machine(
            RELAY_URL.to_string(),
            project_id,
            SigningKey::from_bytes(&key),
            sessions.clone(),
            tx,
            request_rx,
            online_rx,
        ));

        (Self { request_tx, sessions, online_tx: Some(online_tx) }, rx)
    }

    pub async fn add_sessions(
        &self,
        sessions: impl IntoIterator<Item = Session>,
    ) {
        let mut guard = self.sessions.write().await;
        for session in sessions {
            guard.insert(topic_from_sym_key(&session.session_sym_key), session);
        }
    }

    pub async fn get_sessions(&self) -> Vec<Session> {
        let guard = self.sessions.read().await;
        guard.values().cloned().collect()
    }

    /// Call this when the app and user are ready to receive session requests.
    /// Skip calling this if you intend to shortly call another SDK method, as those other methods will themselves call this.
    /// TODO actually call this from other methods
    pub fn online(&mut self) {
        if let Some(online_tx) = self.online_tx.take() {
            if let Err(e) = online_tx.send(()) {
                tracing::warn!("Failed to send online signal: {e:?}");
            }
        } else {
            tracing::warn!("Already called online()");
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

        tracing::debug!("Pairing with URI: {uri}");

        // TODO consider: immediately throw if expired? - maybe not necessary since FetchMessages returns empty array?

        // TODO update relay method to not remove message & approveSession removes it

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

                let request =
                    serde_json::from_slice::<ProposalJsonRpc>(&decrypted)
                        .map_err(|e| {
                            PairError::Internal(format!(
                                "Failed to parse decrypted message: {e}"
                            ))
                        })?;
                if request.method != "wc_sessionPropose" {
                    return Err(PairError::Internal(format!(
                        "Expected wc_sessionPropose, got {}",
                        request.method
                    )));
                }
                tracing::debug!("Decrypted Proposal: {:?}", request);
                tracing::debug!("rpc request: {}", request.id);
                tracing::debug!(
                    "{}",
                    serde_json::to_string_pretty(&request.params).unwrap()
                );
                let proposal = request.params;
                tracing::debug!("{proposal:?}");

                let proposer_public_key = hex::decode(proposal.proposer.public_key)
                    .map_err(|e| {
                        PairError::Internal(format!(
                            "Failed to decode proposer public key: {e}"
                        ))
                    })?
                    .try_into()
                    .map_err(|_| {
                        PairError::Internal(
                            "Failed to convert proposer public key to fixed-size array".to_owned()
                        )
                    })?;
                tracing::debug!("pairing topic: {}", pairing_uri.topic);

                // TODO validate namespaces: https://specs.walletconnect.com/2.0/specs/clients/sign/namespaces#12-proposal-namespaces-must-not-have-chains-empty

                return Ok(SessionProposal {
                    session_proposal_rpc_id: request.id.into_value(),
                    pairing_topic: pairing_uri.topic,
                    pairing_sym_key: pairing_uri.sym_key,
                    proposer_public_key,
                    relays: proposal.relays,
                    required_namespaces: proposal.required_namespaces,
                    optional_namespaces: proposal.optional_namespaces,
                    metadata: proposal.proposer.metadata,
                    session_properties: proposal.session_properties,
                    scoped_properties: proposal.scoped_properties,
                    expiry_timestamp: proposal.expiry_timestamp,
                });
            }
        }

        Err(PairError::Internal("No message found".to_string()))
    }

    pub async fn approve(
        &mut self,
        proposal: SessionProposal,
        approved_namespaces: HashMap<String, SettleNamespace>,
        self_metadata: Metadata,
    ) -> Result<Session, ApproveError> {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L341

        // TODO check is valid

        let self_key = x25519_dalek::StaticSecret::random();
        let self_public_key = PublicKey::from(&self_key);
        let shared_secret =
            diffie_hellman(&proposal.proposer_public_key.into(), &self_key);
        let session_topic = topic_from_sym_key(&shared_secret);
        debug!("session topic: {}", session_topic);

        let session_proposal_response = {
            let serialized = serde_json::to_string(&ProposalResponseJsonRpc {
                id: proposal.session_proposal_rpc_id,
                jsonrpc: "2.0".to_string(),
                result: ProposalResponse {
                    relay: Relay { protocol: "irn".to_string() },
                    responder_public_key: hex::encode(
                        self_public_key.to_bytes(),
                    ),
                },
            })
            .map_err(|e| ApproveError::Internal(e.to_string()))?;

            let key = ChaCha20Poly1305::new(&proposal.pairing_sym_key.into());
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
            let serialized = serde_json::to_string(&SessionSettleJsonRpc {
                id: generate_rpc_id(),
                jsonrpc: "2.0".to_string(),
                method: "wc_sessionSettle".to_string(),
                params: SessionSettle {
                    relay: Relay { protocol: "irn".to_string() },
                    namespaces: approved_namespaces,
                    controller: Controller {
                        public_key: hex::encode(self_public_key.to_bytes()),
                        metadata: self_metadata,
                    },
                    expiry: crate::time::SystemTime::now()
                        .duration_since(crate::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        + 60 * 60 * 24 * 7, // Session expiry is 7 days
                    session_properties: proposal
                        .session_properties
                        .as_ref()
                        .map(|p| serde_json::to_value(p).unwrap_or_default())
                        .unwrap_or_default(),
                    scoped_properties: proposal
                        .scoped_properties
                        .as_ref()
                        .map(|p| serde_json::to_value(p).unwrap_or_default())
                        .unwrap_or_default(),
                    // session_config: proposal.session_config,
                },
            })
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
            pairing_topic: proposal.pairing_topic,
            session_topic: session_topic.clone(),
            session_proposal_response,
            session_settlement_request,
            analytics: Some(AnalyticsData {
                correlation_id: Some(proposal.session_proposal_rpc_id as i64),
                chain_id: None,
                rpc_methods: None,
                tx_hashes: None,
                contract_addresses: None,
            }),
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
                return Err(ApproveError::Internal(
                    "Session rejected by relay".to_owned(),
                ));
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

        let session = Session {
            session_sym_key: shared_secret,
            self_public_key: self_public_key.to_bytes(),
        };
        let session = Session {
            session_sym_key: shared_secret,
            self_public_key: self_public_key.to_bytes(),
        };
        {
            let mut sessions = self.sessions.write().await;
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

        // TODO WS handling:
        // - when a session request is pending, and we get the event that the page regained focus, should we immediately ping the WS connection to test its liveness (?)
    }

    pub async fn respond(
        &mut self,
        topic: Topic,
        response: SessionRequestResponseJsonRpc,
    ) -> Result<(), RespondError> {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L701

        let serialized = serde_json::to_string(&response)
            .map_err(|e| RespondError::Internal(e.to_string()))?;

        let shared_secret = {
            let sessions = self.sessions.read().await;
            let session = sessions.get(&topic).ok_or(
                RespondError::Internal("Session not found".to_owned()),
            )?;
            session.session_sym_key
        };

        let key = ChaCha20Poly1305::new(&shared_secret.into());
        let nonce = ChaCha20Poly1305::generate_nonce()
            .map_err(|e| RespondError::Internal(e.to_string()))?;
        let encrypted = key
            .encrypt(&nonce, serialized.as_bytes())
            .map_err(|e| RespondError::Internal(e.to_string()))?;
        let encoded = encode_envelope_type0(&EnvelopeType0 {
            iv: nonce.into(),
            sb: encrypted,
        })
        .map_err(|e| RespondError::Internal(e.to_string()))?;
        let message = BASE64.encode(encoded.as_slice()).into();

        self.request::<bool>(relay_rpc::rpc::Params::Publish(Publish {
            topic,
            message,
            attestation: None, // TODO
            ttl_secs: 300,
            tag: 1109,
            prompt: false,
            analytics: None, // TODO
        }))
        .await
        .map_err(|e| RespondError::Internal(e.to_string()))?;

        Ok(())
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
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();
        self.request_tx.send((params, response_tx)).map_err(|e| {
            RequestError::Internal(format!(
                "Failed to send request, request_tx closed: {e}"
            ))
        })?;
        let response = match response_rx.await {
            Ok(Ok(response)) => response,
            Ok(Err(e)) => {
                return Err(RequestError::Internal(format!(
                    "Failed to receive response: {e}"
                )));
            }
            Err(e) => {
                return Err(RequestError::Internal(format!(
                    "Failed to receive response (2): {e}"
                )));
            }
        };

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
type ConnectWebSocket = web_sys::WebSocket;

#[cfg(not(target_arch = "wasm32"))]
type ConnectWebSocket = ();

async fn connect(
    relay_url: String,
    project_id: ProjectId,
    key: &SigningKey,
    topics: Vec<Topic>,
    initial_req: Params,
) -> Result<
    (
        u64,
        tokio::sync::mpsc::UnboundedReceiver<IncomingMessage>,
        ConnectWebSocket,
    ),
    RequestError,
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

            encoder.encode(serde_json::to_string(&data).unwrap().as_bytes())
        };
        let header = encoder.encode(
            serde_json::to_string(&JwtHeader::default()).unwrap().as_bytes(),
        );
        let message = format!("{header}.{claims}");
        let signature = {
            let data = key.sign(message.as_bytes());
            encoder.encode(&data.to_bytes())
        };
        let auth = format!("{message}.{signature}");

        let conn_opts =
            ConnectionOptions::new(project_id, auth).with_address(&relay_url);
        conn_opts.as_url().unwrap().to_string()
    };

    // #[cfg(not(target_arch = "wasm32"))]
    // {
    //     let (mut ws_stream, _response) =
    //         crate::time::timeout(REQUEST_TIMEOUT, connect_async(url))
    //             .await
    //             .map_err(|e| RequestError::Internal(e.to_string()))?
    //             .map_err(|e| RequestError::Internal(e.to_string()))?;

    //     let invalid_auth = self.invalid_auth.clone();
    //     crate::spawn::spawn(async move {
    //         use {
    //             tokio::net::TcpStream,
    //             tokio_tungstenite::{MaybeTlsStream, WebSocketStream},
    //         };

    //         let mut message_id = MIN_RPC_ID;
    //         let mut response_channels = HashMap::<
    //             _,
    //             tokio::sync::oneshot::Sender<Result<Response, RequestError>>,
    //         >::new();

    //         async fn send_request(
    //             ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    //             initial_req: (
    //                 Params,
    //                 tokio::sync::oneshot::Sender<
    //                     Result<Response, RequestError>,
    //                 >,
    //             ),
    //             message_id: &mut u64,
    //             response_channels: &mut HashMap<
    //                 MessageId,
    //                 tokio::sync::oneshot::Sender<
    //                     Result<Response, RequestError>,
    //                 >,
    //             >,
    //         ) {
    //             *message_id += 1;
    //             response_channels
    //                 .insert(MessageId::new(*message_id), initial_req.1);
    //             let request = Payload::Request(Request::new(
    //                 MessageId::new(*message_id),
    //                 initial_req.0,
    //             ));
    //             let serialized = serde_json::to_string(&request)
    //                 .map_err(|e| {
    //                     RequestError::ShouldNeverHappen(format!(
    //                         "Failed to serialize request: {e}"
    //                     ))
    //                 })
    //                 .unwrap();
    //             ws_stream.send(Message::Text(serialized.into())).await.unwrap();
    //         }

    //         // TODO this will soon be moved to initial WebSocket request
    //         send_request(
    //             &mut ws_stream,
    //             (initial_req, response_oneshot.0),
    //             &mut message_id,
    //             &mut response_channels,
    //         )
    //         .await;

    //         let exit_reason = loop {
    //             use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;

    //             tokio::select! {
    //                 Some((params, response_tx)) = request_rx.recv() => {
    //                     send_request(&mut ws_stream, (params, response_tx), &mut message_id, &mut response_channels).await;
    //                 }
    //                 Some(message) = ws_stream.next() => {
    //                     let n = message
    //                         .map_err(|e| {
    //                             RequestError::Internal(format!(
    //                                 "WebSocket stream error: {e}"
    //                             ))
    //                         })
    //                         .expect("WebSocket stream error");
    //                     #[allow(clippy::single_match)]
    //                     match n {
    //                         Message::Text(message) => {
    //                             let payload =
    //                                 serde_json::from_str::<Payload>(&message)
    //                                     .map_err(|e| {
    //                                         RequestError::Internal(format!(
    //                                             "Failed to parse payload: {e}"
    //                                         ))
    //                                     });
    //                             match payload {
    //                                 Ok(payload) => {
    //                                     let id = payload.id();
    //                                     match payload {
    //                                         Payload::Request(request) => {
    //                                             #[allow(clippy::single_match)]
    //                                             match request.params {
    //                                                 Params::Subscription(
    //                                                     sub_msg
    //                                                 ) => {
    //                                                     handle_irn_subscription(id, sub_msg);
    //                                                 }
    //                                                 _ => {}
    //                                             }
    //                                         }
    //                                         Payload::Response(response) => {
    //                                             if let Some(response_tx) =
    //                                                 response_channels
    //                                                     .remove(&response.id())
    //                                             {
    //                                                 if let Err(e) =
    //                                                     response_tx.send(Ok(response))
    //                                                 {
    //                                                     tracing::debug!("Failed to send response: {e:?}");
    //                                                 }
    //                                             }
    //                                         }
    //                                     }
    //                                 },
    //                                 Err(e) => {
    //                                     // no-op
    //                                 }
    //                             }
    //                         }
    //                         Message::Close(close_event) => {
    //                             tracing::debug!("websocket onclose: {:?}", close_event);
    //                             if let Some(close_event) = close_event {
    //                                 if close_event.code == CloseCode::Iana(3000) {
    //                                     tracing::error!("Invalid auth: {}", close_event.reason);
    //                                     invalid_auth.store(true, std::sync::atomic::Ordering::Relaxed);
    //                                     break RequestError::InvalidAuth;
    //                                 } else {
    //                                     break RequestError::Offline;
    //                                 }
    //                             } else {
    //                                 break RequestError::Offline;
    //                             }
    //                         }
    //                         e => tracing::debug!("ignoring tungstenite message: {:?}", e),
    //                     }
    //                 }
    //                 Some(id) = irn_subscription_ack_rx.recv() => {
    //                     let request = Payload::Response(Response::Success(SuccessfulResponse {
    //                         id,
    //                         result: serde_json::to_value(true).expect("TODO"),
    //                         jsonrpc: "2.0".to_string().into(),
    //                     }));
    //                     let serialized = serde_json::to_string(&request).map_err(|e| {
    //                         RequestError::ShouldNeverHappen(format!(
    //                             "Failed to serialize request: {e}"
    //                         ))
    //                     }).expect("TODO");
    //                     ws_stream.send(Message::Text(serialized.into())).await.unwrap();
    //                 }
    //             }
    //         };

    //         for (_, response_tx) in response_channels {
    //             let _ = response_tx.send(Err(exit_reason.clone()));
    //         }
    //     });
    // }

    #[cfg(target_arch = "wasm32")]
    {
        use {
            wasm_bindgen::{prelude::Closure, JsCast},
            web_sys::{CloseEvent, Event, MessageEvent},
        };

        let ws = web_sys::WebSocket::new(&url).map_err(|e| {
            RequestError::Internal(format!("Failed to create WebSocket: {e:?}"))
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
        ws.set_onclose(Some(
            // TODO fix leak
            Box::leak(Box::new(on_close_closure)).as_ref().unchecked_ref(),
        ));

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
        ws.set_onerror(Some(
            // TODO fix leak
            Box::leak(Box::new(on_error_closure)).as_ref().unchecked_ref(),
        ));

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
        ws.set_onmessage(Some(
            // TODO fix leak
            Box::leak(Box::new(onmessage_closure)).as_ref().unchecked_ref(),
        ));

        let (tx_open, mut rx_open) = tokio::sync::mpsc::channel(1);
        let onopen_closure = Closure::wrap(Box::new(move |_event: Event| {
            let tx_open = tx_open.clone();
            crate::spawn::spawn(async move {
                let _ = tx_open.send(()).await.ok();
            });
        }) as Box<dyn Fn(Event)>);
        ws.set_onopen(Some(
            // TODO fix leak
            Box::leak(Box::new(onopen_closure)).as_ref().unchecked_ref(),
        ));

        tracing::debug!("awaiting onopen");

        crate::time::timeout(REQUEST_TIMEOUT, rx_open.recv())
            .await
            .map_err(|e| {
                RequestError::Internal(format!(
                    "Timeout waiting for onopen: {e}"
                ))
            })?
            .ok_or_else(|| {
                RequestError::Internal("Failed to receive onopen".to_string())
            })?;
        ws.set_onopen(None);
        tracing::debug!("onopen received");

        let mut message_id = MIN_RPC_ID + 1;

        if !topics.is_empty() {
            // TODO batch this extra request together with the initial connection

            message_id = message_id + 1;
            let payload_request = Payload::Request(Request::new(
                MessageId::new(message_id),
                Params::BatchSubscribe(BatchSubscribe { topics }),
            ));
            let serialized = serde_json::to_string(&payload_request)
                .map_err(|e| {
                    RequestError::ShouldNeverHappen(format!(
                        "Failed to serialize request: {e}"
                    ))
                })
                .expect("TODO");
            ws.send_with_str(&serialized).expect("TODO");

            let incoming_message = match crate::time::timeout(
                REQUEST_TIMEOUT,
                on_incomingmessage_rx.recv(),
            )
            .await
            {
                Ok(Some(message)) => message,
                Ok(None) => {
                    return Err(RequestError::Internal(
                        "Timeout waiting for batch subscribe response"
                            .to_string(),
                    ));
                }
                Err(e) => {
                    return Err(RequestError::Internal(format!(
                        "Timeout waiting for batch subscribe response: {e:?}"
                    )));
                }
            };
            match incoming_message {
                IncomingMessage::Close(reason) => match reason {
                    CloseReason::InvalidAuth => {
                        return Err(RequestError::InvalidAuth);
                    }
                    CloseReason::Error(reason) => {
                        tracing::debug!(
                            "ConnectRequest: CloseReason::Error: {reason}"
                        );
                        return Err(RequestError::Offline);
                    }
                },
                IncomingMessage::Message(payload) => {
                    let id = payload.id();
                    match payload {
                        Payload::Request(request) => {
                            tracing::error!("unexpected message request in ConnectRequest state: {:?}", request);
                            return Err(RequestError::Internal(
                                "unexpected message request in ConnectRequest state".to_string(),
                            ));
                        }
                        Payload::Response(response) => {
                            if id == MessageId::new(message_id) {
                                // success, no-op
                            } else {
                                tracing::error!("unexpected message response in ConnectRequest state: {:?}", response);
                                return Err(RequestError::Internal(
                                    "unexpected message response in ConnectRequest state".to_string(),
                                ));
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
            .map_err(|e| {
                RequestError::ShouldNeverHappen(format!(
                    "Failed to serialize request: {e}"
                ))
            })
            .expect("TODO");
        ws.send_with_str(&serialized).expect("TODO");

        return Ok((message_id, on_incomingmessage_rx, ws));
    }
}

enum ConnectionState {
    Idle,
    Poisoned,
    MaybeReconnect,
    ConnectSubscribe,
    AwaitingSubscribeResponse(
        u64,
        tokio::sync::mpsc::UnboundedReceiver<IncomingMessage>,
        ConnectWebSocket,
        tokio::sync::mpsc::Receiver<()>,
    ),
    Backoff,
    ConnectRequest(
        (Params, tokio::sync::oneshot::Sender<Result<Response, RequestError>>),
    ),
    AwaitingConnectRequestResponse(
        u64,
        tokio::sync::mpsc::UnboundedReceiver<IncomingMessage>,
        ConnectWebSocket,
        (Params, tokio::sync::oneshot::Sender<Result<Response, RequestError>>),
        tokio::sync::mpsc::Receiver<()>,
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
        tokio::sync::mpsc::Receiver<()>,
    ),
    ConnectRetryRequest(
        (Params, tokio::sync::oneshot::Sender<Result<Response, RequestError>>),
    ),
    AwaitingConnectRetryRequestResponse(
        u64,
        tokio::sync::mpsc::UnboundedReceiver<IncomingMessage>,
        ConnectWebSocket,
        tokio::sync::oneshot::Sender<Result<Response, RequestError>>,
        tokio::sync::mpsc::Receiver<()>,
    ),
}

async fn connect_loop_state_machine(
    relay_url: String,
    project_id: ProjectId,
    key: SigningKey,
    sessions: std::sync::Arc<tokio::sync::RwLock<HashMap<Topic, Session>>>,
    session_request_tx: tokio::sync::mpsc::UnboundedSender<(
        Topic,
        SessionRequestJsonRpc,
    )>,
    mut request_rx: tokio::sync::mpsc::UnboundedReceiver<(
        Params,
        tokio::sync::oneshot::Sender<Result<Response, RequestError>>,
    )>,
    mut online_rx: tokio::sync::mpsc::UnboundedReceiver<()>,
) {
    let (irn_subscription_ack_tx, mut irn_subscription_ack_rx) =
        tokio::sync::mpsc::unbounded_channel();
    let handle_irn_subscription = {
        let sessions = sessions.clone();
        let session_request_tx = session_request_tx.clone();
        let irn_subscription_ack_tx = irn_subscription_ack_tx.clone();
        move |id: MessageId, sub_msg: Subscription| {
            let sessions = sessions.clone();
            let session_request_tx = session_request_tx.clone();
            let irn_subscription_ack_tx = irn_subscription_ack_tx.clone();
            async move {
                let session_sym_key = {
                    let sessions = sessions.read().await;
                    // TODO drop message if cannot find decryption key
                    let session = sessions.get(&sub_msg.data.topic).unwrap();
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
                    .decrypt(&Nonce::from(envelope.iv), envelope.sb.as_slice())
                    .map_err(|e| PairError::Internal(e.to_string()))
                    .unwrap();
                let value =
                    serde_json::from_slice::<serde_json::Value>(&decrypted)
                        .map_err(|e| PairError::Internal(e.to_string()))
                        .unwrap();
                if let Some(method) = value.get("method") {
                    if method.as_str() == Some("wc_sessionRequest") {
                        // TODO implement relay-side request queue
                        let request = serde_json::from_value::<
                            SessionRequestJsonRpc,
                        >(value)
                        .map_err(|e| {
                            PairError::Internal(format!(
                                "Failed to parse decrypted message: {e}"
                            ))
                        })
                        .unwrap();
                        session_request_tx
                            .send((sub_msg.data.topic, request))
                            .unwrap();
                        if let Err(e) = irn_subscription_ack_tx.send(id) {
                            tracing::debug!(
                                "Failed to send subscription ack: {e}"
                            );
                        }
                    } else if method.as_str() == Some("wc_sessionUpdate") {
                        // TODO update session locally (if not older than last update)
                        // TODO write state to storage (blocking)
                        if let Err(e) = irn_subscription_ack_tx.send(id) {
                            tracing::debug!(
                                "Failed to send subscription ack: {e}"
                            );
                        }
                    } else if method.as_str() == Some("wc_sessionExtend") {
                        // TODO update session locally (if not older than last update)
                        // TODO write state to storage (blocking)
                        if let Err(e) = irn_subscription_ack_tx.send(id) {
                            tracing::debug!(
                                "Failed to send subscription ack: {e}"
                            );
                        }
                    } else if method.as_str() == Some("wc_sessionEmit") {
                        // TODO dedup events based on JSON RPC history
                        // TODO emit event callback (blocking?)
                        if let Err(e) = irn_subscription_ack_tx.send(id) {
                            tracing::debug!(
                                "Failed to send subscription ack: {e}"
                            );
                        }
                    } else if method.as_str() == Some("wc_sessionPing") {
                        if let Err(e) = irn_subscription_ack_tx.send(id) {
                            tracing::debug!(
                                "Failed to send subscription ack: {e}"
                            );
                        }
                    } else {
                        tracing::error!("Unexpected method: {}", method);
                        if let Err(e) = irn_subscription_ack_tx.send(id) {
                            tracing::debug!(
                                "Failed to send subscription ack: {e}"
                            );
                        }
                    }
                } else {
                    tracing::debug!("ignoring response message: {:?}", value);
                    if let Err(e) = irn_subscription_ack_tx.send(id) {
                        tracing::debug!("Failed to send subscription ack: {e}");
                    }

                    // TODO handle session request responses. Unsure if other responses are needed
                }
            }
        }
    };

    let mut state = ConnectionState::Idle;
    loop {
        match state {
            ConnectionState::Idle => {
                // TODO avoid select! as it doesn't guarantee that `else` branch exists (it will panic otherwise)
                tokio::select! {
                    Some(request) = request_rx.recv() => state = ConnectionState::ConnectRequest(request),
                    Some(()) = online_rx.recv() => state = ConnectionState::MaybeReconnect,
                    else => break,
                }
            }
            ConnectionState::Poisoned => {
                if let Some((_params, response_tx)) = request_rx.recv().await {
                    if let Err(e) =
                        response_tx.send(Err(RequestError::InvalidAuth))
                    {
                        tracing::debug!("Failed to send error response: {e:?}");
                    }
                }
                state = ConnectionState::Idle;
            }
            ConnectionState::MaybeReconnect => {
                if sessions.read().await.is_empty() {
                    state = ConnectionState::Idle;
                } else {
                    state = ConnectionState::ConnectSubscribe;
                }
            }
            ConnectionState::ConnectSubscribe => {
                let topics = {
                    let sessions = sessions.read().await;
                    sessions.iter().map(|(topic, _)| topic.clone()).collect()
                };
                let connect_res = connect(
                    relay_url.clone(),
                    project_id.clone(),
                    &key,
                    vec![],
                    Params::BatchSubscribe(BatchSubscribe { topics }),
                )
                .await;
                match connect_res {
                    Ok((message_id, on_incomingmessage_rx, ws)) => {
                        state = ConnectionState::AwaitingSubscribeResponse(
                            message_id,
                            on_incomingmessage_rx,
                            ws,
                            crate::time::durable_sleep(REQUEST_TIMEOUT),
                        );
                    }
                    Err(e) => {
                        tracing::debug!("ConnectSubscribe failed: {e:?}");
                        state = ConnectionState::Backoff;
                    }
                }
            }
            ConnectionState::AwaitingSubscribeResponse(
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
                                        state = ConnectionState::Poisoned;
                                    }
                                    CloseReason::Error(reason) => {
                                        tracing::debug!("AwaitingSubscribeResponse: CloseReason::Error: {reason}");
                                        state = ConnectionState::Backoff;
                                    }
                                }
                            }
                            IncomingMessage::Message(payload) => {
                                let id = payload.id();
                                match payload {
                                    Payload::Request(request) => {
                                        tracing::warn!("ignoring message request in AwaitingSubscribeResponse state: {:?}", request);
                                        state = ConnectionState::AwaitingSubscribeResponse(
                                            message_id,
                                             on_incomingmessage_rx,
                                            ws,
                                            sleep,
                                        );
                                    }
                                    Payload::Response(response) => {
                                        if id == MessageId::new(message_id) {
                                            state = ConnectionState::Connected(message_id, on_incomingmessage_rx, ws);
                                        } else {
                                            tracing::warn!("ignoring message response in AwaitingSubscribeResponse state: {:?}", response);
                                            state = ConnectionState::AwaitingSubscribeResponse(
                                                message_id,
                                                 on_incomingmessage_rx,
                                                ws,
                                                sleep,
                                            );
                                        }
                                    }
                                }
                                }
                            }
                        } else {
                            state = ConnectionState::Backoff;
                        }
                    },
                    Some(()) = sleep.recv() => state = ConnectionState::Backoff,
                }
            }
            ConnectionState::Backoff => {
                // TODO start at 0, then etc.
                // TODO consider max 1s polling? Why go less frequently?
                let sleep = crate::time::sleep(Duration::from_millis(1000));
                // TODO avoid select! as it doesn't guarantee that all branches are covered (it will panic otherwise)
                tokio::select! {
                    Some(req) = request_rx.recv() => state = ConnectionState::ConnectRequest(req),
                    () = sleep => state = ConnectionState::MaybeReconnect,
                    else => break,
                }
            }
            ConnectionState::ConnectRequest((request, response_tx)) => {
                let topics = {
                    let sessions = sessions.read().await;
                    sessions.iter().map(|(topic, _)| topic.clone()).collect()
                };
                let connect_res = connect(
                    relay_url.clone(),
                    project_id.clone(),
                    &key,
                    topics,
                    request.clone(),
                )
                .await;
                match connect_res {
                    Ok((message_id, on_incomingmessage_rx, ws)) => {
                        state = ConnectionState::AwaitingConnectRequestResponse(
                            message_id,
                            on_incomingmessage_rx,
                            ws,
                            (request, response_tx),
                            crate::time::durable_sleep(REQUEST_TIMEOUT),
                        );
                    }
                    Err(e) => {
                        if let Err(e) = response_tx.send(Err(e)) {
                            tracing::warn!(
                                "Failed to send error response: {e:?}"
                            );
                        }
                        state = ConnectionState::MaybeReconnect;
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
                                        state = ConnectionState::Poisoned;
                                    }
                                    CloseReason::Error(reason) => {
                                        tracing::debug!("AwaitingConnectRequestResponse: CloseReason::Error: {reason}");
                                        if let Err(e) =
                                            response_tx.send(Err(RequestError::Offline))
                                        {
                                            tracing::warn!("Failed to send error response: {e:?}");
                                        }
                                        state = ConnectionState::MaybeReconnect;
                                    }
                                }
                            }
                            IncomingMessage::Message(payload) => {
                                let id = payload.id();
                                match payload {
                                    Payload::Request(payload_request) => {
                                        tracing::warn!("ignoring message request in AwaitingSubscribeResponse state: {:?}", payload_request);
                                        state = ConnectionState::AwaitingConnectRequestResponse(
                                            message_id,
                                            on_incomingmessage_rx,
                                            ws,
                                            (request, response_tx),
                                            sleep,
                                        );
                                    }
                                    Payload::Response(response) => {
                                        if id == MessageId::new(message_id) {
                                            if let Err(e) =
                                                response_tx.send(Ok(response))
                                            {
                                                tracing::warn!("Failed to send response: {e:?}");
                                            }
                                            state = ConnectionState::Connected(message_id, on_incomingmessage_rx, ws);
                                        } else {
                                            tracing::warn!("ignoring message response in AwaitingSubscribeResponse state: {:?}", response);
                                            state = ConnectionState::AwaitingConnectRequestResponse(
                                                message_id,
                                                on_incomingmessage_rx,
                                                ws,
                                                (request, response_tx),
                                                sleep,
                                            );
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
                            state = ConnectionState::MaybeReconnect;
                        }
                    },
                    Some(()) = sleep.recv() => {
                        if let Err(e) =
                            response_tx.send(Err(RequestError::Offline))
                        {
                            tracing::warn!("Failed to send error response: {e:?}");
                        }
                        state = ConnectionState::MaybeReconnect;
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
                                        state = ConnectionState::Poisoned;
                                    } else {
                                        state = ConnectionState::MaybeReconnect;
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
                                                    handle_irn_subscription(id, sub_msg).await;
                                                }
                                                _ => tracing::warn!("ignoring message request in Connected state: {:?}", request),
                                            }
                                            state = ConnectionState::Connected(
                                                message_id,
                                                on_incomingmessage_rx,
                                                ws,
                                            );
                                        }
                                        Payload::Response(response) => {
                                            tracing::warn!("ignoring message response in Connected state: {:?}", response);
                                            state = ConnectionState::Connected(
                                                message_id,
                                                on_incomingmessage_rx,
                                                ws,
                                            );
                                        }
                                    }
                                }
                            }
                        } else {
                            state = ConnectionState::MaybeReconnect;
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
                            ws.send_with_str(&serialized).expect("TODO");

                            state = ConnectionState::AwaitingRequestResponse(
                                message_id,
                                on_incomingmessage_rx,
                                ws,
                                (request, response_tx),
                                crate::time::durable_sleep(REQUEST_TIMEOUT),
                            );
                        } else {
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
                            ws.send_with_str(&serialized).expect("TODO");

                            state = ConnectionState::Connected(
                                message_id,
                                on_incomingmessage_rx,
                                ws,
                            );
                        } else {
                            break;
                        }
                    }
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
                                        state = ConnectionState::Poisoned;
                                    } else {
                                        state = ConnectionState::ConnectRetryRequest((request, response_tx));
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
                                                    handle_irn_subscription(id, sub_msg).await;
                                                }
                                                _ => tracing::warn!("ignoring message request in AwaitingRequestResponse state: {:?}", payload_request),
                                            }
                                            state = ConnectionState::AwaitingRequestResponse(
                                                message_id,
                                                on_incomingmessage_rx,
                                                ws,
                                                (request, response_tx),
                                                sleep,
                                            );
                                        }
                                        Payload::Response(response) => {
                                            if id == MessageId::new(message_id) {
                                                if let Err(e) =
                                                    response_tx.send(Ok(response))
                                                {
                                                    tracing::warn!("Failed to send response in AwaitingRequestResponse state: {e:?}");
                                                }
                                                state = ConnectionState::Connected(message_id, on_incomingmessage_rx, ws);
                                            } else {
                                                tracing::warn!("ignoring message response in AwaitingSubscribeResponse state: {:?}", response);
                                                state = ConnectionState::AwaitingRequestResponse(
                                                    message_id,
                                                    on_incomingmessage_rx,
                                                    ws,
                                                    (request, response_tx),
                                                    sleep,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    Some(()) = sleep.recv() => state = ConnectionState::ConnectRetryRequest((request, response_tx)),
                }
            }
            ConnectionState::ConnectRetryRequest((request, response_tx)) => {
                let topics = {
                    let sessions = sessions.read().await;
                    sessions.iter().map(|(topic, _)| topic.clone()).collect()
                };
                let connect_res = connect(
                    relay_url.clone(),
                    project_id.clone(),
                    &key,
                    topics,
                    request,
                )
                .await;
                match connect_res {
                    Ok((message_id, on_incomingmessage_rx, ws)) => {
                        state = ConnectionState::AwaitingConnectRetryRequestResponse(
                            message_id,
                            on_incomingmessage_rx,
                            ws,
                            response_tx,
                            crate::time::durable_sleep(REQUEST_TIMEOUT),
                        );
                    }
                    Err(e) => {
                        if let Err(e) = response_tx.send(Err(e)) {
                            tracing::warn!(
                                "Failed to send error response: {e:?}"
                            );
                        }
                        state = ConnectionState::MaybeReconnect;
                    }
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
                                        state = ConnectionState::Poisoned;
                                    } else {
                                        if let Err(e) =
                                            response_tx.send(Err(RequestError::Offline))
                                        {
                                            tracing::warn!("Failed to send error response: {e:?}");
                                        }
                                        state = ConnectionState::MaybeReconnect;
                                    }
                                }
                                IncomingMessage::Message(payload) => {
                                    let id = payload.id();
                                    match payload {
                                        Payload::Request(request) => {
                                            // TODO consider handling anyway, if possible
                                            tracing::warn!("ignoring message request in AwaitingConnectRetryRequestResponse state: {:?}", request);
                                            state = ConnectionState::AwaitingConnectRetryRequestResponse(
                                                message_id,
                                                on_incomingmessage_rx,
                                                ws,
                                                response_tx,
                                                sleep,
                                            );
                                        }
                                        Payload::Response(response) => {
                                            if id == MessageId::new(message_id) {
                                                if let Err(e) =
                                                    response_tx.send(Ok(response))
                                                {
                                                    tracing::warn!("Failed to send response: {e:?}");
                                                }
                                                state = ConnectionState::Connected(message_id, on_incomingmessage_rx, ws);
                                            } else {
                                                tracing::warn!("ignoring message response in AwaitingConnectRetryRequestResponse state: {:?}", response);
                                                state = ConnectionState::AwaitingConnectRetryRequestResponse(
                                                    message_id,
                                                    on_incomingmessage_rx,
                                                    ws,
                                                    response_tx,
                                                    sleep,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    Some(()) = sleep.recv() => state = ConnectionState::MaybeReconnect,
                }
            }
        }
    }

    while let Some((_request, response_tx)) = request_rx.recv().await {
        if let Err(e) = response_tx
            .send(Err(RequestError::Internal("request_rx closed".to_string())))
        {
            tracing::warn!("Failed to send error response: {e:?}");
        }
    }
}

pub fn generate_key() -> SecretKey {
    SigningKey::generate(&mut rand::thread_rng()).to_bytes()
}

#[cfg(feature = "uniffi")]
#[uniffi::export(with_foreign)]
pub trait SessionRequestListener: Send + Sync {
    fn on_session_request(
        &self,
        topic: String,
        session_request: SessionRequestJsonRpcFfi,
    );
}

// UniFFI wrapper for better API naming
#[cfg(feature = "uniffi")]
#[derive(uniffi::Object)]
pub struct SignClient {
    client: std::sync::Arc<tokio::sync::Mutex<Client>>,
    session_request_rx: std::sync::Mutex<
        Option<
            tokio::sync::mpsc::UnboundedReceiver<(
                Topic,
                SessionRequestJsonRpc,
            )>,
        >,
    >,
}

#[cfg(feature = "uniffi")]
#[uniffi::export(async_runtime = "tokio")]
impl SignClient {
    #[uniffi::constructor]
    pub fn new(project_id: String, key: Vec<u8>) -> Self {
        tracing::debug!(
            "Creating new SignClient with project_id: {project_id}"
        );
        let (client, session_request_rx) = Client::new(
            ProjectId::from(project_id),
            key.try_into().expect("Invalid key format - must be 32 bytes"),
        );
        Self {
            client: std::sync::Arc::new(tokio::sync::Mutex::new(client)),
            session_request_rx: std::sync::Mutex::new(Some(session_request_rx)),
        }
    }

    pub fn generate_key(&self) -> Vec<u8> {
        generate_key().to_vec()
    }

    pub async fn register_session_request_listener(
        &self,
        listener: Arc<dyn SessionRequestListener>,
    ) {
        let mut rx_guard = self.session_request_rx.lock().unwrap();
        if let Some(mut rx) = rx_guard.take() {
            tokio::spawn(async move {
                tracing::info!(
                    "Starting session request listener with debug logging"
                );
                while let Some((topic, session_request)) = rx.recv().await {
                    tracing::debug!("Received session request - Topic: {:?}, SessionRequest: {:?}", topic, session_request);
                    let session_request_ffi: SessionRequestJsonRpcFfi =
                        session_request.into();
                    listener.on_session_request(
                        topic.to_string(),
                        session_request_ffi,
                    );
                }
                tracing::info!("Session request listener stopped");
            });
        } else {
            tracing::warn!("Session request listener already started or receiver not available");
        }
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

    pub async fn pair_json(&self, uri: String) -> Result<String, PairError> {
        let proposal = {
            let mut client = self.client.lock().await;
            client.pair(&uri).await?
        };
        let proposal_ffi: SessionProposalFfi = proposal.into();
        let serialized_proposal = serde_json::to_string(&proposal_ffi)
            .expect("Failed to serialize response");
        Ok(serialized_proposal)
    }

    //TODO: Add approved namespaces builder util function
    pub async fn approve(
        &self,
        proposal: SessionProposalFfi,
        approved_namespaces: HashMap<String, SettleNamespace>,
        self_metadata: Metadata,
    ) -> Result<SessionFfi, ApproveError> {
        let proposal: SessionProposal = proposal.into();
        tracing::debug!("approved_namespaces: {:?}", approved_namespaces);
        tracing::debug!("self_metadata: {:?}", self_metadata);
        let session = {
            let mut client = self.client.lock().await;
            client.approve(proposal, approved_namespaces, self_metadata).await?
        };
        Ok(session.into())
    }

    pub async fn approve_json(
        &self,
        proposal: String,
        approved_namespaces: String,
        self_metadata: String,
    ) -> Result<String, ApproveError> {
        let proposal: SessionProposalFfi = serde_json::from_str(&proposal)
            .expect("Failed to deserialize proposal");
        let approved_namespaces: HashMap<String, SettleNamespace> =
            serde_json::from_str(&approved_namespaces)
                .expect("Failed to deserialize approved_namespaces");
        let self_metadata: Metadata = serde_json::from_str(&self_metadata)
            .expect("Failed to deserialize self_metadata");

        tracing::debug!("approved_namespaces: {:?}", approved_namespaces);
        tracing::debug!("self_metadata: {:?}", self_metadata);
        let session = {
            let mut client = self.client.lock().await;
            client
                .approve(proposal.into(), approved_namespaces, self_metadata)
                .await?
        };
        let session_ffi: SessionFfi = session.into();
        let serialized_session = serde_json::to_string(&session_ffi)
            .expect("Failed to serialize response");
        Ok(serialized_session)
    }

    pub async fn respond(
        &self,
        topic: String,
        response: SessionRequestResponseJsonRpcFfi,
    ) -> Result<String, RespondError> {
        tracing::debug!("responding session request: {:?}", response);

        let mut client = self.client.lock().await;
        let response_internal: SessionRequestResponseJsonRpc = response.into();
        let topic_topic: Topic = topic.clone().into();
        client.respond(topic_topic, response_internal).await?;
        Ok(topic)
    }
}

#[derive(Debug, Clone)]
pub struct SessionProposal {
    pub session_proposal_rpc_id: u64,
    pub pairing_topic: Topic,
    pub pairing_sym_key: [u8; 32],
    pub proposer_public_key: [u8; 32],
    pub relays: Vec<crate::sign::protocol_types::Relay>,
    pub required_namespaces: ProposalNamespaces,
    pub optional_namespaces: ProposalNamespaces,
    pub metadata: Metadata,
    pub session_properties: Option<HashMap<String, String>>,
    pub scoped_properties: Option<HashMap<String, String>>,
    pub expiry_timestamp: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Session {
    pub session_sym_key: [u8; 32],
    pub self_public_key: [u8; 32],
}

#[cfg(test)]
mod conversion_tests {
    use super::*;

    #[test]
    fn test_session_proposal_conversion() {
        // Create a test SessionProposal with known values
        let test_topic = Topic::from(
            "0c814f7d2d56c0e840f75612addaa170af479b1c8499632430b41c298bf49907"
                .to_string(),
        );
        let test_id = 1234567890;

        let session_proposal = SessionProposal {
            session_proposal_rpc_id: test_id,
            pairing_topic: test_topic.clone(),
            pairing_sym_key: [1u8; 32],
            proposer_public_key: [2u8; 32],
            relays: vec![],
            required_namespaces: std::collections::HashMap::new(),
            optional_namespaces: std::collections::HashMap::new(),
            metadata: Metadata {
                name: "Test".to_string(),
                description: "Test".to_string(),
                url: "https://test.com".to_string(),
                icons: vec![],
                verify_url: None,
                redirect: None,
            },
            session_properties: None,
            scoped_properties: None,
            expiry_timestamp: None,
        };

        // Convert to FFI
        let ffi_proposal: SessionProposalFfi = session_proposal.into();

        // Print the actual values to see what we get
        println!("Original topic: {test_topic:?}");
        println!("Topic Display: {test_topic}");
        println!("Topic Debug: {test_topic:?}");
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
