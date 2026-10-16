//! The client: pairing, sessions, the send and receive pipelines, events and lifecycle.
//! Guide: `V3_RUST_SCOPE.md` §3-§7 in the monorepo.

pub mod poller;
pub mod state;
pub mod types;

use {
    self::{
        state::*,
        types::{
            ConnectResult, Namespaces, ProposalParams, RpcError, RpcRequest,
            RpcResponse, Session, SessionEventData, SessionProposal,
            SignConfig, SignError, SignEvent,
        },
    },
    crate::{
        host::{Listener, Storage},
        protocol::{
            constants::{
                ANSWER_WINDOW_S, CLOCK_SKEW_TOLERANCE_S, DEFAULT_DEADLINE_S,
                LOCAL_GRACE_S, MAX_DEADLINE_S, MAX_LIFETIME_S, MAX_TTL_S,
                MIN_TTL_S, SUPPORTED_VERSIONS,
            },
            crypto::{
                derive_binding_key, derive_sealing_keys, is_valid_pairing_pub,
                new_pairing_key, new_sym_key,
            },
            encoding::{b64u, from_b64u},
            ijson,
            intent::{bind_compatibility, satisfies},
            message::{
                BuildMessage, build_message, build_signed, close_label,
                is_request, labels, open_message_payload, open_signed,
                read_message, sign_bytes, verify_bytes, verify_message,
            },
            types::{
                BindingPayload, BindingProofPayload, DappIntent, Metadata,
                Mode, PeerRequirement, RejectionPayload, RelayEvent, Role,
                SignedObject, WalletIntent, WireMessage, is_sealable,
                may_role_send, method_tag, tag_info, tags,
            },
            uri::{build_uri, is_connection_id, parse_uri},
        },
        rt::{self, BoxFut},
        transport::{HttpClient, relay::Relay},
    },
    ed25519_dalek::SigningKey,
    futures::{
        FutureExt,
        channel::{mpsc, oneshot},
    },
    serde_json::{Value, json},
    std::{
        collections::HashMap,
        sync::{
            Arc, Mutex,
            atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering},
        },
    },
    zeroize::Zeroizing,
};

pub(crate) struct State {
    pub conns: HashMap<String, Conn>,
    pub reqs: HashMap<String, Req>,
    pub props: HashMap<String, Prop>,
    pub cursor: Option<String>,
    pub n: u64,
}

struct Pending {
    tx: oneshot::Sender<Result<Value, SignError>>,
    conn: String,
    seq: u64,
}

#[derive(Default)]
pub(crate) struct PollCtl {
    pub running: bool,
    pub generation: u64,
    pub kick: Option<mpsc::UnboundedSender<()>>,
}

pub(crate) struct Inner {
    pub cfg: SignConfig,
    pub versions: Vec<u32>,
    pub store: Store,
    pub relay: Relay,
    pub listener: Arc<dyn Listener>,
    pub state: futures::lock::Mutex<State>,
    pending: Mutex<HashMap<String, Pending>>,
    pub poll: Mutex<PollCtl>,
    pub clock_offset_s: AtomicI64,
    pub superseded: AtomicBool,
    pub last_byte_ms: AtomicU64,
    pub first_line_seen: AtomicBool,
    pub liveness_dead: AtomicBool,
    pub stats: Stats,
}

#[derive(Default)]
pub struct Stats {
    pub duplicates_dropped: AtomicU64,
    pub invalid_dropped: AtomicU64,
    pub late_dropped: AtomicU64,
    pub polls: AtomicU64,
    pub first_lines: AtomicU64,
}

#[derive(Clone)]
pub struct SignClient {
    pub(crate) inner: Arc<Inner>,
}

static RPC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn new_rpc_id() -> i64 {
    (rt::now_ms() * 1000 + RPC_COUNTER.fetch_add(1, Ordering::Relaxed) % 1000)
        as i64
}

fn err(code: &str, message: impl Into<String>) -> SignError {
    SignError::new(code, message)
}

/// JSON passed in by the app, through the crate's one parser.
fn json_arg(s: &str) -> Result<Value, SignError> {
    ijson::parse(s).map(|p| p.value).map_err(|e| err("invalid_params", e.0))
}

fn relay_json(b: &[u8]) -> Result<Value, SignError> {
    ijson::value(b).ok_or_else(|| err("relay_error", "response is not JSON"))
}

fn pkey(conn: &str, id: i64) -> String {
    format!("{conn}:{id}")
}

impl SignClient {
    /// Loads or creates the client identity and restores all state. Polling starts with `start()`,
    /// so a listener is always in place before the first event arrives.
    pub async fn init(
        cfg: SignConfig,
        storage: Arc<dyn Storage>,
        listener: Arc<dyn Listener>,
    ) -> Result<Self, SignError> {
        #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
        let http: Arc<dyn HttpClient> =
            Arc::new(crate::transport::native::ReqwestClient::default());
        #[cfg(all(feature = "wasm", target_arch = "wasm32"))]
        let http: Arc<dyn HttpClient> =
            Arc::new(crate::transport::wasm::FetchClient);
        Self::init_with_http(cfg, storage, listener, http).await
    }

    pub async fn init_with_http(
        cfg: SignConfig,
        storage: Arc<dyn Storage>,
        listener: Arc<dyn Listener>,
        http: Arc<dyn HttpClient>,
    ) -> Result<Self, SignError> {
        if cfg.hosts.is_empty() {
            return Err(err("invalid_config", "hosts must not be empty"));
        }
        let store = Store::new(storage, cfg.storage_prefix.clone());
        let auth = match store
            .get::<String>("secret/client")
            .await?
            .and_then(|s| key_from_b64(&s))
        {
            Some(k) => k,
            None => {
                let k = new_pairing_key();
                store.put("secret/client", &key_to_b64(&k)).await?;
                k
            }
        };
        let versions = if cfg.versions.is_empty() {
            SUPPORTED_VERSIONS.to_vec()
        } else {
            cfg.versions.clone()
        };
        let relay = Relay::new(
            http,
            cfg.hosts.clone(),
            cfg.audience.clone(),
            cfg.project_id.clone(),
            auth,
        );
        let inner = Arc::new(Inner {
            cfg,
            versions,
            store,
            relay,
            listener,
            state: futures::lock::Mutex::new(State {
                conns: HashMap::new(),
                reqs: HashMap::new(),
                props: HashMap::new(),
                cursor: None,
                n: 0,
            }),
            pending: Mutex::new(HashMap::new()),
            poll: Mutex::new(PollCtl::default()),
            clock_offset_s: AtomicI64::new(0),
            superseded: AtomicBool::new(false),
            last_byte_ms: AtomicU64::new(0),
            first_line_seen: AtomicBool::new(false),
            liveness_dead: AtomicBool::new(false),
            stats: Stats::default(),
        });
        inner.restore().await?;
        Ok(Self { inner })
    }

    pub fn client_id(&self) -> String {
        self.inner.relay.client_id()
    }

    pub fn stats(&self) -> &Stats {
        &self.inner.stats
    }

    pub fn relay(&self) -> &Relay {
        &self.inner.relay
    }

    pub fn is_polling(&self) -> bool {
        self.inner.poll.lock().expect("lock").running
    }

    pub fn is_superseded(&self) -> bool {
        self.inner.superseded.load(Ordering::SeqCst)
    }

    pub fn clock_offset_s(&self) -> i64 {
        self.inner.clock_offset_s.load(Ordering::SeqCst)
    }

    pub fn set_clock_offset_s(&self, v: i64) {
        self.inner.clock_offset_s.store(v, Ordering::SeqCst);
    }

    pub fn start(&self) {
        poller::start(&self.inner);
    }

    pub fn stop(&self) {
        poller::stop(&self.inner);
    }

    /// Foreground, online or pageshow: run the liveness check at once (RCV-09). On native a
    /// superseded poller resumes; on web a superseded tab waits for its next call (guide §10.1).
    pub fn foreground(&self) {
        let i = &self.inner;
        if !i.poll.lock().expect("lock").running {
            if i.superseded.load(Ordering::SeqCst)
                && cfg!(target_arch = "wasm32")
            {
                return;
            }
            self.start();
            return;
        }
        let stale = i.relay.config().foreground_stale_ms;
        if i.first_line_seen.load(Ordering::SeqCst)
            && rt::now_ms()
                .saturating_sub(i.last_byte_ms.load(Ordering::SeqCst))
                > stale
        {
            i.liveness_dead.store(true, Ordering::SeqCst);
            poller::kick(i);
        }
    }

    pub async fn sessions(&self) -> Vec<Session> {
        self.inner
            .state
            .lock()
            .await
            .conns
            .values()
            .map(|c| session_of(&c.rec))
            .collect()
    }

    // ------------------------------------------------------------ dApp

    /// `params` is v2's proposal params minus the transport fields. The dApp metadata is added.
    pub async fn connect(
        &self,
        params: ProposalParams,
        intent: Option<DappIntent>,
    ) -> Result<ConnectResult, SignError> {
        self.inner.ensure_active().await?;
        let mut params = serde_json::to_value(&params).expect("params");
        params["metadata"] =
            serde_json::to_value(&self.inner.cfg.metadata).expect("metadata");
        let intent = intent.unwrap_or(DappIntent {
            own: Mode::Monitored,
            peer: PeerRequirement::None,
        });
        let sym = new_sym_key();
        let binding = derive_binding_key(&sym);
        let bpub = binding.verifying_key().to_bytes();
        let pairing = new_pairing_key();
        let proposal_id = new_rpc_id();
        let proof = BindingProofPayload {
            versions: self.inner.versions.clone(),
            intent,
            pairing_pub: b64u(&pairing.verifying_key().to_bytes()),
        };
        let rpc = json!({ "id": proposal_id, "jsonrpc": "2.0", "method": "wc_sessionCreate", "params": params });
        let body = json!({
            "bindingPub": b64u(&bpub),
            "bindingProof": build_signed(&binding, labels::BINDING_PROOF, &bpub, &proof),
            "proposal": build_message(BuildMessage {
                key: &pairing, version: 0, binding_pub: &bpub, tag: tags::PROPOSAL, seq: 0,
                deadline: None, rpc: &rpc, seal_key: None, nonce: None,
            }),
        });
        let res = self
            .inner
            .relay
            .call(
                "POST",
                "/v3/connections",
                Some(serde_json::to_vec(&body).expect("json")),
                vec![],
            )
            .await?;
        let res: Value = relay_json(&res)?;
        let topic = res["connectionId"]
            .as_str()
            .filter(|id| is_connection_id(id))
            .ok_or_else(|| err("relay_error", "missing connectionId"))?
            .to_string();
        let expires_at = res["expiresAt"]
            .as_u64()
            .ok_or_else(|| err("relay_error", "missing expiresAt"))?;
        let i = &self.inner;
        // The Relay keeps one pending connection per dApp: this one replaced any earlier one.
        let replaced: Vec<String> = i
            .state
            .lock()
            .await
            .reqs
            .values()
            .filter(|r| !r.rec.failed)
            .map(|r| r.rec.id.clone())
            .collect();
        for t in replaced {
            i.fail_request(&t, "replaced").await?;
        }
        let rec = DappReq {
            id: topic.clone(),
            intent,
            versions: self.inner.versions.clone(),
            proposal_id,
            expires_at,
            failed: false,
        };
        i.store
            .put(
                &format!("secret/req/{topic}"),
                &ReqSecrets {
                    sym: b64u(sym.as_ref()),
                    pairing: key_to_b64(&pairing),
                },
            )
            .await?;
        i.store.put(&format!("req/{topic}"), &rec).await?;
        let uri = build_uri(&topic, &sym, expires_at);
        i.state
            .lock()
            .await
            .reqs
            .insert(topic.clone(), Req { rec, sym, key: pairing });
        Ok(ConnectResult { uri, topic })
    }

    // ------------------------------------------------------------ wallet

    pub async fn pair(
        &self,
        uri: String,
    ) -> Result<SessionProposal, SignError> {
        self.inner.ensure_active().await?;
        let i = &self.inner;
        let u =
            parse_uri(&uri).map_err(|e| err("invalid_uri", e.to_string()))?;
        if u.legacy {
            return Err(err(
                "unsupported_uri",
                "v2 URI with a topic: v3↔v2 is not supported",
            ));
        }
        let id = u.connection_id.clone();
        let binding = derive_binding_key(&u.sym);
        let bpub = binding.verifying_key().to_bytes();
        let sig = sign_bytes(&binding, labels::FETCH, &bpub, id.as_bytes());
        let res = i
            .relay
            .call(
                "GET",
                &format!("/v3/connections/{id}"),
                None,
                vec![("wc-signature".into(), sig)],
            )
            .await?;
        let fetched: Value = relay_json(&res)?;
        // PAIR-09: a failed check makes the request invalid; show nothing, neither bind nor reject.
        let checked = (|| -> Result<_, SignError> {
            let proof_obj: SignedObject =
                crate::protocol::ijson::from_value::<SignedObject>(
                    &fetched["bindingProof"],
                )
                .ok_or_else(|| err("invalid_request", "bindingProof"))?;
            let proof =
                open_signed(&bpub, labels::BINDING_PROOF, &bpub, &proof_obj)
                    .map_err(|e| err("invalid_request", e.to_string()))?;
            let versions: Vec<u32> = (0..proof.value["versions"]
                .as_array()
                .map(Vec::len)
                .unwrap_or(0))
                .map(|k| proof.int(&format!("/versions/{k}")).map(|v| v as u32))
                .collect::<Result<_, _>>()
                .map_err(|e| err("invalid_request", e.0))?;
            let intent: DappIntent = crate::protocol::ijson::from_value::<
                DappIntent,
            >(&proof.value["intent"])
            .ok_or_else(|| err("invalid_request", "intent"))?;
            let dapp_pub_s = proof.value["pairingPub"]
                .as_str()
                .ok_or_else(|| err("invalid_request", "pairingPub"))?
                .to_string();
            let dapp_pub = from_b64u(&dapp_pub_s)?;
            if versions.is_empty() || !is_valid_pairing_pub(&dapp_pub) {
                return Err(err("invalid_request", "versions or pairingPub"));
            }
            let wire: WireMessage = crate::protocol::ijson::from_value::<
                WireMessage,
            >(&fetched["proposal"])
            .ok_or_else(|| err("invalid_request", "proposal"))?;
            let m = read_message(&wire)
                .map_err(|e| err("invalid_request", e.to_string()))?;
            if m.plain.tag != tags::PROPOSAL
                || m.plain.seq != 0
                || m.plain.sealed
                || !verify_message(&dapp_pub, 0, &bpub, &m)
            {
                return Err(err("invalid_request", "proposal signature"));
            }
            let rpc = open_message_payload(&m, 0, &bpub, None)
                .map_err(|e| err("invalid_request", e.to_string()))?;
            if rpc.value["method"] != "wc_sessionCreate" {
                return Err(err("invalid_request", "proposal method"));
            }
            Ok((versions, intent, dapp_pub_s, rpc))
        })()?;
        let (dapp_versions, dapp_intent, dapp_pub, rpc) = checked;
        let relay_versions: Vec<u32> = fetched["versions"]
            .as_array()
            .map(|a| {
                a.iter().filter_map(|v| v.as_u64()).map(|v| v as u32).collect()
            })
            .unwrap_or_default();
        let version = dapp_versions
            .iter()
            .copied()
            .filter(|v| i.versions.contains(v) && relay_versions.contains(v))
            .max()
            .unwrap_or(0);
        let params = rpc.value["params"].clone();
        let metadata: Metadata =
            crate::protocol::ijson::from_value::<Metadata>(&params["metadata"])
                .unwrap_or(Metadata {
                    name: String::new(),
                    description: String::new(),
                    url: String::new(),
                    icons: vec![],
                });
        let rec = WalletProp {
            id: id.clone(),
            dapp_intent,
            dapp_pub,
            dapp_versions: dapp_versions.clone(),
            version,
            sealable: fetched["sealable"].as_bool().unwrap_or(false),
            proposal_id: rpc
                .int("/id")
                .map_err(|e| err("invalid_request", e.0))?,
            params: params.clone(),
            metadata: metadata.clone(),
            expires_at: u.expiry_timestamp,
            pending: None,
        };
        i.store
            .put(
                &format!("secret/prop/{id}"),
                &PropSecrets { sym: b64u(u.sym.as_ref()), pairing: None },
            )
            .await?;
        i.store.put(&format!("prop/{id}"), &rec).await?;
        i.state.lock().await.props.insert(
            id.clone(),
            Prop { rec: rec.clone(), sym: u.sym, key: None },
        );
        if version == 0 {
            self.reject(id.clone(), Some("unsupported_version".into())).await?;
            return Err(err(
                "unsupported_version",
                "no version shared by the dApp, this wallet and the Relay",
            ));
        }
        if let Some(req) = i.cfg.require_dapp
            && !satisfies(dapp_intent.own, req)
        {
            self.reject(id.clone(), Some("incompatible_intent".into())).await?;
            return Err(err(
                "incompatible_intent",
                "the dApp's direction does not meet this wallet's requirement",
            ));
        }
        let proposal = SessionProposal {
            id,
            proposal_id: rec.proposal_id,
            params: ijson::from_value(&params).unwrap_or_default(),
            metadata,
            intent: dapp_intent,
            versions: dapp_versions,
            version,
            sealable: rec.sealable,
            expires_at: rec.expires_at,
        };
        i.emit(SignEvent::SessionProposal { proposal: proposal.clone() });
        Ok(proposal)
    }

    pub async fn approve(
        &self,
        topic: String,
        namespaces: Namespaces,
        own: Option<Mode>,
    ) -> Result<Session, SignError> {
        self.inner.ensure_active().await?;
        let i = &self.inner;
        let namespaces = serde_json::to_value(&namespaces).expect("namespaces");
        let (body, pairing, own, expiry, namespaces, rec, sym) = {
            let mut st = i.state.lock().await;
            let prop = st
                .props
                .get_mut(&topic)
                .ok_or_else(|| err("not_found", "unknown proposal"))?;
            if prop.rec.pending.is_none() {
                let own = own.unwrap_or(Mode::Monitored);
                if own == Mode::Sealed && !prop.rec.sealable {
                    return Err(err(
                        "incompatible_intent",
                        "the dApp's project may not seal",
                    ));
                }
                if let Some(why) = bind_compatibility(
                    &prop.rec.dapp_intent,
                    &WalletIntent { own },
                    prop.rec.sealable,
                ) {
                    return Err(err("incompatible_intent", why));
                }
                let binding = derive_binding_key(&prop.sym);
                let bpub = binding.verifying_key().to_bytes();
                let pairing = new_pairing_key();
                let result = json!({ "namespaces": namespaces, "metadata": i.cfg.metadata });
                let rpc = json!({ "id": prop.rec.proposal_id, "jsonrpc": "2.0", "result": result });
                let expiry = i.now_s() + MAX_LIFETIME_S;
                let payload = BindingPayload {
                    connection_id: topic.clone(),
                    version: prop.rec.version,
                    pairing_pub: b64u(&pairing.verifying_key().to_bytes()),
                    intent: WalletIntent { own },
                    expiry,
                    approval: build_message(BuildMessage {
                        key: &pairing,
                        version: 0,
                        binding_pub: &bpub,
                        tag: tags::APPROVAL,
                        seq: 0,
                        deadline: None,
                        rpc: &rpc,
                        seal_key: None,
                        nonce: None,
                    }),
                };
                let body = json!({ "binding": build_signed(&binding, labels::BIND, &bpub, &payload) });
                // Checklist: persist everything and the signed binding before sending (PAIR-16).
                i.store
                    .put(
                        &format!("secret/prop/{topic}"),
                        &PropSecrets {
                            sym: b64u(prop.sym.as_ref()),
                            pairing: Some(key_to_b64(&pairing)),
                        },
                    )
                    .await?;
                prop.rec.pending = Some(PendingAnswer::Bind {
                    body,
                    own,
                    expiry,
                    namespaces: namespaces.clone(),
                });
                prop.key = Some(pairing);
                i.store.put(&format!("prop/{topic}"), &prop.rec).await?;
            }
            let Some(PendingAnswer::Bind { body, own, expiry, namespaces }) =
                prop.rec.pending.clone()
            else {
                return Err(err("request_resolved", "already rejected"));
            };
            let pairing = prop
                .key
                .clone()
                .ok_or_else(|| err("storage", "pairing key missing"))?;
            (
                body,
                pairing,
                own,
                expiry,
                namespaces,
                prop.rec.clone(),
                Zeroizing::new(*prop.sym),
            )
        };
        i.answer(&format!("/v3/connections/{topic}/acceptance"), &body).await?;
        let id = topic.clone();
        let bpub = derive_binding_key(&sym).verifying_key().to_bytes();
        let dapp_pub: [u8; 32] =
            arr32(&rec.dapp_pub).ok_or_else(|| err("storage", "dapp pub"))?;
        let wallet_pub = pairing.verifying_key().to_bytes();
        let keys = derive_sealing_keys(
            rec.version,
            &pairing,
            &dapp_pub,
            &sym,
            &bpub,
            &dapp_pub,
            &wallet_pub,
        )?;
        let conn = Conn {
            rec: ConnRecord {
                id: id.clone(),
                role: Role::Wallet,
                version: rec.version,
                binding_pub: b64u(&bpub),
                dapp_pub: rec.dapp_pub.clone(),
                wallet_pub: b64u(&wallet_pub),
                dapp_mode: rec.dapp_intent.own,
                wallet_mode: own,
                namespaces,
                peer_metadata: rec.metadata.clone(),
                expiry,
                next_seq: 1,
                recv: Default::default(),
                applied_update: 0,
                applied_extend: 0,
            },
            key: pairing,
            d2w: keys.d2w,
            w2d: keys.w2d,
        };
        let session = session_of(&conn.rec);
        let mut st = i.state.lock().await;
        i.save_conn(&conn, true).await?;
        st.conns.insert(id, conn);
        st.props.remove(&topic);
        drop(st);
        i.drop_prop(&topic).await?;
        Ok(session)
    }

    pub async fn reject(
        &self,
        topic: String,
        reason: Option<String>,
    ) -> Result<(), SignError> {
        let i = &self.inner;
        let body = {
            let mut st = i.state.lock().await;
            let prop = st
                .props
                .get_mut(&topic)
                .ok_or_else(|| err("not_found", "unknown proposal"))?;
            if !matches!(prop.rec.pending, Some(PendingAnswer::Reject { .. })) {
                let binding = derive_binding_key(&prop.sym);
                let bpub = binding.verifying_key().to_bytes();
                let payload = RejectionPayload {
                    connection_id: topic.clone(),
                    reason: Some(
                        reason.unwrap_or_else(|| "user_rejected".into()),
                    ),
                };
                prop.rec.pending = Some(PendingAnswer::Reject {
                    body: json!({ "rejection": build_signed(&binding, labels::REJECT, &bpub, &payload) }),
                });
                i.store.put(&format!("prop/{topic}"), &prop.rec).await?;
            }
            let Some(PendingAnswer::Reject { body }) = prop.rec.pending.clone()
            else {
                unreachable!()
            };
            body
        };
        i.answer(&format!("/v3/connections/{topic}/rejection"), &body).await?;
        i.state.lock().await.props.remove(&topic);
        i.drop_prop(&topic).await?;
        Ok(())
    }

    // ------------------------------------------------------------ session

    pub async fn request(
        &self,
        topic: String,
        chain_id: String,
        request: RpcRequest,
        deadline_s: Option<u64>,
    ) -> Result<String, SignError> {
        let request = serde_json::to_value(&request).expect("request");
        let rpc = json!({ "id": new_rpc_id(), "jsonrpc": "2.0", "method": "wc_sessionRequest", "params": { "request": request, "chainId": chain_id } });
        self.inner
            .send_request(
                &topic,
                tags::REQUEST,
                rpc,
                deadline_s.unwrap_or(DEFAULT_DEADLINE_S),
            )
            .await
            .map(|v| v.to_string())
    }

    pub async fn respond(
        &self,
        topic: String,
        id: i64,
        result: Option<String>,
        error: Option<RpcError>,
    ) -> Result<(), SignError> {
        let i = &self.inner;
        let mut rpc = json!({ "id": id, "jsonrpc": "2.0" });
        match (result, error) {
            (_, Some(e)) => {
                rpc["error"] = serde_json::to_value(&e).expect("error")
            }
            (Some(r), None) => rpc["result"] = json_arg(&r)?,
            (None, None) => rpc["result"] = Value::Null,
        }
        let key = format!("inreq/{topic}/{id}");
        let req: InReq = i.store.get(&key).await?.ok_or_else(|| {
            err("not_found", "no pending request with that id")
        })?;
        let tag = method_tag(&req.method)
            .ok_or_else(|| err("not_found", "unknown request method"))?
            + 1;
        i.send(&topic, tag, rpc, DEFAULT_DEADLINE_S).await?;
        i.store.remove(&key).await?;
        Ok(())
    }

    pub async fn emit(
        &self,
        topic: String,
        chain_id: String,
        event: SessionEventData,
    ) -> Result<(), SignError> {
        let event = serde_json::to_value(&event).expect("event");
        let rpc = json!({ "id": new_rpc_id(), "jsonrpc": "2.0", "method": "wc_sessionEvent", "params": { "chainId": chain_id, "event": event } });
        self.inner
            .send_request(&topic, tags::EVENT, rpc, DEFAULT_DEADLINE_S)
            .await
            .map(|_| ())
    }

    pub async fn ping(&self, topic: String) -> Result<(), SignError> {
        let rpc = json!({ "id": new_rpc_id(), "jsonrpc": "2.0", "method": "wc_sessionPing", "params": {} });
        self.inner
            .send_request(&topic, tags::PING, rpc, DEFAULT_DEADLINE_S)
            .await
            .map(|_| ())
    }

    pub async fn update(
        &self,
        topic: String,
        namespaces: Namespaces,
    ) -> Result<(), SignError> {
        let namespaces = serde_json::to_value(&namespaces).expect("namespaces");
        self.inner
            .require_role(
                &topic,
                Role::Wallet,
                "only the wallet updates a v3 session",
            )
            .await?;
        let rpc = json!({ "id": new_rpc_id(), "jsonrpc": "2.0", "method": "wc_sessionUpdate", "params": { "namespaces": namespaces } });
        self.inner
            .send_request(&topic, tags::UPDATE, rpc, DEFAULT_DEADLINE_S)
            .await?;
        self.inner.patch_conn(&topic, |c| c.namespaces = namespaces).await
    }

    pub async fn extend(
        &self,
        topic: String,
        expiry: Option<u64>,
    ) -> Result<(), SignError> {
        self.inner
            .require_role(
                &topic,
                Role::Wallet,
                "only the wallet extends a v3 session",
            )
            .await?;
        let expiry =
            expiry.unwrap_or_else(|| self.inner.now_s() + MAX_LIFETIME_S);
        let rpc = json!({ "id": new_rpc_id(), "jsonrpc": "2.0", "method": "wc_sessionExtend", "params": { "expiry": expiry } });
        self.inner
            .send_request(&topic, tags::EXTEND, rpc, DEFAULT_DEADLINE_S)
            .await?;
        self.inner.patch_conn(&topic, |c| c.expiry = expiry).await
    }

    pub async fn disconnect(&self, topic: String) -> Result<(), SignError> {
        self.inner.ensure_active().await?;
        let i = &self.inner;
        let (key, version, bpub) = {
            let st = i.state.lock().await;
            let c = st
                .conns
                .get(&topic)
                .ok_or_else(|| err("not_found", "unknown session"))?;
            (c.key.clone(), c.rec.version, c.rec.binding())
        };
        let res = i.close_remote(&topic, &key, version, &bpub).await;
        i.drop_conn(&topic, "closed", false, false).await?;
        res
    }
}

/// `{ method, params }` inside a `wc_sessionRequest`.
fn rpc_request_of(v: &Value) -> RpcRequest {
    RpcRequest {
        method: v["method"].as_str().unwrap_or_default().into(),
        params: v["params"].to_string(),
    }
}

fn rpc_response_of(rpc: &Value) -> RpcResponse {
    RpcResponse {
        id: rpc["id"].as_i64().unwrap_or_default(),
        result: rpc.get("result").map(Value::to_string),
        error: rpc.get("error").and_then(ijson::from_value),
    }
}

/// The stored request a late response answers: the RPC call for a session request, the
/// session method itself otherwise.
fn stored_request_of(o: &OutReq) -> RpcRequest {
    if o.method == "wc_sessionRequest" {
        rpc_request_of(&o.params["request"])
    } else {
        RpcRequest { method: o.method.clone(), params: o.params.to_string() }
    }
}

pub(crate) fn session_of(c: &ConnRecord) -> Session {
    Session {
        topic: c.id.clone(),
        role: c.role,
        version: c.version,
        namespaces: ijson::from_value(&c.namespaces).unwrap_or_default(),
        peer_metadata: c.peer_metadata.clone(),
        dapp_mode: c.dapp_mode,
        wallet_mode: c.wallet_mode,
        expiry: c.expiry,
    }
}

/// PAIR-13 and EVT-04 close "under the version in the binding as received", even when it fails.
fn unverified_version(o: &SignedObject) -> u32 {
    from_b64u(&o.payload)
        .ok()
        .and_then(|b| ijson::value(&b))
        .and_then(|v| v["version"].as_u64())
        .unwrap_or(0) as u32
}

impl Inner {
    pub fn emit(&self, e: SignEvent) {
        self.listener.on_event(e);
    }

    pub fn cfg_first_line_timeout(&self) -> u64 {
        self.cfg
            .first_line_timeout_ms
            .unwrap_or(crate::protocol::constants::FIRST_LINE_TIMEOUT_MS)
    }

    /// MSG-11: when the device clock is off by more than the tolerance, use the Relay's time.
    pub fn now_s(&self) -> u64 {
        ((rt::now_ms() / 1000) as i64
            + self.clock_offset_s.load(Ordering::SeqCst))
        .max(0) as u64
    }

    pub fn on_relay_time(&self, t_ms: u64) {
        let skew = (t_ms as i64 - rt::now_ms() as i64) / 1000;
        let next =
            if skew.unsigned_abs() > CLOCK_SKEW_TOLERANCE_S { skew } else { 0 };
        if self.clock_offset_s.swap(next, Ordering::SeqCst) != next && next != 0
        {
            self.emit(SignEvent::ClockSkew { skew_s: skew });
        }
    }

    async fn restore(&self) -> Result<(), SignError> {
        let mut st = self.state.lock().await;
        st.conns.clear();
        st.reqs.clear();
        st.props.clear();
        for k in self.store.keys("conn/").await? {
            let Some(rec) = self.store.get::<ConnRecord>(&k).await? else {
                continue;
            };
            let Some(sec) =
                self.store.get::<ConnSecrets>(&format!("secret/{k}")).await?
            else {
                continue;
            };
            let (Some(key), Some(d2w), Some(w2d)) =
                (key_from_b64(&sec.pairing), arr32(&sec.d2w), arr32(&sec.w2d))
            else {
                continue;
            };
            st.conns.insert(
                rec.id.clone(),
                Conn {
                    rec,
                    key,
                    d2w: Zeroizing::new(d2w),
                    w2d: Zeroizing::new(w2d),
                },
            );
        }
        for k in self.store.keys("req/").await? {
            let Some(rec) = self.store.get::<DappReq>(&k).await? else {
                continue;
            };
            let Some(sec) =
                self.store.get::<ReqSecrets>(&format!("secret/{k}")).await?
            else {
                continue;
            };
            let (Some(sym), Some(key)) =
                (arr32(&sec.sym), key_from_b64(&sec.pairing))
            else {
                continue;
            };
            st.reqs.insert(
                rec.id.clone(),
                Req { rec, sym: Zeroizing::new(sym), key },
            );
        }
        for k in self.store.keys("prop/").await? {
            let Some(rec) = self.store.get::<WalletProp>(&k).await? else {
                continue;
            };
            let Some(sec) =
                self.store.get::<PropSecrets>(&format!("secret/{k}")).await?
            else {
                continue;
            };
            let Some(sym) = arr32(&sec.sym) else { continue };
            let key = sec.pairing.as_deref().and_then(key_from_b64);
            st.props.insert(
                rec.id.clone(),
                Prop { rec, sym: Zeroizing::new(sym), key },
            );
        }
        st.cursor = self.store.get("poll/cursor").await?;
        st.n = self.store.get("poll/n").await?.unwrap_or(0);
        Ok(())
    }

    /// Web: a superseded tab reactivates on its next call. It reloads state first, because the
    /// active tab has moved seq counters, the cursor and sessions forward (guide §10.1).
    async fn ensure_active(self: &Arc<Self>) -> Result<(), SignError> {
        if cfg!(target_arch = "wasm32")
            && self.superseded.load(Ordering::SeqCst)
        {
            self.restore().await?;
            self.superseded.store(false, Ordering::SeqCst);
            poller::start(self);
        }
        Ok(())
    }

    fn save_conn<'a>(
        &'a self,
        c: &'a Conn,
        with_secrets: bool,
    ) -> BoxFut<'a, Result<(), SignError>> {
        rt::boxed(self.save_conn_inner(c, with_secrets))
    }

    async fn save_conn_inner(
        &self,
        c: &Conn,
        with_secrets: bool,
    ) -> Result<(), SignError> {
        if with_secrets {
            let sec = ConnSecrets {
                pairing: key_to_b64(&c.key),
                d2w: b64u(c.d2w.as_ref()),
                w2d: b64u(c.w2d.as_ref()),
            };
            self.store.put(&format!("secret/conn/{}", c.rec.id), &sec).await?;
        }
        self.store.put(&format!("conn/{}", c.rec.id), &c.rec).await?;
        Ok(())
    }

    async fn patch_conn(
        &self,
        id: &str,
        f: impl FnOnce(&mut ConnRecord),
    ) -> Result<(), SignError> {
        let mut st = self.state.lock().await;
        let c = st
            .conns
            .get_mut(id)
            .ok_or_else(|| err("not_found", "unknown session"))?;
        f(&mut c.rec);
        self.save_conn(c, false).await
    }

    async fn require_role(
        self: &Arc<Self>,
        id: &str,
        role: Role,
        why: &str,
    ) -> Result<(), SignError> {
        self.ensure_active().await?;
        let st = self.state.lock().await;
        let c = st
            .conns
            .get(id)
            .ok_or_else(|| err("not_found", "unknown session"))?;
        if c.rec.role != role {
            return Err(err("wallet_only", why));
        }
        Ok(())
    }

    /// An answer after the window gets 410 and one to a resolved connection 409: either way
    /// it's too late.
    async fn answer(&self, path: &str, body: &Value) -> Result<(), SignError> {
        match self
            .relay
            .call(
                "PUT",
                path,
                Some(serde_json::to_vec(body).expect("json")),
                vec![],
            )
            .await
            .map_err(SignError::from)
        {
            Err(e) if e.code() == "connection_closed" => {
                Err(err("request_resolved", "answer window closed"))
            }
            other => other.map(|_| ()),
        }
    }

    async fn drop_prop(&self, topic: &str) -> Result<(), SignError> {
        self.store.remove(&format!("prop/{topic}")).await?;
        self.store.remove(&format!("secret/prop/{topic}")).await?;
        Ok(())
    }

    async fn drop_req(
        &self,
        st: &mut State,
        topic: &str,
    ) -> Result<(), SignError> {
        st.reqs.remove(topic);
        self.store.remove(&format!("req/{topic}")).await?;
        self.store.remove(&format!("secret/req/{topic}")).await?;
        Ok(())
    }

    fn drop_conn<'a>(
        &'a self,
        id: &'a str,
        reason: &'a str,
        notify: bool,
        verified: bool,
    ) -> BoxFut<'a, Result<(), SignError>> {
        rt::boxed(self.drop_conn_inner(id, reason, notify, verified))
    }

    async fn drop_conn_inner(
        &self,
        id: &str,
        reason: &str,
        notify: bool,
        verified: bool,
    ) -> Result<(), SignError> {
        let removed = self.state.lock().await.conns.remove(id).is_some();
        if !removed {
            return Ok(());
        }
        self.store.remove(&format!("conn/{id}")).await?;
        self.store.remove(&format!("secret/conn/{id}")).await?;
        for k in self.store.keys(&format!("out/{id}/")).await? {
            self.store.remove(&k).await?;
        }
        for k in self.store.keys(&format!("inreq/{id}/")).await? {
            self.store.remove(&k).await?;
        }
        for k in self.store.keys(&format!("outreq/{id}/")).await? {
            self.store.remove(&k).await?;
        }
        let stale: Vec<Pending> = {
            let mut p = self.pending.lock().expect("lock");
            let keys: Vec<String> = p
                .iter()
                .filter(|(_, v)| v.conn == id)
                .map(|(k, _)| k.clone())
                .collect();
            keys.into_iter().filter_map(|k| p.remove(&k)).collect()
        };
        for p in stale {
            let _ = p
                .tx
                .send(Err(err("connection_closed", "the session was closed")));
        }
        if notify {
            self.emit(SignEvent::SessionDelete {
                topic: id.into(),
                reason: reason.into(),
                verified,
            });
        }
        Ok(())
    }

    fn close_remote<'a>(
        &'a self,
        id: &'a str,
        key: &'a SigningKey,
        version: u32,
        bpub: &'a [u8; 32],
    ) -> BoxFut<'a, Result<(), SignError>> {
        rt::boxed(self.close_remote_inner(id, key, version, bpub))
    }

    async fn close_remote_inner(
        &self,
        id: &str,
        key: &SigningKey,
        version: u32,
        bpub: &[u8; 32],
    ) -> Result<(), SignError> {
        let sig = sign_bytes(key, &close_label(version), bpub, &[]);
        self.relay
            .call(
                "DELETE",
                &format!("/v3/connections/{id}"),
                None,
                vec![("wc-signature".into(), sig)],
            )
            .await?;
        Ok(())
    }

    // ------------------------------------------------------------ sending

    async fn send_request(
        self: &Arc<Self>,
        conn: &str,
        tag: u32,
        rpc: Value,
        deadline_s: u64,
    ) -> Result<Value, SignError> {
        self.ensure_active().await?;
        let id = rpc["id"].as_i64().expect("rpc id");
        let key = pkey(conn, id);
        let out_key = format!("outreq/{conn}/{id}");
        let (tx, rx) = oneshot::channel();
        // Register before sending so a fast response is never missed.
        self.pending
            .lock()
            .expect("lock")
            .insert(key.clone(), Pending { tx, conn: conn.into(), seq: 0 });
        let seq = match self.send(conn, tag, rpc, deadline_s).await {
            Ok(seq) => seq,
            Err(e) => {
                self.pending.lock().expect("lock").remove(&key);
                let _ = self.store.remove(&out_key).await;
                return Err(e);
            }
        };
        if let Some(p) = self.pending.lock().expect("lock").get_mut(&key) {
            p.seq = seq;
        }
        // Local deadline, as v2's request expiry: the response may have reached another tab.
        let local_deadline =
            rt::sleep_ms((deadline_s + CLOCK_SKEW_TOLERANCE_S.min(30)) * 1000)
                .fuse();
        futures::pin_mut!(local_deadline);
        let mut rx = rx.fuse();
        futures::select! {
            r = rx => r.unwrap_or_else(|_| Err(err("connection_closed", "client stopped"))),
            _ = local_deadline => {
                self.pending.lock().expect("lock").remove(&key);
                let _ = self.store.remove(&out_key).await;
                // EVT-03: it may still have been seen, or the Relay withheld it.
                self.emit(SignEvent::MessageExpired { topic: conn.into(), seq, id: Some(id) });
                Err(err("message_expired", "no response by the deadline"))
            }
        }
    }

    /// Guide §5.3. Returns the seq used.
    pub(crate) async fn send(
        &self,
        conn: &str,
        tag: u32,
        rpc: Value,
        deadline_s: u64,
    ) -> Result<u64, SignError> {
        let deadline_s = deadline_s.min(MAX_DEADLINE_S);
        match self.send_once(conn, tag, &rpc, deadline_s).await {
            // Another tab advanced this connection's seq: reload it from storage and re-sign once.
            Err(e) if e.code() == "seq_reused" => {
                self.reload_conn(conn).await?;
                self.send_once(conn, tag, &rpc, deadline_s).await
            }
            other => other,
        }
    }

    async fn reload_conn(&self, id: &str) -> Result<(), SignError> {
        if let Some(rec) =
            self.store.get::<ConnRecord>(&format!("conn/{id}")).await?
            && let Some(c) = self.state.lock().await.conns.get_mut(id)
        {
            c.rec = rec;
        }
        Ok(())
    }

    fn send_once<'a>(
        &'a self,
        conn: &'a str,
        tag: u32,
        rpc: &'a Value,
        deadline_s: u64,
    ) -> BoxFut<'a, Result<u64, SignError>> {
        rt::boxed(self.send_once_inner(conn, tag, rpc, deadline_s))
    }

    async fn send_once_inner(
        &self,
        conn: &str,
        tag: u32,
        rpc: &Value,
        deadline_s: u64,
    ) -> Result<u64, SignError> {
        let out = {
            let mut st = self.state.lock().await;
            let c = st
                .conns
                .get_mut(conn)
                .ok_or_else(|| err("not_found", "unknown session"))?;
            let seq = c.rec.next_seq;
            // MSG-04: never reuse a seq; persist the counter before signing.
            c.rec.next_seq += 1;
            self.save_conn(c, false).await?;
            let seal = is_sealable(tag) && c.rec.own_mode() == Mode::Sealed;
            let bpub = c.rec.binding();
            let now = self.now_s();
            // The Relay keeps a message for its ttl. An update or extension outlives its
            // deadline (MSG-13), so it is kept until the session it changes would end.
            let until = match tag {
                tags::EXTEND => rpc["params"]["expiry"].as_u64().unwrap_or(0),
                tags::UPDATE => c.rec.expiry,
                _ => 0,
            };
            let ttl = deadline_s
                .max(until.saturating_sub(now))
                .clamp(MIN_TTL_S, MAX_TTL_S);
            let message = build_message(BuildMessage {
                key: &c.key,
                version: c.rec.version,
                binding_pub: &bpub,
                tag,
                seq,
                deadline: Some(now + deadline_s),
                rpc,
                seal_key: seal.then(|| c.send_key()),
                nonce: None,
            });
            // An outgoing request is stored before it can be delivered, so its response or
            // expiry always finds it, even after a restart.
            if is_request(rpc) {
                let id = rpc["id"].as_i64().unwrap_or_default();
                let tracked = OutReq {
                    conn: conn.into(),
                    id,
                    method: rpc["method"].as_str().unwrap_or_default().into(),
                    seq,
                    deadline: now + deadline_s,
                    params: rpc["params"].clone(),
                };
                self.store
                    .put(&format!("outreq/{conn}/{id}"), &tracked)
                    .await?;
            }
            let out = Outbox { conn: conn.into(), seq, message, ttl };
            self.store.put(&format!("out/{conn}/{seq}"), &out).await?;
            out
        };
        self.post(&out).await?;
        Ok(out.seq)
    }

    pub(crate) fn post<'a>(
        &'a self,
        out: &'a Outbox,
    ) -> BoxFut<'a, Result<(), SignError>> {
        rt::boxed(self.post_inner(out))
    }

    async fn post_inner(&self, out: &Outbox) -> Result<(), SignError> {
        let body = serde_json::to_vec(
            &json!({ "message": out.message, "ttl": out.ttl }),
        )
        .expect("json");
        match self
            .relay
            .call(
                "POST",
                &format!("/v3/connections/{}/messages", out.conn),
                Some(body),
                vec![],
            )
            .await
        {
            Ok(_) => {
                self.store
                    .remove(&format!("out/{}/{}", out.conn, out.seq))
                    .await?;
                Ok(())
            }
            Err(e) => {
                let final_error = matches!(&e, crate::transport::relay::RelayError::Http { status, .. } if *status < 500 && *status != 429);
                let se = SignError::from(e);
                if final_error {
                    // The seq stays used and leaves a gap (MSG-06).
                    self.store
                        .remove(&format!("out/{}/{}", out.conn, out.seq))
                        .await?;
                }
                if se.code() == "connection_closed" {
                    self.drop_conn(&out.conn, "closed", true, false).await?;
                }
                Err(se)
            }
        }
    }

    /// Checklist: signed messages not yet answered with 202 are resent with identical bytes.
    pub(crate) async fn flush_outbox(&self) {
        let Ok(keys) = self.store.keys("out/").await else { return };
        for k in keys {
            if let Ok(Some(out)) = self.store.get::<Outbox>(&k).await {
                let _ = self.post(&out).await;
            }
        }
    }

    // ------------------------------------------------------------ receiving

    pub(crate) async fn handle_events(
        self: &Arc<Self>,
        events: Vec<Value>,
    ) -> Result<(), SignError> {
        for raw in events {
            let Some(e) =
                crate::protocol::ijson::from_value::<RelayEvent>(&raw)
            else {
                continue;
            };
            match e {
                RelayEvent::ConnectionEstablished {
                    connection_id,
                    binding,
                } => {
                    rt::boxed(self.on_established(&connection_id, &binding))
                        .await?
                }
                RelayEvent::ConnectionRejected { connection_id, rejection } => {
                    rt::boxed(self.on_rejected(&connection_id, &rejection))
                        .await?
                }
                RelayEvent::MessageReceived { connection_id, message } => {
                    rt::boxed(self.on_message(&connection_id, &message)).await?
                }
                RelayEvent::ConnectionClosed {
                    connection_id,
                    reason,
                    close,
                } => {
                    rt::boxed(self.on_closed(
                        &connection_id,
                        &reason,
                        close.as_deref(),
                    ))
                    .await?
                }
                RelayEvent::Unknown => {}
            }
        }
        Ok(())
    }

    async fn on_established(
        self: &Arc<Self>,
        id: &str,
        signed: &SignedObject,
    ) -> Result<(), SignError> {
        let mut st = self.state.lock().await;
        let Some(req) = st.reqs.get(id) else { return Ok(()) };
        let binding = derive_binding_key(&req.sym);
        let bpub = binding.verifying_key().to_bytes();
        let key = req.key.clone();
        // EVT-04: established for a request counted as failed is answered with a signed close.
        if req.rec.failed {
            drop(st);
            let _ = self
                .close_remote(id, &key, unverified_version(signed), &bpub)
                .await;
            return Ok(());
        }
        let checked = (|| -> Result<(ConnRecord, [u8; 32]), SignError> {
            let p = open_signed(&bpub, labels::BIND, &bpub, signed)?;
            let bad = |m: &str| err("invalid_peer", m.to_string());
            let version = p.int("/version").map_err(|e| bad(&e.0))? as u32;
            let expiry = p.int("/expiry").map_err(|e| bad(&e.0))? as u64;
            if p.value["connectionId"] != id {
                return Err(bad("binding is for another request"));
            }
            if !req.rec.versions.contains(&version) {
                return Err(bad("version not offered"));
            }
            let intent: WalletIntent = crate::protocol::ijson::from_value::<
                WalletIntent,
            >(&p.value["intent"])
            .ok_or_else(|| bad("intent"))?;
            if !satisfies(intent.own, req.rec.intent.peer) {
                return Err(bad("wallet intent does not meet the requirement"));
            }
            let wallet_pub_s = p.value["pairingPub"]
                .as_str()
                .ok_or_else(|| bad("pairingPub"))?;
            let wallet_pub: [u8; 32] = from_b64u(wallet_pub_s)?
                .try_into()
                .map_err(|_| bad("pairingPub"))?;
            if !is_valid_pairing_pub(&wallet_pub) {
                return Err(bad("invalid wallet pairingPub"));
            }
            let wire: WireMessage = crate::protocol::ijson::from_value::<
                WireMessage,
            >(&p.value["approval"])
            .ok_or_else(|| bad("approval"))?;
            let m = read_message(&wire)?;
            if m.plain.tag != tags::APPROVAL
                || m.plain.seq != 0
                || m.plain.sealed
                || !verify_message(&wallet_pub, 0, &bpub, &m)
            {
                return Err(bad("approval signature"));
            }
            let rpc = open_message_payload(&m, 0, &bpub, None)?;
            if is_request(&rpc.value)
                || rpc.value["id"].as_i64() != Some(req.rec.proposal_id)
                || !rpc.value["error"].is_null()
            {
                return Err(bad("malformed approval"));
            }
            let result = &rpc.value["result"];
            let metadata: Metadata = crate::protocol::ijson::from_value::<
                Metadata,
            >(&result["metadata"])
            .ok_or_else(|| bad("metadata"))?;
            Ok((
                ConnRecord {
                    id: id.into(),
                    role: Role::Dapp,
                    version,
                    binding_pub: b64u(&bpub),
                    dapp_pub: b64u(&req.key.verifying_key().to_bytes()),
                    wallet_pub: wallet_pub_s.into(),
                    dapp_mode: req.rec.intent.own,
                    wallet_mode: intent.own,
                    namespaces: result["namespaces"].clone(),
                    peer_metadata: metadata,
                    expiry,
                    next_seq: 1,
                    recv: Default::default(),
                    applied_update: 0,
                    applied_extend: 0,
                },
                wallet_pub,
            ))
        })();
        match checked {
            Ok((rec, wallet_pub)) => {
                let dapp_pub = key.verifying_key().to_bytes();
                let keys = derive_sealing_keys(
                    rec.version,
                    &key,
                    &wallet_pub,
                    &req.sym,
                    &bpub,
                    &dapp_pub,
                    &wallet_pub,
                )?;
                let conn = Conn { rec, key, d2w: keys.d2w, w2d: keys.w2d };
                self.save_conn(&conn, true).await?;
                let session = session_of(&conn.rec);
                st.conns.insert(id.into(), conn);
                self.drop_req(&mut st, id).await?;
                drop(st);
                self.emit(SignEvent::SessionConnect { session });
            }
            Err(e) => {
                // PAIR-13: never use it; close it under the version in the binding as received.
                self.stats.invalid_dropped.fetch_add(1, Ordering::Relaxed);
                if let Some(r) = st.reqs.get_mut(id) {
                    r.rec.failed = true;
                    self.store.put(&format!("req/{id}"), &r.rec).await?;
                }
                drop(st);
                self.emit(SignEvent::ProposalFailed {
                    topic: id.into(),
                    reason: e.to_string(),
                });
                let _ = self
                    .close_remote(id, &key, unverified_version(signed), &bpub)
                    .await;
            }
        }
        Ok(())
    }

    async fn on_rejected(
        &self,
        id: &str,
        rejection: &SignedObject,
    ) -> Result<(), SignError> {
        let reason = {
            let st = self.state.lock().await;
            let Some(req) = st.reqs.get(id) else { return Ok(()) };
            if req.rec.failed {
                return Ok(());
            }
            let bpub = derive_binding_key(&req.sym).verifying_key().to_bytes();
            match open_signed(&bpub, labels::REJECT, &bpub, rejection) {
                Ok(p) if p.value["connectionId"] == id => {
                    p.value["reason"].as_str().map(str::to_string)
                }
                _ => return Ok(()),
            }
        };
        if self.mark_failed(id).await? {
            self.emit(SignEvent::ProposalRejected { topic: id.into(), reason });
        }
        Ok(())
    }

    /// The request ends without a session: `expired`, `replaced` or `cursor_expired`.
    async fn fail_request(
        &self,
        id: &str,
        reason: &str,
    ) -> Result<(), SignError> {
        if self.mark_failed(id).await? {
            self.emit(SignEvent::ProposalExpire {
                topic: id.into(),
                reason: reason.into(),
            });
        }
        Ok(())
    }

    /// Keeps the request, marked failed, so EVT-04 can close a late binding. False when it was
    /// already failed or is unknown.
    async fn mark_failed(&self, id: &str) -> Result<bool, SignError> {
        let mut st = self.state.lock().await;
        let Some(req) = st.reqs.get_mut(id) else { return Ok(false) };
        if req.rec.failed {
            return Ok(false);
        }
        req.rec.failed = true;
        self.store.put(&format!("req/{id}"), &req.rec).await?;
        Ok(true)
    }

    /// Guide §5.4.
    async fn on_message(
        self: &Arc<Self>,
        id: &str,
        wire: &WireMessage,
    ) -> Result<(), SignError> {
        let mut st = self.state.lock().await;
        let Some(c) = st.conns.get_mut(id) else { return Ok(()) };
        let invalid = |reason: String| {
            self.stats.invalid_dropped.fetch_add(1, Ordering::Relaxed);
            SignEvent::MessageInvalid { topic: id.into(), reason }
        };
        let m = match read_message(wire) {
            Ok(m) => m,
            Err(e) => {
                drop(st);
                self.emit(invalid(e.to_string()));
                return Ok(());
            }
        };
        let tag = m.plain.tag;
        let peer = c.rec.role.peer();
        let sealable = is_sealable(tag);
        let checks = if tag_info(tag).is_none()
            || tag == tags::PROPOSAL
            || tag == tags::APPROVAL
            || !may_role_send(peer, tag)
        {
            Err("tag not allowed for sender".to_string())
        } else if m.plain.sealed
            != (sealable && c.rec.peer_mode() == Mode::Sealed)
        {
            Err("sealed flag contradicts tag or mode".to_string())
        } else if !verify_message(
            &c.rec.peer_pub(),
            c.rec.version,
            &c.rec.binding(),
            &m,
        ) {
            Err("invalid signature".to_string())
        } else {
            open_message_payload(
                &m,
                c.rec.version,
                &c.rec.binding(),
                m.plain.sealed.then(|| c.recv_key()),
            )
            .map_err(|e| e.to_string())
        };
        let rpc = match checks {
            Ok(p) => p.value,
            Err(reason) => {
                drop(st);
                self.emit(invalid(reason));
                return Ok(());
            }
        };
        let is_state_change = tag == tags::UPDATE || tag == tags::EXTEND;
        if !is_state_change
            && m.plain.deadline.unwrap_or(0) + CLOCK_SKEW_TOLERANCE_S
                < self.now_s()
        {
            self.stats.late_dropped.fetch_add(1, Ordering::Relaxed);
            c.rec.recv.accept(m.plain.seq);
            return self.save_conn(c, false).await;
        }
        if !c.rec.recv.accept(m.plain.seq) {
            self.stats.duplicates_dropped.fetch_add(1, Ordering::Relaxed);
            return Ok(());
        }
        // Persist the replay window before the app sees the message.
        self.save_conn(c, false).await?;
        let rid = rpc["id"].as_i64().unwrap_or(0);
        if !is_request(&rpc) {
            drop(st);
            let hit = self.pending.lock().expect("lock").remove(&pkey(id, rid));
            let out_key = format!("outreq/{id}/{rid}");
            let stored: Option<OutReq> = self.store.get(&out_key).await?;
            if stored.is_some() {
                self.store.remove(&out_key).await?;
            }
            match hit {
                Some(p) => {
                    let res = if rpc["error"].is_null() {
                        Ok(rpc["result"].clone())
                    } else {
                        // The peer's `{ code, message }` as JSON, so callers keep its code.
                        Err(err("rpc_error", rpc["error"].to_string()))
                    };
                    let _ = p.tx.send(res);
                }
                None => self.emit(SignEvent::SessionResponse {
                    topic: id.into(),
                    response: rpc_response_of(&rpc),
                    request: stored.map(|o| stored_request_of(&o)),
                }),
            }
            return Ok(());
        }
        let method = rpc["method"].as_str().unwrap_or_default().to_string();
        let params = rpc["params"].clone();
        let mut ack = true;
        let event = match method.as_str() {
            "wc_sessionRequest" => {
                ack = false;
                let inreq = InReq {
                    conn: id.into(),
                    id: rid,
                    method: method.clone(),
                    deadline: m.plain.deadline.unwrap_or(0),
                    params: params.clone(),
                };
                self.store.put(&format!("inreq/{id}/{rid}"), &inreq).await?;
                Some(SignEvent::SessionRequest {
                    id: rid,
                    topic: id.into(),
                    chain_id: params["chainId"]
                        .as_str()
                        .unwrap_or_default()
                        .into(),
                    request: rpc_request_of(&params["request"]),
                })
            }
            "wc_sessionEvent" => Some(SignEvent::SessionEvent {
                id: rid,
                topic: id.into(),
                chain_id: params["chainId"].as_str().unwrap_or_default().into(),
                event: SessionEventData {
                    name: params["event"]["name"]
                        .as_str()
                        .unwrap_or_default()
                        .into(),
                    data: params["event"]["data"].to_string(),
                },
            }),
            "wc_sessionPing" => {
                Some(SignEvent::SessionPing { id: rid, topic: id.into() })
            }
            // MSG-13: apply whatever the deadline, only above the last applied seq of its kind.
            "wc_sessionUpdate" if m.plain.seq > c.rec.applied_update => {
                c.rec.applied_update = m.plain.seq;
                c.rec.namespaces = params["namespaces"].clone();
                self.save_conn(c, false).await?;
                Some(SignEvent::SessionUpdate {
                    id: rid,
                    topic: id.into(),
                    namespaces: ijson::from_value(&params["namespaces"])
                        .unwrap_or_default(),
                })
            }
            "wc_sessionExtend" if m.plain.seq > c.rec.applied_extend => {
                let expiry = params["expiry"].as_u64().unwrap_or(c.rec.expiry);
                c.rec.applied_extend = m.plain.seq;
                c.rec.expiry = expiry;
                self.save_conn(c, false).await?;
                Some(SignEvent::SessionExtend {
                    id: rid,
                    topic: id.into(),
                    expiry,
                })
            }
            "wc_sessionUpdate" | "wc_sessionExtend" => None,
            _ => None,
        };
        drop(st);
        let known = method_tag(&method).is_some();
        if let Some(e) = event {
            self.emit(e);
        }
        if ack || !known {
            let me = self.clone();
            let id = id.to_string();
            let reply = if known {
                json!({ "id": rid, "jsonrpc": "2.0", "result": true })
            } else {
                json!({ "id": rid, "jsonrpc": "2.0", "error": { "code": -32601, "message": "method not found" } })
            };
            rt::spawn(async move {
                let _ = me.send(&id, tag + 1, reply, DEFAULT_DEADLINE_S).await;
            });
        }
        Ok(())
    }

    async fn on_closed(
        &self,
        id: &str,
        reason: &str,
        close: Option<&str>,
    ) -> Result<(), SignError> {
        let verified = {
            let st = self.state.lock().await;
            let Some(c) = st.conns.get(id) else { return Ok(()) };
            // EVT-02: a close is the peer's act only when its signature verifies.
            reason == "closed"
                && close.is_some_and(|sig| {
                    verify_bytes(
                        &c.rec.peer_pub(),
                        sig,
                        &close_label(c.rec.version),
                        &c.rec.binding(),
                        &[],
                    )
                })
        };
        self.drop_conn(id, reason, true, verified).await
    }

    /// RCV-05: close every connection with a signed close, fail pending requests, poll without a cursor.
    pub(crate) async fn on_cursor_expired(&self) -> Result<(), SignError> {
        let conns: Vec<(String, SigningKey, u32, [u8; 32])> = self
            .state
            .lock()
            .await
            .conns
            .values()
            .map(|c| {
                (
                    c.rec.id.clone(),
                    c.key.clone(),
                    c.rec.version,
                    c.rec.binding(),
                )
            })
            .collect();
        for (id, key, version, bpub) in conns {
            let _ = self.close_remote(&id, &key, version, &bpub).await;
            self.drop_conn(&id, "cursor_expired", true, false).await?;
        }
        let topics: Vec<String> =
            self.state.lock().await.reqs.keys().cloned().collect();
        for t in topics {
            self.fail_request(&t, "cursor_expired").await?;
        }
        self.emit(SignEvent::CursorExpired);
        Ok(())
    }

    /// EVT-06: decide expiry only after a poll that caught up with the log.
    pub(crate) async fn sweep_expired(&self) -> Result<(), SignError> {
        let now = self.now_s();
        let expired: Vec<String> = self
            .state
            .lock()
            .await
            .conns
            .values()
            .filter(|c| c.rec.expiry <= now)
            .map(|c| c.rec.id.clone())
            .collect();
        for id in expired {
            self.drop_conn(&id, "expired", true, false).await?;
        }
        // The Relay drops an unanswered request at its window, silently.
        let unanswered: Vec<String> = self
            .state
            .lock()
            .await
            .reqs
            .values()
            .filter(|r| !r.rec.failed && r.rec.expires_at + LOCAL_GRACE_S < now)
            .map(|r| r.rec.id.clone())
            .collect();
        for t in unanswered {
            self.fail_request(&t, "expired").await?;
        }
        // An outgoing request restored after a restart, with no live call to time it out.
        for k in self.store.keys("outreq/").await? {
            let Some(o) = self.store.get::<OutReq>(&k).await? else { continue };
            let live = self
                .pending
                .lock()
                .expect("lock")
                .contains_key(&pkey(&o.conn, o.id));
            if !live && o.deadline + LOCAL_GRACE_S < now {
                self.store.remove(&k).await?;
                self.emit(SignEvent::MessageExpired {
                    topic: o.conn,
                    seq: o.seq,
                    id: Some(o.id),
                });
            }
        }
        let mut st = self.state.lock().await;
        let old: Vec<String> = st
            .reqs
            .values()
            .filter(|r| {
                r.rec.failed && r.rec.expires_at + ANSWER_WINDOW_S < now
            })
            .map(|r| r.rec.id.clone())
            .collect();
        for t in old {
            self.drop_req(&mut st, &t).await?;
        }
        Ok(())
    }
}
