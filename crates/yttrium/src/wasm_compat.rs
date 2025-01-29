#[cfg(feature = "chain_abstraction_client")]
use {
    crate::{
        call::Call,
        chain_abstraction::{
            api::{
                prepare::{PrepareResponse, PrepareResponseAvailable},
                status::{StatusResponse, StatusResponseCompleted},
            },
            client::Client as InnerClient,
            currency::Currency,
            error::{
                PrepareDetailedResponse, PrepareError, UiFieldsError,
                WaitForSuccessError,
            },
            ui_fields::UiFields,
        },
    },
    alloy::primitives::Address,
    std::time::Duration,
    wasm_bindgen::prelude::*,
};

#[cfg(feature = "chain_abstraction_client")]
#[wasm_bindgen(getter_with_clone)]
pub struct Client {
    inner: InnerClient,
}

#[cfg(feature = "chain_abstraction_client")]
#[wasm_bindgen]
impl Client {
    #[wasm_bindgen(constructor)]
    pub fn new(project_id: String) -> Self {
        Self { inner: InnerClient::new(project_id.into()) }
    }

    #[wasm_bindgen]
    pub async fn prepare(
        &self,
        chain_id: String,
        from: String,
        call: Call,
    ) -> Result<PrepareResponse, JsValue> {
        self.inner
            .prepare(
                chain_id,
                from.parse::<Address>()
                    .map_err(|e| JsValue::from_str(&e.to_string()))?,
                call,
            )
            .await
            .map_err(Into::into)
    }

    #[wasm_bindgen]
    pub async fn get_ui_fields(
        &self,
        prepare_response: PrepareResponseAvailable,
        local_currency: Currency,
    ) -> Result<UiFields, UiFieldsError> {
        self.inner.get_ui_fields(prepare_response, local_currency).await
    }

    #[wasm_bindgen]
    pub async fn prepare_detailed(
        &self,
        chain_id: String,
        from: String,
        call: Call,
        local_currency: Currency,
    ) -> Result<PrepareDetailedResponse, JsValue> {
        self.inner
            .prepare_detailed(
                chain_id,
                from.parse::<Address>()
                    .map_err(|e| JsValue::from_str(&e.to_string()))?,
                call,
                local_currency,
            )
            .await
            .map_err(Into::into)
    }

    #[wasm_bindgen]
    pub async fn status(
        &self,
        orchestration_id: String,
    ) -> Result<StatusResponse, PrepareError> {
        self.inner.status(orchestration_id).await
    }

    #[wasm_bindgen]
    pub async fn wait_for_success(
        &self,
        orchestration_id: String,
        check_in_ms: u64,
    ) -> Result<StatusResponseCompleted, WaitForSuccessError> {
        self.inner
            .wait_for_success(
                orchestration_id,
                Duration::from_millis(check_in_ms),
            )
            .await
    }

    pub async fn wait_for_success_with_timeout(
        &self,
        orchestration_id: String,
        check_in_ms: u64,
        timeout_ms: u64,
    ) -> Result<StatusResponseCompleted, WaitForSuccessError> {
        self.inner
            .wait_for_success_with_timeout(
                orchestration_id,
                Duration::from_millis(check_in_ms),
                Duration::from_millis(timeout_ms),
            )
            .await
    }

    pub async fn erc20_token_balance(
        &self,
        chain_id: &str,
        token: String,
        owner: String,
    ) -> Result<String, String> {
        Ok(self
            .inner
            .erc20_token_balance(
                chain_id,
                token.parse::<Address>().map_err(|e| e.to_string())?,
                owner.parse::<Address>().map_err(|e| e.to_string())?,
            )
            .await
            .map_err(|e| e.to_string())?
            .to_string())
    }
}
