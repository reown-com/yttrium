use crate::pay::{
    PayError as CorePayError, WalletConnectPay as CoreWalletConnectPay,
};

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum PayError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("API error: {code} - {message}")]
    Api { code: String, message: String },
}

impl From<CorePayError> for PayError {
    fn from(e: CorePayError) -> Self {
        match e {
            CorePayError::Http(e) => Self::Http(e.to_string()),
            CorePayError::Api { code, message } => Self::Api { code, message },
        }
    }
}

#[derive(uniffi::Object)]
pub struct WalletConnectPay {
    inner: CoreWalletConnectPay,
}

#[uniffi::export(async_runtime = "tokio")]
impl WalletConnectPay {
    #[uniffi::constructor]
    pub fn new(base_url: String) -> Self {
        Self { inner: CoreWalletConnectPay::new(base_url) }
    }

    pub async fn get_payment(
        &self,
        payment_id: String,
        accounts: Vec<String>,
    ) -> Result<(), PayError> {
        self.inner.get_payment(payment_id, accounts).await?;
        Ok(())
    }
}
