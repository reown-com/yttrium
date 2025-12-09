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

#[derive(Debug, Clone, uniffi::Record)]
pub struct ConfirmResultFfi {
    pub result_type: String,
    pub value: String,
}

impl From<ConfirmResultFfi> for pay_api::ConfirmResult {
    fn from(r: ConfirmResultFfi) -> Self {
        Self { result_type: r.result_type, value: r.value }
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

    pub async fn confirm_payment(
        &self,
        payment_id: String,
        option_id: String,
        results: Vec<ConfirmResultFfi>,
    ) -> Result<(), PayError> {
        let results = results.into_iter().map(Into::into).collect();
        self.inner.confirm_payment(payment_id, option_id, results).await?;
        Ok(())
    }
}
