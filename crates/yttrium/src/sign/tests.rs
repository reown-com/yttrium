use crate::sign::Client;

const RELAY_URL: &str = "wss://relay.walletconnect.org";
const CLIENT_ID: &str = "123";

#[tokio::test]
#[cfg(feature = "test_depends_on_env_REOWN_PROJECT_ID")]
async fn test_sign() {
    let mut client = Client::new(
        RELAY_URL.to_owned(),
        std::env::var("REOWN_PROJECT_ID").unwrap().into(),
        CLIENT_ID.to_owned().into(),
    );
    let uri = "wc:cd2188da9945d1d7ed4ace9f39e4336ddb8291c0979129b452f202c7a5033f9a@2?relay-protocol=irn&symKey=bece1ba80ff0a012e06bf44821ac2bf3ec039095832d20736858264212b849df&expiryTimestamp=1752855750";
    let pairing = client.pair(uri).await.unwrap();
    client.approve(pairing).await.unwrap();
}
