use crate::sign::test_helpers::test_sign_impl;

#[tokio::test]
#[ignore = "blocked on relay-side irn_fetchMessages bug"]
async fn test_sign() {
    tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init();

    test_sign_impl().await.unwrap();
}
