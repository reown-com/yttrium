//! Verify v4 App Signatures through the client, against an in-memory Relay:
//! the dApp signs exactly the bytes it sends, and the wallet checks them.

use {
    crate::{
        AppSignatureError, AppSignatureRequest, AppSigner, AppVerification,
        Listener, MemoryStorage, Namespace, Namespaces, ProposalParams,
        RpcRequest, SignClient, SignConfig, SignEvent,
        protocol::{
            encoding::{b64u, from_b64u},
            types::{DappIntent, Metadata, Mode, PeerRequirement},
            verify::{decode_claims, hash_b64u},
        },
        transport::{
            HttpClient, HttpError, HttpRequest, HttpResponse, StreamResponse,
        },
    },
    serde_json::{Value, json},
    std::{
        sync::{Arc, Mutex},
        time::Duration,
    },
};

const CONNECTION_ID: &str = "09178356979748733185";

/// Records every call; answers create with a connection and fetch with
/// whatever the test set.
#[derive(Default)]
struct FakeRelay {
    calls: Mutex<Vec<(String, String, Value)>>,
    fetch: Mutex<Value>,
}

impl FakeRelay {
    fn bodies(&self, method: &str, path: &str) -> Vec<Value> {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|(m, p, _)| m == method && p == path)
            .map(|(_, _, b)| b.clone())
            .collect()
    }

    async fn wait_for(&self, method: &str, path: &str, n: usize) -> Value {
        for _ in 0..200 {
            if let Some(b) = self.bodies(method, path).get(n - 1) {
                return b.clone();
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("no {method} {path} #{n}");
    }
}

#[async_trait::async_trait]
impl HttpClient for FakeRelay {
    async fn request(
        &self,
        req: HttpRequest,
    ) -> Result<HttpResponse, HttpError> {
        let path = req
            .url
            .trim_start_matches("https://relay.test")
            .split('?')
            .next()
            .unwrap()
            .to_string();
        let body = req
            .body
            .as_deref()
            .and_then(|b| serde_json::from_slice(b).ok())
            .unwrap_or(Value::Null);
        self.calls.lock().unwrap().push((
            req.method.into(),
            path.clone(),
            body,
        ));
        let res = match (req.method, path.as_str()) {
            ("POST", "/v3/connections") => json!({
                "connectionId": CONNECTION_ID,
                "expiresAt": crate::rt::now_ms() / 1000 + 300,
            }),
            ("GET", _) => self.fetch.lock().unwrap().clone(),
            _ => {
                return Ok(HttpResponse {
                    status: 204,
                    retry_after_s: None,
                    body: vec![],
                });
            }
        };
        Ok(HttpResponse {
            status: 200,
            retry_after_s: None,
            body: serde_json::to_vec(&res).unwrap(),
        })
    }

    async fn stream(
        &self,
        _: HttpRequest,
    ) -> Result<StreamResponse, HttpError> {
        Err(HttpError::Network("no poll in tests".into()))
    }
}

/// Stands in for the app's backend: signs `sub` over the request string as
/// given, and `cth` as given.
#[derive(Default)]
struct TestSigner {
    seen: Mutex<Vec<AppSignatureRequest>>,
    fail: bool,
}

fn jwt(sub: &str, cth: Option<&str>) -> String {
    let claims = json!({ "iss": "app.example", "sub": sub, "cth": cth, "exp": 4102444800u64 });
    format!(
        "{}.{}.{}",
        b64u(br#"{"alg":"ES256","typ":"JWT"}"#),
        b64u(claims.to_string().as_bytes()),
        b64u(b"sig")
    )
}

#[async_trait::async_trait]
impl AppSigner for TestSigner {
    async fn sign_request(
        &self,
        request: AppSignatureRequest,
    ) -> Result<String, AppSignatureError> {
        self.seen.lock().unwrap().push(request.clone());
        if self.fail {
            return Err(AppSignatureError::Failed {
                message: "denylisted".into(),
            });
        }
        Ok(jwt(
            &hash_b64u(request.request.as_bytes()),
            request.ciphertext_hash.as_deref(),
        ))
    }
}

#[derive(Default)]
struct Events(Mutex<Vec<SignEvent>>);

impl Listener for Events {
    fn on_event(&self, event: SignEvent) {
        self.0.lock().unwrap().push(event);
    }
}

impl Events {
    fn requests(&self) -> Vec<(RpcRequest, AppVerification)> {
        self.0
            .lock()
            .unwrap()
            .iter()
            .filter_map(|e| match e {
                SignEvent::SessionRequest { request, verification, .. } => {
                    Some((request.clone(), verification.clone()))
                }
                _ => None,
            })
            .collect()
    }

    fn mismatches(&self) -> Vec<(Option<i64>, Option<String>)> {
        self.0
            .lock()
            .unwrap()
            .iter()
            .filter_map(|e| match e {
                SignEvent::AppSignatureMismatch { id, method, .. } => {
                    Some((*id, method.clone()))
                }
                _ => None,
            })
            .collect()
    }
}

async fn client(relay: &Arc<FakeRelay>, events: Arc<Events>) -> SignClient {
    let cfg = SignConfig {
        hosts: vec!["https://relay.test".into()],
        audience: None,
        project_id: "p".into(),
        metadata: Metadata {
            name: "Self-asserted name".into(),
            description: String::new(),
            url: "https://self-asserted.example".into(),
            icons: vec!["https://self-asserted.example/i.png".into()],
            redirect: Some(crate::Redirect {
                native: Some("app://".into()),
                ..Default::default()
            }),
        },
        versions: vec![],
        require_dapp: None,
        storage_prefix: "t:".into(),
        first_line_timeout_ms: None,
    };
    SignClient::init_with_http(
        cfg,
        Arc::new(MemoryStorage::default()),
        events,
        relay.clone(),
    )
    .await
    .unwrap()
}

fn params() -> ProposalParams {
    ProposalParams {
        required_namespaces: Namespaces::from([(
            "eip155".into(),
            Namespace {
                chains: Some(vec!["eip155:1".into()]),
                accounts: None,
                methods: vec!["personal_sign".into()],
                events: vec![],
            },
        )]),
        ..Default::default()
    }
}

/// `appMetadata` as the Relay delivers it: the JWT, verbatim, inside.
fn with_signature(app_signature: &Value, mut app_metadata: Value) -> Value {
    if !app_signature.is_null() {
        if app_metadata.is_null() {
            app_metadata = json!({});
        }
        app_metadata["appSignature"] = app_signature.clone();
    }
    app_metadata
}

fn fetched(create: &Value, app_signature: Value, app_metadata: Value) -> Value {
    json!({
        "bindingProof": create["bindingProof"],
        "proposal": create["proposal"],
        "versions": [3],
        "sealable": true,
        "appMetadata": with_signature(&app_signature, app_metadata),
    })
}

#[tokio::test]
async fn the_proposal_is_signed_and_the_wallet_checks_it() {
    let relay = Arc::new(FakeRelay::default());
    let dapp = client(&relay, Default::default()).await;
    let signer = Arc::new(TestSigner::default());
    dapp.set_app_signer(Some(signer.clone()));
    let uri = dapp.connect(params(), None).await.unwrap().uri;

    let create = relay.bodies("POST", "/v3/connections").remove(0);
    let payload =
        from_b64u(create["proposal"]["payload"].as_str().unwrap()).unwrap();
    let seen = signer.seen.lock().unwrap().clone();
    // The callback got exactly the proposal's payload bytes, unsealed.
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].request.as_bytes(), payload.as_slice());
    assert_eq!(seen[0].ciphertext_hash, None);
    let app_signature = create["appSignature"].clone();
    let claims = decode_claims(app_signature.as_str().unwrap()).unwrap();
    assert_eq!(claims.sub, hash_b64u(&payload));
    assert_eq!(claims.cth, None);

    let origin = json!({ "origin": "https://app.example", "name": "App" });
    *relay.fetch.lock().unwrap() =
        fetched(&create, app_signature.clone(), origin.clone());
    let wallet = client(&relay, Default::default()).await;
    let p = wallet.pair(uri.clone()).await.unwrap();
    assert_eq!(
        p.verification,
        AppVerification {
            domain: Some("app.example".into()),
            name: Some("App".into()),
            ..Default::default()
        }
    );
    // The dApp's own name, url and icons never reach the wallet.
    let shown = serde_json::to_string(&p).unwrap();
    assert!(!shown.contains("self-asserted"), "{shown}");
    assert_eq!(
        p.metadata.redirect.and_then(|r| r.native).as_deref(),
        Some("app://")
    );

    // A signature for another proposal: dropped, the dApp told with a
    // rejection, and the wallet with an error and an event.
    let other = json!(jwt(&hash_b64u(b"another proposal"), None));
    *relay.fetch.lock().unwrap() = fetched(&create, other, origin);
    let events = Arc::new(Events::default());
    let wallet = client(&relay, events.clone()).await;
    let e = wallet.pair(uri.clone()).await.unwrap_err();
    assert_eq!(e.code(), "app_signature_mismatch");
    assert_eq!(events.mismatches().len(), 1);
    assert!(
        !events
            .0
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, SignEvent::SessionProposal { .. }))
    );
    let rejection = relay
        .bodies("PUT", &format!("/v3/connections/{CONNECTION_ID}/rejection"))
        .remove(0);
    let payload =
        from_b64u(rejection["rejection"]["payload"].as_str().unwrap()).unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&payload).unwrap(),
        json!({ "connectionId": CONNECTION_ID })
    );

    // Unsigned: only the screening flags, whatever else the Relay sent.
    *relay.fetch.lock().unwrap() = fetched(
        &create,
        Value::Null,
        json!({ "origin": "https://app.example", "isScam": true }),
    );
    let wallet = client(&relay, Default::default()).await;
    let p = wallet.pair(uri).await.unwrap();
    assert_eq!(
        p.verification,
        AppVerification { is_scam: true, ..Default::default() }
    );
}

#[tokio::test]
async fn a_failing_callback_fails_the_call_and_sends_nothing() {
    let relay = Arc::new(FakeRelay::default());
    let dapp = client(&relay, Default::default()).await;
    dapp.set_app_signer(Some(Arc::new(TestSigner {
        fail: true,
        ..Default::default()
    })));
    let e = dapp.connect(params(), None).await.unwrap_err();
    assert_eq!(e.code(), "app_signature_failed");
    assert!(relay.bodies("POST", "/v3/connections").is_empty());

    // Without a callback, nothing is signed.
    dapp.set_app_signer(None);
    dapp.connect(params(), None).await.unwrap();
    assert!(
        relay.bodies("POST", "/v3/connections")[0]
            .get("appSignature")
            .is_none()
    );
}

#[tokio::test]
async fn sealed_requests_carry_sub_and_cth_and_the_wallet_checks_both() {
    let relay = Arc::new(FakeRelay::default());
    let dapp_events = Arc::new(Events::default());
    let dapp = client(&relay, dapp_events.clone()).await;
    let signer = Arc::new(TestSigner::default());
    dapp.set_app_signer(Some(signer.clone()));
    let sealed = DappIntent { own: Mode::Sealed, peer: PeerRequirement::None };
    let uri = dapp.connect(params(), Some(sealed)).await.unwrap().uri;
    let create = relay.bodies("POST", "/v3/connections").remove(0);
    *relay.fetch.lock().unwrap() =
        fetched(&create, create["appSignature"].clone(), Value::Null);

    let wallet_events = Arc::new(Events::default());
    let wallet = client(&relay, wallet_events.clone()).await;
    let p = wallet.pair(uri).await.unwrap();
    let mut accounts = params().required_namespaces;
    accounts.get_mut("eip155").unwrap().accounts =
        Some(vec!["eip155:1:0xabc".into()]);
    wallet.approve(p.id, accounts, None).await.unwrap();
    let accept = relay
        .bodies("PUT", &format!("/v3/connections/{CONNECTION_ID}/acceptance"))
        .remove(0);
    dapp.inner
        .handle_events(vec![json!({
            "type": "connection.established",
            "connectionId": CONNECTION_ID,
            "binding": accept["binding"],
        })])
        .await
        .unwrap();
    assert!(
        dapp_events
            .0
            .lock()
            .unwrap()
            .iter()
            .any(|e| matches!(e, SignEvent::SessionConnect { .. }))
    );

    let path = format!("/v3/connections/{CONNECTION_ID}/messages");
    let send = |params: &str| {
        let dapp = dapp.clone();
        let request = RpcRequest {
            method: "personal_sign".into(),
            params: params.into(),
        };
        tokio::spawn(async move {
            let _ = dapp
                .request(CONNECTION_ID.into(), "eip155:1".into(), request, None)
                .await;
        });
    };
    send(r#"["0x68656c6c6f","0xabc"]"#);
    let first = relay.wait_for("POST", &path, 1).await;
    let payload =
        from_b64u(first["message"]["payload"].as_str().unwrap()).unwrap();
    let seen = signer.seen.lock().unwrap()[1].clone();
    // Sealed: the callback got the plaintext, which is not what was sent,
    // and the hash of what was sent.
    assert_ne!(seen.request.as_bytes(), payload.as_slice());
    assert_eq!(seen.ciphertext_hash, Some(hash_b64u(&payload)));
    let claims =
        decode_claims(first["appSignature"].as_str().unwrap()).unwrap();
    assert_eq!(claims.sub, hash_b64u(seen.request.as_bytes()));
    assert_eq!(claims.cth, Some(hash_b64u(&payload)));

    let deliver = |body: &Value, app_signature: &Value| {
        json!({
            "type": "message.received",
            "connectionId": CONNECTION_ID,
            "message": body["message"],
            "appMetadata": with_signature(
                app_signature,
                json!({ "origin": "https://app.example", "name": "App", "icon": "https://app.example/i.png", "isVerified": true }),
            ),
        })
    };
    wallet
        .inner
        .handle_events(vec![deliver(&first, &first["appSignature"])])
        .await
        .unwrap();
    // The second request arrives with the first one's signature.
    send(r#"["0x6279","0xabc"]"#);
    let second = relay.wait_for("POST", &path, 2).await;
    wallet
        .inner
        .handle_events(vec![deliver(&second, &first["appSignature"])])
        .await
        .unwrap();

    // Only the first request reaches the wallet.
    let got = wallet_events.requests();
    assert_eq!(got.len(), 1);
    assert_eq!(
        got[0].1,
        AppVerification {
            domain: Some("app.example".into()),
            name: Some("App".into()),
            icon: Some("https://app.example/i.png".into()),
            is_verified: true,
            ..Default::default()
        }
    );
    // The second is dropped, reported, and answered with an error.
    let second_rpc: Value =
        serde_json::from_str(&signer.seen.lock().unwrap()[2].request).unwrap();
    let second_id = second_rpc["id"].as_i64();
    assert_eq!(
        wallet_events.mismatches(),
        vec![(second_id, Some("wc_sessionRequest".into()))]
    );
    let reply = relay.wait_for("POST", &path, 3).await;
    let reply: Value = serde_json::from_slice(
        &from_b64u(reply["message"]["payload"].as_str().unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(reply["id"].as_i64(), second_id);
    assert_eq!(
        reply["error"],
        json!({ "code": -32600, "message": "app_signature_mismatch" })
    );

    // The wallet's session shows the proposal's verification, and none of
    // the dApp's own identity.
    let session = wallet.sessions().await.remove(0);
    assert_eq!(session.wallet_metadata, None);
    assert_eq!(session.dapp_metadata.unwrap().description, "");
    assert_eq!(session.verification, Some(AppVerification::default()));
    let dapp_session = dapp.sessions().await.remove(0);
    assert_eq!(dapp_session.dapp_metadata, None);
    assert!(dapp_session.wallet_metadata.is_some());
}

#[tokio::test]
async fn a_retry_after_the_jwt_expired_asks_the_callback_again() {
    use crate::{
        client::state::Outbox,
        protocol::{types::WireMessage, verify::is_expired},
    };
    let relay = Arc::new(FakeRelay::default());
    let dapp = client(&relay, Default::default()).await;
    let signer = Arc::new(TestSigner::default());
    dapp.set_app_signer(Some(signer.clone()));
    let request = AppSignatureRequest {
        request:
            r#"{"id":1,"jsonrpc":"2.0","method":"wc_sessionPing","params":{}}"#
                .into(),
        ciphertext_hash: Some("cth".into()),
    };
    let now = crate::rt::now_ms() / 1000;
    let signed = |exp: u64| {
        format!(
            "{}.{}.{}",
            b64u(b"{}"),
            b64u(json!({ "sub": hash_b64u(request.request.as_bytes()), "exp": exp }).to_string().as_bytes()),
            b64u(b"sig")
        )
    };
    let out = |seq: u64, jwt: String| Outbox {
        conn: CONNECTION_ID.into(),
        seq,
        message: WireMessage {
            plain: "p".into(),
            payload: "x".into(),
            sig: "s".into(),
        },
        ttl: 300,
        app_signature: Some(jwt),
        app_signature_request: Some(request.clone()),
    };
    let path = format!("/v3/connections/{CONNECTION_ID}/messages");
    let expired = signed(now - 1);
    let fresh = signed(now + 120);
    assert!(is_expired(&expired, now));
    assert!(!is_expired(&fresh, now));
    // A crash left two unacknowledged sends: one whose JWT has expired since.
    dapp.inner
        .store
        .put(&format!("out/{CONNECTION_ID}/1"), &out(1, expired.clone()))
        .await
        .unwrap();
    dapp.inner
        .store
        .put(&format!("out/{CONNECTION_ID}/2"), &out(2, fresh.clone()))
        .await
        .unwrap();
    dapp.inner.flush_outbox().await;

    let sent: Vec<String> = relay
        .bodies("POST", &path)
        .iter()
        .map(|b| b["appSignature"].as_str().unwrap().to_string())
        .collect();
    // The expired one was signed again, for the same request, and never sent.
    assert_eq!(signer.seen.lock().unwrap().clone(), vec![request.clone()]);
    assert!(!sent.contains(&expired));
    assert_eq!(sent.len(), 2);
    assert!(sent.contains(&fresh));
    assert!(!is_expired(&sent[0], now));
    assert!(dapp.inner.store.keys("out/").await.unwrap().is_empty());

    // Without a callback to sign again, the expired send is dropped, unsent.
    dapp.set_app_signer(None);
    dapp.inner
        .store
        .put(&format!("out/{CONNECTION_ID}/3"), &out(3, expired.clone()))
        .await
        .unwrap();
    dapp.inner.flush_outbox().await;
    assert_eq!(relay.bodies("POST", &path).len(), 2);
    assert!(dapp.inner.store.keys("out/").await.unwrap().is_empty());
}
