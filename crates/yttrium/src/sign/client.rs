use {
    crate::sign::{
        client_errors::{
            ApproveError, ConnectError, DisconnectError, EmitError,
            ExtendError, PairError, RejectError, RequestError, RespondError,
            UpdateError,
        },
        client_types::{
            ConnectParams, ConnectResult, PairingInfo, RejectionReason,
            Session, SessionProposal, TransportType,
        },
        envelope_type0::decrypt_type0_envelope,
        pairing_uri,
        protocol_types::{
            Controller, JsonRpcRequest, JsonRpcRequestParams, Metadata,
            Proposal, ProposalJsonRpc, ProposalResponse,
            ProposalResultResponseJsonRpc, Proposer, Relay, SessionDelete,
            SessionDeleteJsonRpc, SessionExtend, SessionExtendJsonRpc,
            SessionRequest, SessionRequestJsonRpc,
            SessionRequestJsonRpcResponse, SessionSettle, SessionUpdate,
            SettleNamespace,
        },
        relay::IncomingSessionMessage,
        storage::Storage,
        utils::{
            diffie_hellman, generate_rpc_id, is_expired,
            serialize_and_encrypt_message_type0_envelope, topic_from_sym_key,
        },
        verify::{handle_verify, VerifyContext, VERIFY_SERVER_URL},
    },
    relay_rpc::{
        auth::ed25519_dalek::{SecretKey, SigningKey},
        domain::{ProjectId, Topic},
        rpc::{
            AnalyticsData, ApproveSession, FetchMessages, FetchResponse,
            Params, ProposeSession, Publish, Response,
        },
    },
    serde::de::DeserializeOwned,
    sha2::Digest,
    std::{collections::HashMap, sync::Arc},
    tracing::debug,
    x25519_dalek::PublicKey,
};

const RELAY_URL: &str = "wss://relay.walletconnect.org";

// Abstraction for requests that may need Verify API attestation
pub enum MaybeVerifiedRequest {
    Unverified(Params),
    Verified(
        Box<dyn Fn(String) -> Params>,
        tokio::sync::oneshot::Receiver<String>,
    ),
}

// Type aliases to reduce clippy::type-complexity warnings for channel message types
type RpcRequestMessage = (
    MaybeVerifiedRequest,
    tokio::sync::oneshot::Sender<Result<Response, RequestError>>,
);
type RpcRequestSender = tokio::sync::mpsc::UnboundedSender<RpcRequestMessage>;
type RpcRequestReceiver =
    tokio::sync::mpsc::UnboundedReceiver<RpcRequestMessage>;

pub struct Client {
    http_client: reqwest::Client,
    tx: tokio::sync::mpsc::UnboundedSender<(Topic, IncomingSessionMessage)>,
    request_tx: RpcRequestSender,
    online_tx: Option<tokio::sync::mpsc::UnboundedSender<()>>,
    cleanup_tx: Option<tokio_util::sync::CancellationToken>,
    storage: Arc<dyn Storage>,
    // Lazy-start fields for spawning the relay loop
    project_id: ProjectId,
    signing_key_bytes: [u8; 32],
    pending_request_rx: Option<RpcRequestReceiver>,
    pending_online_rx: Option<tokio::sync::mpsc::UnboundedReceiver<()>>,
    probe_group: Option<String>,
}

// Deduplication: does deduplication happen at the irn_subscription layer (like current SDKs) or do we do it for each action e.g. update, event, etc. (remember layered state and stateless architecture)

// TODO
//   - disconnect if no ping for 30s etc. (native only)
//   - back-off w/ random jitter to prevent server overload
//   - online/offline hints
//   - background/foreground hints

// TODO
// - session pings, update, events, emit, extend
// - emit events for session pings, update, events, extend, disconnect
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
// - initial connection request in query param
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
//TODO
// - Validation and Utils methods

// TODO tests
// - memory leak slow tests, run for days?. Kill WS many times over and over again to test. Create many sessions over and over again, update sessions, session requests, etc.
// - test killing the WS, not returning request, failing to connect, etc. in various stages of the lifecycle
// - flow works even when Verify API isblocked: https://github.com/reown-com/appkit/pull/5023

#[allow(unused)]
impl Client {
    pub fn new(
        project_id: ProjectId,
        key: SecretKey,
        session_store: Arc<dyn Storage>,
    ) -> (
        Self,
        tokio::sync::mpsc::UnboundedReceiver<(Topic, IncomingSessionMessage)>,
    ) {
        assert_eq!(
            project_id.value().len(),
            32,
            "Project ID must be exactly 32 characters"
        );
        // TODO validate format i.e. hex

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let (request_tx, request_rx): (RpcRequestSender, RpcRequestReceiver) =
            tokio::sync::mpsc::unbounded_channel();
        let (online_tx, online_rx) = tokio::sync::mpsc::unbounded_channel();
        let cleanup_rx = tokio_util::sync::CancellationToken::new();

        (
            Self {
                http_client: reqwest::Client::new(),
                tx,
                request_tx,
                storage: session_store,
                online_tx: Some(online_tx),
                cleanup_tx: Some(cleanup_rx),
                project_id,
                signing_key_bytes: SigningKey::from_bytes(&key).to_bytes(),
                pending_request_rx: Some(request_rx),
                pending_online_rx: Some(online_rx),
                probe_group: None,
            },
            rx,
        )
    }

    pub fn set_probe_group(&mut self, probe_group: String) {
        self.probe_group = Some(probe_group);
    }

    pub fn start(&mut self) {
        if let (Some(request_rx), Some(online_rx)) =
            (self.pending_request_rx.take(), self.pending_online_rx.take())
        {
            let cleanup_rx = self
                .cleanup_tx
                .as_ref()
                .expect("cleanup token must exist")
                .clone();
            let project_id = self.project_id.clone();
            let signing_key = SigningKey::from_bytes(&self.signing_key_bytes);
            let session_store = self.storage.clone();
            let tx = self.tx.clone();

            crate::spawn::spawn(
                crate::sign::relay::connect_loop_state_machine(
                    RELAY_URL.to_string(),
                    project_id,
                    signing_key,
                    session_store,
                    self.http_client.clone(),
                    tx,
                    request_rx,
                    online_rx,
                    cleanup_rx,
                    self.probe_group.clone(),
                ),
            );
        }
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

    pub async fn pair(
        &mut self,
        uri: &str,
    ) -> Result<(SessionProposal, VerifyContext), PairError> {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L330

        // TODO parse URI and URI validation
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/core/src/controllers/pairing.ts#L132

        let pairing_uri = pairing_uri::parse(uri)
            .map_err(|e| PairError::Internal(e.to_string()))?;

        tracing::debug!("Pairing with URI: {uri}");

        // TODO consider: immediately throw if expired? - maybe not necessary since FetchMessages returns empty array?
        // TODO update relay method to not remove message & approveSession removes it

        let response = self
            .do_request::<FetchResponse>(MaybeVerifiedRequest::Unverified(
                relay_rpc::rpc::Params::FetchMessages(FetchMessages {
                    topic: pairing_uri.topic.clone(),
                }),
            ))
            .await
            .map_err(|e| PairError::Internal(e.to_string()))?;

        tracing::debug!("Pairing Response: {:?}", response);

        let message = response
            .messages
            .iter()
            .find(|message| message.tag == 1100)
            .ok_or(PairError::Internal(
                "No message found with tag 1100".to_owned(),
            ))?;

        if message.topic != pairing_uri.topic {
            return Err(PairError::Internal(format!(
                "Expected topic {}, got {}",
                pairing_uri.topic, message.topic
            )));
        }

        let decrypted =
            decrypt_type0_envelope(pairing_uri.sym_key, &message.message)?;
        let request = serde_json::from_slice::<ProposalJsonRpc>(&decrypted)
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

        let request_json = serde_json::to_string_pretty(&request).unwrap();

        if self.storage.does_json_rpc_exist(request.id).unwrap_or(false) {
            return Err(PairError::Internal(format!(
                "Duplicated JsonRpc RequestId for SessionPropose {}",
                request.id
            )));
        } else {
            self.storage.insert_json_rpc_history(
                request.id,
                pairing_uri.topic.to_string(),
                request.method.clone(),
                request_json,
                Some(TransportType::Relay),
            );
        }

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
                    "Failed to convert proposer public key to fixed-size array"
                        .to_owned(),
                )
            })?;
        tracing::debug!("pairing topic: {:?}", pairing_uri.topic.clone());

        // TODO: validate namespaces: https://specs.walletconnect.com/2.0/specs/clients/sign/namespaces#12-proposal-namespaces-must-not-have-chains-empty

        let decrypted_hash = sha2::Sha256::digest(&decrypted);
        let attestation = handle_verify(
            VERIFY_SERVER_URL.to_string(),
            decrypted_hash.to_vec().try_into().unwrap(),
            self.http_client.clone(),
            self.storage.clone(),
            message.clone(),
            proposal.proposer.metadata.url.clone(),
        )
        .await;

        Ok((
            SessionProposal {
                session_proposal_rpc_id: request.id,
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
            },
            attestation,
        ))
    }

    pub async fn connect(
        &mut self,
        params: ConnectParams,
        self_metadata: Metadata,
    ) -> Result<ConnectResult, ConnectError> {
        // Validate connect parameters
        self.is_valid_connect(&params)?;

        // Always create new pairing topic (reuse is deprecated)
        let pairing_info = Self::create_pairing().await?;
        let uri = pairing_info.uri.clone();
        let sym_key = pairing_info.sym_key.clone().try_into().unwrap();
        let expiry_timestamp = pairing_info.expiry;

        let self_key = x25519_dalek::StaticSecret::random();
        let self_public_key = PublicKey::from(&self_key);

        let session_proposal = Proposal {
            relays: vec![Relay { protocol: "irn".to_string() }],
            required_namespaces: HashMap::new(), // Deprecated, now empty
            optional_namespaces: Some(params.optional_namespaces),
            proposer: Proposer {
                public_key: hex::encode(self_public_key.to_bytes()),
                metadata: self_metadata.clone(),
            },
            session_properties: params.session_properties.clone(),
            scoped_properties: params.scoped_properties.clone(),
            expiry_timestamp: Some(expiry_timestamp),
        };

        let rpc_id = generate_rpc_id();
        let session_proposal_json_rpc = JsonRpcRequest {
            id: rpc_id,
            jsonrpc: "2.0".to_string(),
            method: "wc_sessionPropose".to_string(),
            params: JsonRpcRequestParams::SessionPropose(
                session_proposal.clone(),
            ),
        };
        let session_proposal_params_json =
            serde_json::to_string_pretty(&session_proposal_json_rpc)
                .map_err(|e| ConnectError::ShouldNeverHappen(e.to_string()))?;

        let message = serialize_and_encrypt_message_type0_envelope(
            sym_key,
            &session_proposal_json_rpc,
        )
        .map_err(ConnectError::ShouldNeverHappen)?;

        tracing::debug!(
            group = self.probe_group.clone(),
            probe = "sending_propose_session_request",
        );

        // Create the ProposeSession params
        let pairing_topic = pairing_info.topic.clone();
        let session_proposal_message = message.clone();
        let correlation_id = rpc_id;

        // Create callback that inserts attestation
        let callback = Box::new(move |attestation: String| -> Params {
            Params::ProposeSession(ProposeSession {
                pairing_topic: pairing_topic.clone(),
                session_proposal: session_proposal_message.clone(),
                attestation: Some(attestation.into()),
                analytics: Some(AnalyticsData {
                    correlation_id: Some(correlation_id.try_into().unwrap()),
                    chain_id: None,
                    rpc_methods: None,
                    tx_hashes: None,
                    contract_addresses: None,
                }),
            })
        });

        // Create placeholder channel (will be replaced by state machine)
        let (_attestation_tx, attestation_rx) = tokio::sync::oneshot::channel();

        // Create verified request
        let verified_request =
            MaybeVerifiedRequest::Verified(callback, attestation_rx);

        self.do_request::<bool>(verified_request)
            .await
            .map_err(ConnectError::Request)?;
        tracing::debug!(
            group = self.probe_group.clone(),
            probe = "propose_session_request_success",
        );

        self.storage.insert_json_rpc_history(
            rpc_id,
            pairing_info.topic.to_string(),
            "wc_sessionPropose".to_string(),
            session_proposal_params_json,
            Some(TransportType::Relay),
        );

        self.storage.save_pairing(
            pairing_info.topic.clone(),
            rpc_id,
            sym_key,
            self_key.to_bytes(),
        );
        tracing::debug!(
            group = self.probe_group.clone(),
            probe = "pairing_saved",
            "pairing saved"
        );

        // TODO should return a promise/completer like JS/Flutter or should we just await the on_session_connect event?
        Ok(ConnectResult { topic: pairing_info.topic.clone(), uri })
    }

    async fn create_pairing() -> Result<PairingInfo, ConnectError> {
        let sym_key = x25519_dalek::StaticSecret::random();
        let pairing_topic = topic_from_sym_key(sym_key.as_bytes());
        let expiry = crate::time::SystemTime::now()
            .duration_since(crate::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 5 * 60;
        let relay = Relay { protocol: "irn".to_string() };
        let uri = pairing_uri::format(&pairing_topic, &sym_key, &relay, expiry);
        let pairing_info = PairingInfo {
            topic: pairing_topic.clone(),
            uri,
            sym_key: sym_key.as_bytes().to_vec(),
            expiry,
            relay,
            active: false,
            methods: None,       // TODO: Add methods parameter
            peer_metadata: None, // TODO: Add peer metadata parameter
        };

        // TODO: Store pairing in local storage
        // TODO: Emit pairing created event

        Ok(pairing_info)
    }

    // TODO implement and move to utils
    fn is_valid_connect(
        &self,
        _params: &ConnectParams,
    ) -> Result<(), ConnectError> {
        // TODO: Implement validation logic
        // - Check if namespaces are valid
        // - Validate metadata
        // - Check other constraints
        Ok(())
    }

    pub async fn approve(
        &mut self,
        proposal: SessionProposal,
        approved_namespaces: HashMap<String, SettleNamespace>,
        self_metadata: Metadata,
    ) -> Result<Session, ApproveError> {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L341

        // TODO check is valid: validate namespaces, validate metadata, validate expiry timestamp

        let self_key = x25519_dalek::StaticSecret::random();
        let self_public_key = PublicKey::from(&self_key);
        let shared_secret =
            diffie_hellman(&proposal.proposer_public_key.into(), &self_key);
        let session_topic = topic_from_sym_key(&shared_secret);
        debug!("session topic: {}", session_topic);

        let response_result_json_rpc = ProposalResultResponseJsonRpc {
            id: proposal.session_proposal_rpc_id,
            jsonrpc: "2.0".to_string(),
            result: ProposalResponse {
                relay: Relay { protocol: "irn".to_string() },
                responder_public_key: hex::encode(self_public_key.to_bytes()),
            },
        };

        let response_result_json =
            serde_json::to_string_pretty(&response_result_json_rpc)
                .map_err(|e| ApproveError::ShouldNeverHappen(e.to_string()))?;

        let session_proposal_response = {
            let proposal_response = response_result_json_rpc;
            serialize_and_encrypt_message_type0_envelope(
                proposal.pairing_sym_key,
                &proposal_response,
            )
            .map_err(ApproveError::ShouldNeverHappen)?
        };

        let session_expiry = crate::time::SystemTime::now()
            .duration_since(crate::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 60 * 60 * 24 * 7; // Session expiry is 7 days

        let session_settlement_request_id = generate_rpc_id();
        let session_settlement_request_params = SessionSettle {
            relay: Relay { protocol: "irn".to_string() },
            namespaces: approved_namespaces.clone(),
            controller: Controller {
                public_key: hex::encode(self_public_key.to_bytes()),
                metadata: self_metadata.clone(),
            },
            expiry: session_expiry,
            session_properties: proposal.session_properties.clone(),
            scoped_properties: proposal.scoped_properties.clone(),
        };
        let session_settlement_json_rpc = JsonRpcRequest {
            id: session_settlement_request_id,
            jsonrpc: "2.0".to_string(),
            method: "wc_sessionSettle".to_string(),
            params: JsonRpcRequestParams::SessionSettle(
                session_settlement_request_params.clone(),
            ),
        };
        let session_settlement_request =
            serialize_and_encrypt_message_type0_envelope(
                shared_secret,
                &session_settlement_json_rpc,
            )
            .map_err(ApproveError::ShouldNeverHappen)?;

        let session_settlement_request_params_json =
            serde_json::to_string_pretty(&session_settlement_json_rpc)
                .map_err(|e| ApproveError::ShouldNeverHappen(e.to_string()))?;

        let approve_session = ApproveSession {
            pairing_topic: proposal.pairing_topic.clone(),
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
        let session = Session {
            request_id: proposal.session_proposal_rpc_id,
            session_sym_key: shared_secret,
            self_public_key: self_public_key.to_bytes(),
            topic: session_topic,
            expiry: session_expiry,
            relay_protocol: "irn".to_string(),
            relay_data: None,
            controller_key: Some(self_public_key.to_bytes()),
            self_meta_data: self_metadata.clone(),
            peer_public_key: Some(proposal.proposer_public_key),
            peer_meta_data: Some(proposal.metadata),
            session_namespaces: approved_namespaces,
            required_namespaces: proposal.required_namespaces.clone(),
            optional_namespaces: proposal.optional_namespaces.clone(),
            session_properties: proposal.session_properties.clone(),
            scoped_properties: proposal.scoped_properties.clone(),
            is_acknowledged: false,
            pairing_topic: proposal.pairing_topic.clone(),
            transport_type: None, //TODO: add transport type for link mode
        };

        self.storage.add_session(session.clone());

        match self
            .do_request::<bool>(MaybeVerifiedRequest::Unverified(
                relay_rpc::rpc::Params::ApproveSession(approve_session),
            ))
            .await
        {
            Ok(true) => {
                //Store SessionSettle Request
                self.storage.insert_json_rpc_history(
                    session_settlement_request_id,
                    session.topic.to_string(),
                    "wc_sessionSettle".to_string(),
                    session_settlement_request_params_json,
                    Some(TransportType::Relay),
                );

                //Store SessionApprove Response
                self.storage.update_json_rpc_history_response(
                    proposal.session_proposal_rpc_id,
                    response_result_json,
                );

                self.tx
                    .send((
                        session.topic.clone(),
                        IncomingSessionMessage::SessionConnect(
                            proposal.session_proposal_rpc_id,
                            session.topic.clone(),
                        ),
                    ))
                    .unwrap();
                Ok(session)
            }
            Ok(false) => {
                self.storage.delete_session(session.topic);
                Err(ApproveError::Internal(
                    "Session rejected by relay".to_owned(),
                ))
            }
            Err(e) => {
                self.storage.delete_session(session.topic);
                Err(ApproveError::Request(e))
            }
        }
    }

    // TODO will use storage in the future, for now it's ok to receive the whole proposal as parameter much like on approve() method.
    pub async fn reject(
        &mut self,
        proposal: SessionProposal,
        reason: RejectionReason,
    ) -> Result<(), RejectError> {
        // Check if proposal is expired
        // TODO remove this check: https://reown-inc.slack.com/archives/C098LHLHCNM/p1756148081338769
        if let Some(expiry) = proposal.expiry_timestamp {
            if is_expired(expiry) {
                return Err(RejectError::Internal(format!(
                    "Proposal id {} has expired",
                    proposal.session_proposal_rpc_id
                )));
            }
        }

        // Map enum to ErrorData using centralized conversion
        let mapped_error: relay_rpc::rpc::ErrorData = reason.into();

        // Send error response to the pairing topic
        let error_response = relay_rpc::rpc::ErrorResponse {
            id: relay_rpc::domain::MessageId::new(
                proposal.session_proposal_rpc_id,
            ),
            jsonrpc: relay_rpc::rpc::JSON_RPC_VERSION.clone(),
            error: mapped_error,
        };

        let response_result_json =
            serde_json::to_string_pretty(&error_response)
                .map_err(|e| RejectError::ShouldNeverHappen(e.to_string()))?;

        let message = serialize_and_encrypt_message_type0_envelope(
            proposal.pairing_sym_key,
            &error_response,
        )
        .map_err(RejectError::ShouldNeverHappen)?;

        // Publish error response to pairing topic
        match self
            .do_request::<bool>(MaybeVerifiedRequest::Unverified(
                relay_rpc::rpc::Params::Publish(Publish {
                    topic: proposal.pairing_topic.clone(),
                    message,
                    attestation: None, // TODO
                    ttl_secs: 300,
                    tag: 1120,
                    prompt: false,
                    analytics: Some(AnalyticsData {
                        correlation_id: Some(
                            proposal
                                .session_proposal_rpc_id
                                .try_into()
                                .unwrap(),
                        ),
                        chain_id: None,
                        rpc_methods: None,
                        tx_hashes: None,
                        contract_addresses: None,
                    }),
                }),
            ))
            .await
        {
            Ok(true) => {
                self.storage.update_json_rpc_history_response(
                    proposal.session_proposal_rpc_id,
                    response_result_json,
                );
                Ok(())
            }
            Ok(false) => {
                // we don't need delete from storage from rust side (like on approve method does for session) as is not implemented for proposal
                // proposal will be deleted from each SDK storage.
                Err(RejectError::Internal(
                    "Failed to send rejection to relay".to_string(),
                ))
            }
            Err(e) => Err(RejectError::Request(e)),
        }
    }

    pub async fn _extend(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L569
        unimplemented!()
    }

    pub async fn request(
        &mut self,
        topic: Topic,
        session_request: SessionRequest,
    ) -> Result<(u64), RequestError> {
        let shared_secret = self
            .storage
            .get_session(topic.clone())
            .map_err(|e| RequestError::Internal(e.to_string()))?
            .map(|s| s.session_sym_key)
            .unwrap();

        let rpc = SessionRequestJsonRpc {
            id: generate_rpc_id(),
            jsonrpc: "2.0".to_string(),
            method: "wc_sessionRequest".to_string(),
            params: session_request,
        };
        let message =
            serialize_and_encrypt_message_type0_envelope(shared_secret, &rpc)
                .map_err(|e| RequestError::Internal(e.to_string()))?;

        // Create the Publish params
        let publish_topic = topic.clone();
        let publish_message = message.clone();

        // Create callback that inserts attestation
        let callback = Box::new(move |attestation: String| -> Params {
            Params::Publish(Publish {
                topic: publish_topic.clone(),
                message: publish_message.clone(),
                attestation: Some(attestation.into()),
                ttl_secs: 300,
                tag: 1108,
                prompt: false,
                analytics: None,
            })
        });

        // Create placeholder channel (will be replaced by state machine)
        let (_attestation_tx, attestation_rx) = tokio::sync::oneshot::channel();

        // Create verified request
        let verified_request =
            MaybeVerifiedRequest::Verified(callback, attestation_rx);

        self.do_request::<bool>(verified_request)
            .await
            .map_err(|e| RequestError::Internal(e.to_string()))?;

        // TODO WS handling:
        // - when a session request is pending, and we get the event that the page regained focus, should we immediately ping the WS connection to test its liveness (?)

        let session_request_params_json = serde_json::to_string_pretty(&rpc)
            .map_err(|e| RequestError::Internal(e.to_string()))?;

        self.storage.insert_json_rpc_history(
            rpc.id,
            topic.to_string(),
            rpc.method,
            session_request_params_json,
            Some(TransportType::Relay),
        );

        Ok(rpc.id)
    }

    pub async fn respond(
        &mut self,
        topic: Topic,
        response: SessionRequestJsonRpcResponse,
    ) -> Result<(), RespondError> {
        let shared_secret = self
            .storage
            .get_session(topic.clone())
            .map_err(RespondError::Storage)?
            .map(|s| s.session_sym_key)
            .ok_or(RespondError::SessionNotFound)?;

        let message = serialize_and_encrypt_message_type0_envelope(
            shared_secret,
            &response,
        )
        .map_err(RespondError::ShouldNeverHappen)?;

        self.do_request::<bool>(MaybeVerifiedRequest::Unverified(
            relay_rpc::rpc::Params::Publish(Publish {
                topic,
                message,
                attestation: None, // TODO
                ttl_secs: 300,
                tag: 1109,
                prompt: false,
                analytics: Some(AnalyticsData {
                    correlation_id: Some(
                        match &response {
                            SessionRequestJsonRpcResponse::Result(r) => r.id,
                            SessionRequestJsonRpcResponse::Error(e) => e.id,
                        }
                        .try_into()
                        .unwrap(),
                    ),
                    chain_id: None,           // TODO
                    rpc_methods: None,        // TODO
                    tx_hashes: None,          // TODO
                    contract_addresses: None, // TODO
                }),
            }),
        ))
        .await
        .map_err(RespondError::Request)?;

        let response_result_json = serde_json::to_string_pretty(&response)
            .map_err(|e| RespondError::ShouldNeverHappen(e.to_string()))?;

        match &response {
            SessionRequestJsonRpcResponse::Result(r) => {
                self.storage.update_json_rpc_history_response(
                    r.id,
                    response_result_json,
                );
            }
            SessionRequestJsonRpcResponse::Error(e) => {
                self.storage.update_json_rpc_history_response(
                    e.id,
                    response_result_json,
                );
            }
        }

        Ok(())
    }

    pub async fn update(
        &mut self,
        topic: Topic,
        namespaces: std::collections::HashMap<String, SettleNamespace>,
    ) -> Result<(), UpdateError> {
        //TODO: add validate namespaces

        let session_opt = self
            .storage
            .get_session(topic.clone())
            .map_err(UpdateError::Storage)?;
        let shared_secret = session_opt
            .as_ref()
            .map(|s| s.session_sym_key)
            .ok_or(UpdateError::SessionNotFound)?;

        // validateController: only the controller can send updates
        if let Some(session) = &session_opt {
            if session.controller_key != Some(session.self_public_key) {
                return Err(UpdateError::Unauthorized);
            }
        }

        // Update local storage immediately
        if let Some(mut session) = session_opt {
            session.session_namespaces = namespaces.clone();
            self.storage.add_session(session);
        }

        let id = generate_rpc_id();
        let session_update_json_rpc =
            crate::sign::protocol_types::SessionUpdateJsonRpc {
                id,
                jsonrpc: "2.0".to_string(),
                method: "wc_sessionUpdate".to_string(),
                params: SessionUpdate { namespaces: namespaces.clone() },
            };
        let namespaces_params_json =
            serde_json::to_string_pretty(&session_update_json_rpc)
                .map_err(|e| UpdateError::ShouldNeverHappen(e.to_string()))?;

        let message = serialize_and_encrypt_message_type0_envelope(
            shared_secret,
            &session_update_json_rpc,
        )
        .map_err(UpdateError::ShouldNeverHappen)?;

        self.do_request::<bool>(MaybeVerifiedRequest::Unverified(
            relay_rpc::rpc::Params::Publish(Publish {
                topic: topic.clone(),
                message,
                attestation: None,
                ttl_secs: 86400,
                tag: 1104,
                prompt: false,
                analytics: Some(AnalyticsData {
                    correlation_id: Some(id.try_into().unwrap()),
                    chain_id: None,
                    rpc_methods: None,
                    tx_hashes: None,
                    contract_addresses: None,
                }),
            }),
        ))
        .await
        .map_err(UpdateError::Request)?;

        self.storage.insert_json_rpc_history(
            id,
            topic.to_string(),
            "wc_sessionUpdate".to_string(),
            namespaces_params_json,
            Some(TransportType::Relay),
        );

        Ok(())
    }

    /// Extend session by 7 days from now
    pub async fn extend(&mut self, topic: Topic) -> Result<(), ExtendError> {
        let mut session = self
            .storage
            .get_session(topic.clone())
            .map_err(ExtendError::Storage)?
            .ok_or(ExtendError::SessionNotFound)?;

        // Compute new expiry = now + 7 days
        let now = crate::time::SystemTime::now()
            .duration_since(crate::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let new_expiry = now + 60 * 60 * 24 * 7;

        // Must be strictly increasing, and at most 7 days from now (already enforced)
        if new_expiry <= session.expiry {
            return Err(ExtendError::InvalidExpiry);
        }

        // Update local storage first
        session.expiry = new_expiry;
        let shared_secret = session.session_sym_key;
        self.storage.add_session(session);
        let id = generate_rpc_id();
        let session_extend_json_rpc = SessionExtendJsonRpc {
            id,
            jsonrpc: "2.0".to_string(),
            method: "wc_sessionExtend".to_string(),
            params: SessionExtend { expiry: new_expiry },
        };
        let expiry_rpc_json =
            serde_json::to_string_pretty(&session_extend_json_rpc)
                .map_err(|e| ExtendError::ShouldNeverHappen(e.to_string()))?;

        let message = serialize_and_encrypt_message_type0_envelope(
            shared_secret,
            &session_extend_json_rpc,
        )
        .map_err(ExtendError::ShouldNeverHappen)?;

        self.do_request::<bool>(MaybeVerifiedRequest::Unverified(
            relay_rpc::rpc::Params::Publish(Publish {
                topic: topic.clone(),
                message,
                attestation: None,
                ttl_secs: 86400,
                tag: 1106,
                prompt: false,
                analytics: Some(AnalyticsData {
                    correlation_id: Some(id.try_into().unwrap()),
                    chain_id: None,
                    rpc_methods: None,
                    tx_hashes: None,
                    contract_addresses: None,
                }),
            }),
        ))
        .await
        .map_err(ExtendError::Request)?;

        self.storage.insert_json_rpc_history(
            id,
            topic.to_string(),
            "wc_sessionExtend".to_string(),
            expiry_rpc_json,
            Some(TransportType::Relay),
        );

        Ok(())
    }
    pub async fn _ping(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L727
        unimplemented!()
    }

    pub async fn emit(
        &mut self,
        topic: Topic,
        name: String,
        data: serde_json::Value,
        chain_id: String,
    ) -> Result<(), EmitError> {
        let shared_secret = self
            .storage
            .get_session(topic.clone())
            .map_err(EmitError::Storage)?
            .map(|s| s.session_sym_key)
            .ok_or(EmitError::SessionNotFound)?;

        let id = generate_rpc_id();
        let session_event_json_rpc =
            crate::sign::protocol_types::SessionEventJsonRpc {
                id,
                jsonrpc: "2.0".to_string(),
                method: "wc_sessionEvent".to_string(),
                params: crate::sign::protocol_types::EventParams {
                    event: crate::sign::protocol_types::SessionEventVO {
                        name,
                        data,
                    },
                    chain_id,
                },
            };
        let event_json = serde_json::to_string_pretty(&session_event_json_rpc)
            .map_err(|e| EmitError::ShouldNeverHappen(e.to_string()))?;
        let message = serialize_and_encrypt_message_type0_envelope(
            shared_secret,
            &session_event_json_rpc,
        )
        .map_err(EmitError::ShouldNeverHappen)?;

        self.do_request::<bool>(MaybeVerifiedRequest::Unverified(
            relay_rpc::rpc::Params::Publish(Publish {
                topic: topic.clone(),
                message,
                attestation: None,
                ttl_secs: 86400,
                tag: 1110,
                prompt: false,
                analytics: Some(AnalyticsData {
                    correlation_id: Some(id.try_into().unwrap()),
                    chain_id: None,
                    rpc_methods: None,
                    tx_hashes: None,
                    contract_addresses: None,
                }),
            }),
        ))
        .await
        .map_err(EmitError::Request)?;

        self.storage.insert_json_rpc_history(
            id,
            topic.to_string(),
            "wc_sessionEvent".to_string(),
            event_json,
            Some(TransportType::Relay),
        );

        Ok(())
    }

    pub async fn disconnect(
        &mut self,
        topic: Topic,
    ) -> Result<(), DisconnectError> {
        let shared_secret = self
            .storage
            .get_session(topic.clone())
            .map_err(DisconnectError::Storage)?
            .map(|s| s.session_sym_key);

        if let Some(shared_secret) = shared_secret {
            let id = generate_rpc_id();
            let session_delete_json_rpc = SessionDeleteJsonRpc {
                id,
                jsonrpc: "2.0".to_string(),
                method: "wc_sessionDelete".to_string(),
                params: SessionDelete {
                    code: 6000,
                    message: "User disconnected.".to_string(),
                },
            };
            let delete_json =
                serde_json::to_string_pretty(&session_delete_json_rpc)
                    .map_err(|e| {
                        DisconnectError::ShouldNeverHappen(e.to_string())
                    })?;
            let message = serialize_and_encrypt_message_type0_envelope(
                shared_secret,
                &session_delete_json_rpc,
            )
            .map_err(DisconnectError::ShouldNeverHappen)?;

            self.do_request::<bool>(MaybeVerifiedRequest::Unverified(
                relay_rpc::rpc::Params::Publish(Publish {
                    topic: topic.clone(),
                    message,
                    attestation: None, // TODO
                    ttl_secs: 86400,
                    tag: 1112,
                    prompt: false,
                    analytics: None, // TODO
                }),
            ))
            .await
            .map_err(DisconnectError::Request)?;

            self.storage.insert_json_rpc_history(
                id,
                topic.to_string(),
                "wc_sessionDelete".to_string(),
                delete_json,
                Some(TransportType::Relay),
            );

            self.storage.delete_session(topic.clone());

            self.tx
                .send((
                    topic.clone(),
                    IncomingSessionMessage::Disconnect(id, topic),
                ))
                .unwrap();
        } else {
            tracing::debug!(
                "disconnect: session not found for topic, ignoring: {:?}",
                topic
            );
        }

        Ok(())
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

    async fn do_request<T: DeserializeOwned>(
        &mut self,
        request: MaybeVerifiedRequest,
    ) -> Result<T, RequestError> {
        tracing::debug!("Connect: Call");

        let (response_tx, response_rx) = tokio::sync::oneshot::channel();
        self.request_tx.send((request, response_tx)).map_err(|e| {
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

impl Drop for Client {
    fn drop(&mut self) {
        if let Some(cleanup_tx) = self.cleanup_tx.take() {
            // Just drop the sender - this closes the channel and signals cleanup
            drop(cleanup_tx);
        } else {
            tracing::warn!("cleanup_tx already taken");
        }
    }
}

pub fn generate_client_id_key() -> SecretKey {
    SigningKey::generate(&mut rand::thread_rng()).to_bytes()
}
