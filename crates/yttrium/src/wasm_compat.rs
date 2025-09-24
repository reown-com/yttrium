use wasm_bindgen::prelude::*;
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
            client::ExecuteDetails,
            currency::Currency,
            error::{
                PrepareDetailedResponse, StatusError, UiFieldsError,
                WaitForSuccessError,
            },
            pulse::PulseMetadata,
            ui_fields::{RouteSig, UiFields},
        },
    },
    std::time::Duration,
};

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = include_str!("wasm_compat.ts");

#[cfg(feature = "chain_abstraction_client")]
#[wasm_bindgen(getter_with_clone)]
pub struct Client {
    inner: InnerClient,
}

#[cfg(feature = "chain_abstraction_client")]
#[wasm_bindgen]
impl Client {
    #[wasm_bindgen(constructor)]
    pub fn new(project_id: String, pulse_metadata: PulseMetadata) -> Self {
        Self { inner: InnerClient::new(project_id.into(), pulse_metadata) }
    }

    #[wasm_bindgen]
    pub async fn prepare(
        &self,
        chain_id: String,
        from: String,
        call: Call,
        accounts: Vec<String>,
        use_lifi: bool,
    ) -> Result<PrepareResponse, JsError> {
        self.inner
            .prepare(chain_id, from.parse()?, call, accounts, use_lifi)
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
        accounts: Vec<String>,
        local_currency: Currency,
        use_lifi: bool,
    ) -> Result<PrepareDetailedResponse, JsError> {
        self.inner
            .prepare_detailed(
                chain_id,
                from.parse()?,
                call,
                accounts,
                local_currency,
                use_lifi,
            )
            .await
            .map_err(Into::into)
    }

    #[wasm_bindgen]
    pub async fn status(
        &self,
        orchestration_id: String,
    ) -> Result<StatusResponse, StatusError> {
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

    #[wasm_bindgen]
    pub async fn execute(
        &self,
        ui_fields: UiFields,
        route_txn_sigs: Vec<RouteSig>,
        initial_txn_sig: String,
    ) -> Result<ExecuteDetails, JsError> {
        self.inner
            .execute(ui_fields, route_txn_sigs, initial_txn_sig.parse()?)
            .await
            .map_err(Into::into)
    }

    pub async fn erc20_token_balance(
        &self,
        chain_id: &str,
        token: String,
        owner: String,
    ) -> Result<String, JsError> {
        self.inner
            .erc20_token_balance(chain_id, token.parse()?, owner.parse()?)
            .await
            .map_err(Into::into)
            .map(|balance| balance.to_string())
    }
}
