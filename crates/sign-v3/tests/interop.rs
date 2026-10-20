//! Cross-implementation: the Rust SDK against a TypeScript Sign v3 peer, through the same Relay.
//! Needs SIGN_V3_RELAY_HOSTS and SIGN_V3_TS_PEER_DIR, a directory whose `scripts/ts-peer.ts` plays
//! the dApp or the wallet.
mod common;

use {
    common::{Opts, auto_respond, node, require_relay, wait_for},
    serde_json::{Value, json},
    sign_v3::SignEvent,
    std::{
        io::{BufRead, BufReader},
        process::{Child, Command, Stdio},
        sync::mpsc,
        time::Duration,
    },
};

struct Peer {
    child: Child,
    lines: mpsc::Receiver<Value>,
}

impl Drop for Peer {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

fn ts_peer(hosts: &[String], args: &[&str]) -> Option<Peer> {
    let dir = std::env::var("SIGN_V3_TS_PEER_DIR").ok()?;
    let mut child = Command::new("npx")
        .args(["tsx", "scripts/ts-peer.ts"])
        .args(args)
        .current_dir(dir)
        .env("HOSTS", hosts.join(","))
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn ts peer");
    let stdout = child.stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            if let Ok(v) = serde_json::from_str::<Value>(&line) {
                let _ = tx.send(v);
            }
        }
    });
    Some(Peer { child, lines: rx })
}

fn next_with(p: &Peer, key: &str) -> Value {
    loop {
        let v = p
            .lines
            .recv_timeout(Duration::from_secs(30))
            .unwrap_or_else(|_| panic!("ts peer: no `{key}` line"));
        assert!(v.get("error").is_none(), "ts peer error: {v}");
        if v.get(key).is_some() {
            return v;
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn rust_dapp_pairs_with_ts_wallet() {
    let hosts = require_relay!();
    let d = node(&hosts, Opts::default()).await;
    let mut dev = d.subscribe();
    let c = d.client.connect(common::required(), None).await.unwrap();
    let Some(w) = ts_peer(&hosts, &["wallet", &c.uri]) else {
        eprintln!("SIGN_V3_TS_PEER_DIR not set: skipping");
        return;
    };
    let SignEvent::SessionConnect { session } =
        wait_for(&mut dev, |e| matches!(e, SignEvent::SessionConnect { .. }))
            .await
    else {
        unreachable!()
    };
    assert_eq!(session.peer_metadata.name, "ts-wallet");
    assert_eq!(next_with(&w, "approved")["approved"], session.topic);
    let r = d
        .client
        .request(
            session.topic.clone(),
            "eip155:1".into(),
            common::sign_req(),
            None,
        )
        .await
        .unwrap();
    assert_eq!(r, "\"0xts\"");
    assert_eq!(
        next_with(&w, "request")["request"]["request"]["method"],
        "personal_sign"
    );
    d.client.disconnect(session.topic).await.unwrap();
    assert_eq!(next_with(&w, "deleted")["deleted"]["verified"], true);
}

#[tokio::test(flavor = "multi_thread")]
async fn ts_dapp_pairs_with_rust_wallet() {
    let hosts = require_relay!();
    let Some(dp) = ts_peer(&hosts, &["dapp"]) else {
        eprintln!("SIGN_V3_TS_PEER_DIR not set: skipping");
        return;
    };
    let w = node(&hosts, Opts { wallet: true, ..Default::default() }).await;
    let mut wev = w.subscribe();
    auto_respond(&w, json!("0xrust"));
    let uri = next_with(&dp, "uri")["uri"].as_str().unwrap().to_string();
    let p = w.client.pair(uri).await.unwrap();
    assert_eq!(p.metadata.name, "ts-dapp");
    let s = w.client.approve(p.id, common::approved(), None).await.unwrap();
    let connected = next_with(&dp, "connected");
    assert_eq!(connected["connected"], s.topic);
    assert_eq!(connected["peer"], "wallet");
    assert_eq!(next_with(&dp, "result")["result"], "0xrust");
    let e =
        wait_for(&mut wev, |e| matches!(e, SignEvent::SessionDelete { .. }))
            .await;
    assert!(matches!(e, SignEvent::SessionDelete { verified: true, .. }));
}
