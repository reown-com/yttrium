#![cfg(feature = "test_depends_on_env_REOWN_PROJECT_ID")]

use crate::sign::test_helpers::test_sign_impl;

#[tokio::test]
async fn test_sign() {
    tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init();

    test_sign_impl().await.unwrap();
}
