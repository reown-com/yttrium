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
    let uri = std::env::var("PAIRING_URI").unwrap();
    let pairing = client.pair(&uri).await.unwrap();
    client.approve(pairing).await.unwrap();
}
