//! Shared helpers for tests against a running v3 Relay, for example:
//!   SIGN_V3_RELAY_HOSTS=https://<v3 relay host> cargo test -p sign-v3 --features rustls
//! A Relay with debug routes and a second host also runs the tamper, sealing and failover tests.
#![allow(dead_code)]

use {
    serde_json::{Value, json},
    sign_v3::{
        Listener, MemoryStorage, Namespace, Namespaces, ProposalParams,
        RpcRequest, SignClient, SignConfig, SignEvent,
        protocol::types::Metadata,
    },
    std::{collections::HashMap, sync::Arc, time::Duration},
    tokio::sync::broadcast,
};

pub fn hosts() -> Option<Vec<String>> {
    let v = std::env::var("SIGN_V3_RELAY_HOSTS").ok()?;
    Some(v.split(',').map(|s| s.trim().to_string()).collect())
}

macro_rules! require_relay {
    () => {
        match common::hosts() {
            Some(h) => h,
            None => {
                eprintln!("SIGN_V3_RELAY_HOSTS not set: skipping");
                return;
            }
        }
    };
}
pub(crate) use require_relay;

/// Skips a test that needs what only the mock Relay has: its debug routes, or sealing.
#[allow(unused_macros)]
macro_rules! require_mock {
    ($hosts:expr, $why:literal) => {
        if !common::is_mock($hosts).await {
            eprintln!("not the mock Relay: skipping, {}", $why);
            return;
        }
    };
}
#[allow(unused_imports)]
pub(crate) use require_mock;

/// The mock Relay answers its debug routes; a deployed Relay doesn't.
pub async fn is_mock(hosts: &[String]) -> bool {
    match reqwest::get(format!("{}/debug/seen", hosts[0])).await {
        Ok(r) => r.status().is_success(),
        Err(_) => false,
    }
}

struct ChannelListener(broadcast::Sender<SignEvent>);
impl Listener for ChannelListener {
    fn on_event(&self, event: SignEvent) {
        let _ = self.0.send(event);
    }
}

pub struct Node {
    pub client: SignClient,
    pub events: broadcast::Sender<SignEvent>,
    pub storage: Arc<MemoryStorage>,
}

impl Node {
    pub fn subscribe(&self) -> broadcast::Receiver<SignEvent> {
        self.events.subscribe()
    }
}

pub fn meta(name: &str) -> Metadata {
    Metadata {
        name: name.into(),
        description: name.into(),
        url: format!("https://{name}.example"),
        icons: vec![],
    }
}

pub struct Opts {
    pub project: &'static str,
    pub wallet: bool,
    pub storage: Option<Arc<MemoryStorage>>,
    pub require_dapp: Option<sign_v3::protocol::types::PeerRequirement>,
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            project: "default",
            wallet: false,
            storage: None,
            require_dapp: None,
        }
    }
}

pub async fn node(hosts: &[String], o: Opts) -> Node {
    let storage = o.storage.unwrap_or_default();
    let (tx, _) = broadcast::channel(1024);
    let cfg = SignConfig {
        hosts: hosts.to_vec(),
        audience: None,
        project_id: o.project.into(),
        metadata: meta(if o.wallet { "wallet" } else { "dapp" }),
        versions: vec![],
        require_dapp: o.require_dapp,
        storage_prefix: "wc3:".into(),
        first_line_timeout_ms: Some(1500),
    };
    let client = SignClient::init(
        cfg,
        storage.clone(),
        Arc::new(ChannelListener(tx.clone())),
    )
    .await
    .expect("init");
    client.start();
    Node { client, events: tx, storage }
}

pub async fn wait_for<F: Fn(&SignEvent) -> bool>(
    rx: &mut broadcast::Receiver<SignEvent>,
    f: F,
) -> SignEvent {
    tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let e = rx.recv().await.expect("event stream");
            if f(&e) {
                return e;
            }
        }
    })
    .await
    .expect("timed out waiting for event")
}

fn ns(accounts: Option<Vec<String>>) -> Namespaces {
    HashMap::from([(
        "eip155".to_string(),
        Namespace {
            chains: Some(vec!["eip155:1".into()]),
            accounts,
            methods: vec!["personal_sign".into()],
            events: vec!["chainChanged".into()],
        },
    )])
}

pub fn required() -> ProposalParams {
    ProposalParams { required_namespaces: ns(None), ..Default::default() }
}

pub fn approved() -> Namespaces {
    ns(Some(vec!["eip155:1:0x1111111111111111111111111111111111111111".into()]))
}

pub fn sign_req() -> RpcRequest {
    RpcRequest {
        method: "personal_sign".into(),
        params: json!(["0xdead", "0x1111111111111111111111111111111111111111"])
            .to_string(),
    }
}

/// Pairs and returns the connectionId.
pub async fn pair(
    dapp: &Node,
    wallet: &Node,
    intent: Option<sign_v3::protocol::types::DappIntent>,
    own: Option<sign_v3::protocol::types::Mode>,
) -> String {
    let mut dev = dapp.subscribe();
    let c = dapp.client.connect(required(), intent).await.expect("connect");
    let p = wallet.client.pair(c.uri.clone()).await.expect("pair");
    let s =
        wallet.client.approve(p.id, approved(), own).await.expect("approve");
    let SignEvent::SessionConnect { session } =
        wait_for(&mut dev, |e| matches!(e, SignEvent::SessionConnect { .. }))
            .await
    else {
        unreachable!()
    };
    assert_eq!(session.topic, s.topic);
    assert_eq!(c.topic, s.topic);
    s.topic
}

/// Answers every session_request with a fixed result.
pub fn auto_respond(wallet: &Node, result: Value) {
    let mut rx = wallet.subscribe();
    let client = wallet.client.clone();
    tokio::spawn(async move {
        while let Ok(e) = rx.recv().await {
            if let SignEvent::SessionRequest { id, topic, .. } = e {
                let c = client.clone();
                let r = result.to_string();
                tokio::spawn(async move {
                    let _ = c.respond(topic, id, Some(r), None).await;
                });
            }
        }
    });
}

pub async fn debug(hosts: &[String], method: &str, path: &str) -> Value {
    let url = format!("{}{path}", hosts[0]);
    let c = reqwest::Client::new();
    let res = if method == "POST" {
        c.post(url).send().await
    } else {
        c.get(url).send().await
    };
    res.expect("debug route").json().await.unwrap_or(Value::Null)
}
