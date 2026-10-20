//! Host failover. Its own test binary, because it blackholes a shared Relay host.
mod common;

use {
    common::{
        Opts, auto_respond, debug, node, pair, require_mock, require_relay,
        sign_req,
    },
    serde_json::json,
    std::time::Duration,
};

#[tokio::test(flavor = "multi_thread")]
async fn moves_to_the_other_host_when_the_poll_goes_silent() {
    let hosts = require_relay!();
    require_mock!(&hosts, "/debug/blackhole and a second host");
    debug(&hosts, "POST", "/debug/reset").await;
    let d = node(&hosts, Opts::default()).await;
    let w = node(&hosts, Opts { wallet: true, ..Default::default() }).await;
    let topic = pair(&d, &w, None, None).await;
    auto_respond(&w, json!("0xfailover"));
    assert_eq!(d.client.relay().host_index(), 0);
    reqwest::Client::new()
        .post(format!("{}/debug/blackhole?host=0&on=1", hosts[1]))
        .send()
        .await
        .unwrap();
    let moved = tokio::time::timeout(Duration::from_secs(10), async {
        while d.client.relay().host_index() != 1
            || w.client.relay().host_index() != 1
        {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await;
    reqwest::Client::new()
        .post(format!("{}/debug/reset", hosts[1]))
        .send()
        .await
        .unwrap();
    moved.expect("both clients moved to host 1");
    assert_eq!(
        d.client
            .request(topic, "eip155:1".into(), sign_req(), None)
            .await
            .unwrap(),
        "\"0xfailover\""
    );
}
