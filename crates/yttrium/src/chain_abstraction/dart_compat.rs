#[cfg(feature = "chain_abstraction_client")]
use {
    super::{
        dart_compat_models::{
            CallCompat, Eip1559EstimationCompat, ErrorCompat,
            ExecuteDetailsCompat, PrepareDetailedResponseCompat,
            PrimitiveSignatureCompat, PulseMetadataCompat, UiFieldsCompat,
        },
        ui_fields::UiFields,
    },
    crate::{
        call::Call,
        chain_abstraction::{
            // api::status::{StatusResponse, StatusResponseCompleted},
            client::Client,
            currency::Currency,
            pulse::PulseMetadata,
        },
    },
    alloy::{
        primitives::PrimitiveSignature,
        // signers::Signature,
        providers::{
            // utils::Eip1559Estimation,
            Provider,
        },
    },
    relay_rpc::domain::ProjectId,
    std::str::FromStr,
    // std::time::Duration,
};

// -----------------

#[cfg(feature = "chain_abstraction_client")]
pub struct ChainAbstractionClient {
    // project_id: String,
    client: Client,
}

#[cfg(feature = "chain_abstraction_client")]
impl ChainAbstractionClient {
    pub fn new(
        project_id: String,
        pulse_metadata: PulseMetadataCompat,
    ) -> Self {
        let pulse_metadata_orig =
            PulseMetadata::try_from(pulse_metadata).unwrap();
        let project_id = ProjectId::from(project_id.clone());
        let client = Client::new(project_id, pulse_metadata_orig);
        Self { client }
    }

    // pub async fn prepare(
    //     &self,
    //     chain_id: String,
    //     from: String,
    //     call: FFICall,
    // ) -> Result<PrepareResponse, ErrorCompat> {
    //     type Address = alloy::primitives::Address;
    //     let ffi_from = Address::from_str(&from).unwrap();
    //     let ffi_call = CallCompat::try_from(call)?;

    //     self.client
    //         .prepare(chain_id, ffi_from, ffi_call)
    //         .await
    //         .map_err(|e| ErrorCompat::General(e.to_string()))
    // }

    // FIXME
    // pub async fn get_ui_fields(
    //     &self,
    //     route_response: PrepareResponseAvailable,
    //     currency: Currency,
    // ) -> Result<UiFieldsCompat, ErrorCompat> {

    //     self.client
    //         .get_ui_fields(route_response, currency)
    //         .await
    //         .map(Into::into)
    //         .map_err(|e| ErrorCompat::General(e.to_string()))
    // }

    pub async fn prepare_detailed(
        &self,
        chain_id: String,
        from: String,
        call: CallCompat,
        local_currency: Currency,
    ) -> Result<PrepareDetailedResponseCompat, ErrorCompat> {
        type Address = alloy::primitives::Address;
        let ffi_from = Address::from_str(&from).unwrap();
        let call_orig = Call::try_from(call).unwrap();

        self.client
            .prepare_detailed(chain_id, ffi_from, call_orig, local_currency)
            .await
            .map(Into::into)
            .map_err(|e| ErrorCompat::General(e.to_string()))
    }

    // pub async fn status(
    //     &self,
    //     orchestration_id: String,
    // ) -> Result<StatusResponse, ErrorCompat> {
    //     self.client
    //         .status(orchestration_id)
    //         .await
    //         .map_err(|e| ErrorCompat::General(e.to_string()))
    // }

    // pub async fn wait_for_success_with_timeout(
    //     &self,
    //     orchestration_id: String,
    //     check_in: u64,
    //     timeout: u64,
    // ) -> Result<StatusResponseCompleted, ErrorCompat> {
    //     self.client
    //         .wait_for_success_with_timeout(
    //             orchestration_id,
    //             Duration::from_secs(check_in),
    //             Duration::from_secs(timeout),
    //         )
    //         .await
    //         .map_err(|e| ErrorCompat::General(e.to_string()))
    // }

    pub async fn estimate_fees(
        &self,
        chain_id: String,
    ) -> Result<Eip1559EstimationCompat, ErrorCompat> {
        self.client
            .provider_pool
            .get_provider(&chain_id)
            .await
            .estimate_eip1559_fees(None)
            .await
            .map(Into::into)
            .map_err(|e| ErrorCompat::General(e.to_string()))
    }

    // pub fn prepare_erc20_transfer_call(
    //     &self,
    //     erc20_address: String,
    //     to: String,
    //     amount: u128,
    // ) -> FFICall {
    //     type Address = alloy::primitives::Address;
    //     type U256 = alloy::primitives::U256;
    //     // let ffi_erc20_address = Address::from_str(&erc20_address).unwrap_or_else(|_| Address::ZERO);
    //     let ffi_erc20_address = Address::from_str(&erc20_address).unwrap();
    //     // let ffi_to = Address::from_str(&to).unwrap_or_else(|_| Address::ZERO);
    //     let ffi_to = Address::from_str(&to).unwrap();
    //     let ffi_amount = U256::from(amount);

    //     let encoded_data = transferCall::new((ffi_to, ffi_amount)).abi_encode().into();

    //     CallCompat {
    //         to: ffi_erc20_address,
    //         value: U256::ZERO,
    //         input: encoded_data,
    //     }.into()
    // }

    pub async fn erc20_token_balance(
        &self,
        chain_id: &str,
        token: String,
        owner: String,
    ) -> Result<String, ErrorCompat> {
        type Address = alloy::primitives::Address;
        // let ffi_token = Address::try_from(token).map_err(|e| ErrorCompat::General(e.to_string()))?;
        let ffi_token = Address::from_str(&token).unwrap();
        // let ffi_owner = Address::try_from(owner).map_err(|e| ErrorCompat::General(e.to_string()))?;
        let ffi_owner = Address::from_str(&owner).unwrap();

        self.client
            .erc20_token_balance(chain_id, ffi_token, ffi_owner)
            .await
            .map(|balance| balance.to_string())
            .map_err(|e| ErrorCompat::General(e.to_string()))
    }

    // FIXME
    pub async fn execute(
        &self,
        ui_fields: UiFieldsCompat,
        route_txn_sigs: Vec<PrimitiveSignatureCompat>,
        initial_txn_sig: PrimitiveSignatureCompat,
    ) -> Result<ExecuteDetailsCompat, ErrorCompat> {
        let ui_fields_orig = UiFields::try_from(ui_fields).unwrap();
        let route_txn_sigs_orig = route_txn_sigs
            .into_iter()
            .map(|sig| PrimitiveSignature::from(sig))
            .collect();

        let ffi_initial_txn_sig_orig =
            PrimitiveSignature::from(initial_txn_sig);

        self.client
            .execute(
                ui_fields_orig,
                route_txn_sigs_orig,
                ffi_initial_txn_sig_orig,
            )
            .await
            .map(Into::into)
            // TODO wanted to return ExecuteError directly here, but can't because Swift keeps the UniFFI lifer private to the yttrium crate and not available to kotlin-ffi crate
            // This will be fixed when we merge these crates
            .map_err(|e| ErrorCompat::General(e.to_string()))
    }
}
