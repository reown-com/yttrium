//! Verify v4 App Signatures through the client, against an in-memory Relay:
//! the dApp signs exactly the bytes it sends, and the wallet checks them.

use {
    crate::{
        AppSignatureError, AppSignatureRequest, AppSigner, Listener,
        MemoryStorage, Namespace, Namespaces, ProposalParams, RpcRequest,
        SignClient, SignConfig, SignEvent, VerifyContext, VerifyState,
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
    fn requests(&self) -> Vec<(RpcRequest, VerifyContext)> {
        self.0
            .lock()
            .unwrap()
            .iter()
            .filter_map(|e| match e {
                SignEvent::SessionRequest {
                    request, verify_context, ..
                } => Some((request.clone(), verify_context.clone())),
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
            icons: vec![],
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

fn fetched(create: &Value, app_signature: Value, app_metadata: Value) -> Value {
    json!({
        "bindingProof": create["bindingProof"],
        "proposal": create["proposal"],
        "versions": [3],
        "sealable": true,
        "appSignature": app_signature,
        "appMetadata": app_metadata,
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
        p.verify_context,
        VerifyContext {
            state: VerifyState::BusinessName,
            origin: Some("https://app.example".into()),
            name: Some("App".into()),
            ..Default::default()
        }
    );

    // A signature for another proposal: invalid, and none of the metadata.
    let other = json!(jwt(&hash_b64u(b"another proposal"), None));
    *relay.fetch.lock().unwrap() = fetched(&create, other, origin);
    let wallet = client(&relay, Default::default()).await;
    let p = wallet.pair(uri.clone()).await.unwrap();
    assert_eq!(
        p.verify_context,
        VerifyContext { state: VerifyState::Invalid, ..Default::default() }
    );

    // Unsigned: Default. The app's own metadata never becomes the origin.
    *relay.fetch.lock().unwrap() =
        fetched(&create, Value::Null, json!({ "isScam": true }));
    let wallet = client(&relay, Default::default()).await;
    let p = wallet.pair(uri).await.unwrap();
    assert_eq!(
        p.verify_context,
        VerifyContext {
            state: VerifyState::Default,
            is_scam: Some(true),
            ..Default::default()
        }
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
            "appSignature": app_signature,
            "appMetadata": { "origin": "https://app.example", "name": "App", "icon": "https://app.example/i.png", "isVerified": true },
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

    let got = wallet_events.requests();
    assert_eq!(got.len(), 2);
    assert_eq!(
        got[0].1,
        VerifyContext {
            state: VerifyState::Verified,
            origin: Some("https://app.example".into()),
            name: Some("App".into()),
            icon: Some("https://app.example/i.png".into()),
            ..Default::default()
        }
    );
    // Still surfaced, as invalid, with nothing about the app.
    assert_eq!(got[1].0.params, r#"["0x6279","0xabc"]"#);
    assert_eq!(
        got[1].1,
        VerifyContext { state: VerifyState::Invalid, ..Default::default() }
    );
}
