#![cfg(feature = "test_depends_on_env_REOWN_PROJECT_ID")]
use crate::sign::Client;

#[tokio::test]
async fn test_sign() {
    let mut client =
        Client::new(std::env::var("REOWN_PROJECT_ID").unwrap().into());
    let uri = std::env::var("PAIRING_URI").unwrap();
    let pairing = client.pair(&uri).await.unwrap();
    client.approve(pairing).await.unwrap();
}
