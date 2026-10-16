//! End-to-end against a running v3 Relay (see tests/common/mod.rs).
mod common;

use {
    common::{
        Opts, auto_respond, debug, node, pair, require_mock, require_relay,
        sign_req, wait_for,
    },
    serde_json::{Value, json},
    sign_v3::{
        MemoryStorage, Namespace, Namespaces, SignEvent,
        protocol::types::{DappIntent, Mode, PeerRequirement},
    },
    std::sync::Arc,
};

#[tokio::test(flavor = "multi_thread")]
async fn pairs_and_round_trips_a_monitored_request() {
    let hosts = require_relay!();
    let d = node(&hosts, Opts::default()).await;
    let w = node(&hosts, Opts { wallet: true, ..Default::default() }).await;
    let mut wev = w.subscribe();
    let topic = pair(&d, &w, None, None).await;
    assert!(sign_v3::protocol::uri::is_connection_id(&topic));
    let dc = d.client.clone();
    let t = topic.clone();
    let handle = tokio::spawn(async move {
        dc.request(t, "eip155:1".into(), sign_req(), None).await
    });
    let SignEvent::SessionRequest { id, chain_id, request, .. } =
        wait_for(&mut wev, |e| matches!(e, SignEvent::SessionRequest { .. }))
            .await
    else {
        unreachable!()
    };
    assert_eq!(chain_id, "eip155:1");
    assert_eq!(request, sign_req());
    w.client
        .respond(topic.clone(), id, Some("\"0xsig\"".into()), None)
        .await
        .unwrap();
    assert_eq!(handle.await.unwrap().unwrap(), "\"0xsig\"");
    if common::is_mock(&hosts).await {
        let seen = debug(&hosts, "GET", "/debug/seen").await;
        assert!(
            seen.as_array().unwrap().iter().any(|s| s["connectionId"] == topic
                && s["payload"].as_str().unwrap().contains("personal_sign"))
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn seals_both_directions_and_the_relay_sees_only_ciphertext() {
    let hosts = require_relay!();
    require_mock!(&hosts, "sealing and /debug/seen");
    let d =
        node(&hosts, Opts { project: "entitled", ..Default::default() }).await;
    let w = node(
        &hosts,
        Opts { project: "entitled", wallet: true, ..Default::default() },
    )
    .await;
    let topic = pair(
        &d,
        &w,
        Some(DappIntent { own: Mode::Sealed, peer: PeerRequirement::Sealed }),
        Some(Mode::Sealed),
    )
    .await;
    auto_respond(&w, json!("0xsealed"));
    let r = d
        .client
        .request(topic.clone(), "eip155:1".into(), sign_req(), None)
        .await
        .unwrap();
    assert_eq!(r, "\"0xsealed\"");
    let seen = debug(&hosts, "GET", "/debug/seen").await;
    let mine: Vec<&Value> = seen
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| {
            s["connectionId"] == topic && (s["tag"] == 1108 || s["tag"] == 1109)
        })
        .collect();
    assert_eq!(mine.len(), 2);
    assert!(mine.iter().all(|s| s["sealed"] == true
        && s["payload"].as_str().unwrap().starts_with("ciphertext")));
}

#[tokio::test(flavor = "multi_thread")]
async fn delivers_a_signed_rejection() {
    let hosts = require_relay!();
    let d = node(&hosts, Opts::default()).await;
    let w = node(&hosts, Opts { wallet: true, ..Default::default() }).await;
    let mut dev = d.subscribe();
    let c = d.client.connect(common::required(), None).await.unwrap();
    let p = w.client.pair(c.uri.clone()).await.unwrap();
    w.client.reject(p.id, None).await.unwrap();
    let e =
        wait_for(&mut dev, |e| matches!(e, SignEvent::ProposalRejected { .. }))
            .await;
    assert!(
        matches!(e, SignEvent::ProposalRejected { reason: Some(ref r), .. } if r == "user_rejected")
    );
    // Once resolved, only its members may fetch it.
    assert_eq!(w.client.pair(c.uri).await.unwrap_err().code(), "forbidden");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_newer_connect_replaces_the_pending_one() {
    let hosts = require_relay!();
    let d = node(&hosts, Opts::default()).await;
    let w = node(&hosts, Opts { wallet: true, ..Default::default() }).await;
    let mut dev = d.subscribe();
    let first = d.client.connect(common::required(), None).await.unwrap();
    let second = d.client.connect(common::required(), None).await.unwrap();
    let e =
        wait_for(&mut dev, |e| matches!(e, SignEvent::ProposalExpire { .. }))
            .await;
    assert!(
        matches!(e, SignEvent::ProposalExpire { ref topic, ref reason } if *topic == first.topic && reason == "replaced")
    );
    assert_eq!(w.client.pair(first.uri).await.unwrap_err().code(), "not_found");
    let p = w.client.pair(second.uri).await.unwrap();
    assert_eq!(p.id, second.topic);
    w.client.approve(p.id, common::approved(), None).await.unwrap();
    let e =
        wait_for(&mut dev, |e| matches!(e, SignEvent::SessionConnect { .. }))
            .await;
    assert!(
        matches!(e, SignEvent::SessionConnect { ref session } if session.topic == second.topic)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn wallet_policy_rejects_an_unacceptable_dapp_direction() {
    let hosts = require_relay!();
    let d =
        node(&hosts, Opts { project: "entitled", ..Default::default() }).await;
    let w = node(
        &hosts,
        Opts {
            project: "entitled",
            wallet: true,
            require_dapp: Some(PeerRequirement::Monitored),
            ..Default::default()
        },
    )
    .await;
    let mut dev = d.subscribe();
    let c = d
        .client
        .connect(
            common::required(),
            Some(DappIntent { own: Mode::Sealed, peer: PeerRequirement::None }),
        )
        .await
        .unwrap();
    assert_eq!(
        w.client.pair(c.uri).await.unwrap_err().code(),
        "incompatible_intent"
    );
    let e =
        wait_for(&mut dev, |e| matches!(e, SignEvent::ProposalRejected { .. }))
            .await;
    assert!(
        matches!(e, SignEvent::ProposalRejected { reason: Some(ref r), .. } if r == "incompatible_intent")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn only_the_wallet_updates_and_extends() {
    let hosts = require_relay!();
    let d = node(&hosts, Opts::default()).await;
    let w = node(&hosts, Opts { wallet: true, ..Default::default() }).await;
    let topic = pair(&d, &w, None, None).await;
    let mut dev = d.subscribe();
    let ns = Namespaces::from([(
        "eip155".to_string(),
        Namespace {
            chains: Some(vec!["eip155:1".into(), "eip155:10".into()]),
            accounts: Some(vec![]),
            methods: vec![],
            events: vec![],
        },
    )]);
    w.client.update(topic.clone(), ns.clone()).await.unwrap();
    let SignEvent::SessionUpdate { namespaces, .. } =
        wait_for(&mut dev, |e| matches!(e, SignEvent::SessionUpdate { .. }))
            .await
    else {
        unreachable!()
    };
    assert_eq!(namespaces, ns);
    assert_eq!(d.client.sessions().await[0].namespaces, ns);
    assert_eq!(
        d.client.update(topic.clone(), ns).await.unwrap_err().code(),
        "wallet_only"
    );
    assert_eq!(
        d.client.extend(topic.clone(), None).await.unwrap_err().code(),
        "wallet_only"
    );
    let before = d.client.sessions().await[0].expiry;
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    w.client.extend(topic.clone(), None).await.unwrap();
    let SignEvent::SessionExtend { expiry, .. } =
        wait_for(&mut dev, |e| matches!(e, SignEvent::SessionExtend { .. }))
            .await
    else {
        unreachable!()
    };
    assert!(expiry > before);
}

#[tokio::test(flavor = "multi_thread")]
async fn closes_with_a_close_the_peer_verifies() {
    let hosts = require_relay!();
    let d = node(&hosts, Opts::default()).await;
    let w = node(&hosts, Opts { wallet: true, ..Default::default() }).await;
    let topic = pair(&d, &w, None, None).await;
    let mut wev = w.subscribe();
    d.client.disconnect(topic.clone()).await.unwrap();
    let e =
        wait_for(&mut wev, |e| matches!(e, SignEvent::SessionDelete { .. }))
            .await;
    assert!(
        matches!(e, SignEvent::SessionDelete { verified: true, ref reason, .. } if reason == "closed")
    );
    assert_eq!(w.client.ping(topic).await.unwrap_err().code(), "not_found");
}

#[tokio::test(flavor = "multi_thread")]
async fn resumes_from_storage_with_the_same_identity() {
    let hosts = require_relay!();
    let ds = Arc::new(MemoryStorage::default());
    let ws = Arc::new(MemoryStorage::default());
    let d1 =
        node(&hosts, Opts { storage: Some(ds.clone()), ..Default::default() })
            .await;
    let w1 = node(
        &hosts,
        Opts { wallet: true, storage: Some(ws.clone()), ..Default::default() },
    )
    .await;
    let topic = pair(&d1, &w1, None, None).await;
    let id = d1.client.client_id();
    d1.client.stop();
    w1.client.stop();
    let d2 =
        node(&hosts, Opts { storage: Some(ds), ..Default::default() }).await;
    let w2 = node(
        &hosts,
        Opts { wallet: true, storage: Some(ws), ..Default::default() },
    )
    .await;
    assert_eq!(d2.client.client_id(), id);
    assert_eq!(d2.client.sessions().await[0].topic, topic);
    auto_respond(&w2, json!("0xafter"));
    assert_eq!(
        d2.client
            .request(topic, "eip155:1".into(), sign_req(), None)
            .await
            .unwrap(),
        "\"0xafter\""
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn drops_a_message_tampered_in_transit() {
    let hosts = require_relay!();
    require_mock!(&hosts, "/debug/tamper-next");
    let d = node(&hosts, Opts::default()).await;
    let w = node(&hosts, Opts { wallet: true, ..Default::default() }).await;
    let topic = pair(&d, &w, None, None).await;
    let mut wev = w.subscribe();
    debug(&hosts, "POST", &format!("/debug/tamper-next?connection={topic}"))
        .await;
    let c = d.client.clone();
    tokio::spawn(async move {
        let _ = c.request(topic, "eip155:1".into(), sign_req(), Some(5)).await;
    });
    let e = wait_for(&mut wev, |e| {
        matches!(
            e,
            SignEvent::MessageInvalid { .. } | SignEvent::SessionRequest { .. }
        )
    })
    .await;
    assert!(
        matches!(e, SignEvent::MessageInvalid { ref reason, .. } if reason == "invalid signature"),
        "{e:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn refuses_a_v2_uri() {
    let hosts = require_relay!();
    let w = node(&hosts, Opts { wallet: true, ..Default::default() }).await;
    let v2 = format!(
        "wc:{}@2?relay-protocol=irn&symKey={}&expiryTimestamp=1",
        "a".repeat(64),
        "b".repeat(64)
    );
    assert_eq!(w.client.pair(v2).await.unwrap_err().code(), "unsupported_uri");
}

#[tokio::test(flavor = "multi_thread")]
async fn wallet_answers_a_request_after_a_restart() {
    let hosts = require_relay!();
    let ws = Arc::new(MemoryStorage::default());
    let d = node(&hosts, Opts::default()).await;
    let w1 = node(
        &hosts,
        Opts { wallet: true, storage: Some(ws.clone()), ..Default::default() },
    )
    .await;
    let topic = pair(&d, &w1, None, None).await;
    let mut wev = w1.subscribe();
    let dc = d.client.clone();
    let t = topic.clone();
    let pending = tokio::spawn(async move {
        dc.request(t, "eip155:1".into(), sign_req(), None).await
    });
    let SignEvent::SessionRequest { id, .. } =
        wait_for(&mut wev, |e| matches!(e, SignEvent::SessionRequest { .. }))
            .await
    else {
        unreachable!()
    };
    w1.client.stop();
    let w2 = node(
        &hosts,
        Opts { wallet: true, storage: Some(ws.clone()), ..Default::default() },
    )
    .await;
    assert_eq!(
        w2.client
            .respond(topic.clone(), id + 1, Some("\"0x\"".into()), None)
            .await
            .unwrap_err()
            .code(),
        "not_found"
    );
    w2.client
        .respond(topic.clone(), id, Some("\"0xafter-restart\"".into()), None)
        .await
        .unwrap();
    assert_eq!(pending.await.unwrap().unwrap(), "\"0xafter-restart\"");
    assert!(!ws.snapshot().keys().any(|k| k.contains("inreq/")));
}

#[tokio::test(flavor = "multi_thread")]
async fn dapp_matches_a_response_after_a_restart() {
    let hosts = require_relay!();
    let ds = Arc::new(MemoryStorage::default());
    let d1 =
        node(&hosts, Opts { storage: Some(ds.clone()), ..Default::default() })
            .await;
    let w = node(&hosts, Opts { wallet: true, ..Default::default() }).await;
    let topic = pair(&d1, &w, None, None).await;
    let mut wev = w.subscribe();
    let dc = d1.client.clone();
    let t = topic.clone();
    tokio::spawn(async move {
        let _ = dc.request(t, "eip155:1".into(), sign_req(), None).await;
    });
    let SignEvent::SessionRequest { id, .. } =
        wait_for(&mut wev, |e| matches!(e, SignEvent::SessionRequest { .. }))
            .await
    else {
        unreachable!()
    };
    assert!(
        ds.snapshot()
            .keys()
            .any(|k| k.contains(&format!("outreq/{topic}/{id}")))
    );
    d1.client.stop();
    let d2 =
        node(&hosts, Opts { storage: Some(ds.clone()), ..Default::default() })
            .await;
    let mut dev = d2.subscribe();
    w.client
        .respond(topic.clone(), id, Some("\"0xlate\"".into()), None)
        .await
        .unwrap();
    let SignEvent::SessionResponse { response, request, .. } =
        wait_for(&mut dev, |e| matches!(e, SignEvent::SessionResponse { .. }))
            .await
    else {
        unreachable!()
    };
    assert_eq!(response.id, id);
    assert_eq!(response.result.as_deref(), Some("\"0xlate\""));
    assert_eq!(request.expect("matched request"), sign_req());
    assert!(!ds.snapshot().keys().any(|k| k.contains("outreq/")));
}
