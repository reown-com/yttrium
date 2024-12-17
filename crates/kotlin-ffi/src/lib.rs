uniffi::setup_scaffolding!();

use {
    alloy::{
        network::Ethereum,
        primitives::{
            Bytes as FFIBytes, Uint, U128 as FFIU128, U256 as FFIU256,
            U64 as FFIU64,
        },
        providers::{Provider, ReqwestProvider},
    },
    relay_rpc::domain::ProjectId,
    std::time::Duration,
    yttrium::{
        account_client::AccountClient as YAccountClient,
        chain_abstraction::{
            api::{
                prepare::{PrepareResponse, RouteResponseAvailable},
                status::{StatusResponse, StatusResponseCompleted},
                InitialTransaction,
            },
            client::Client,
            currency::Currency,
            ui_fields::UiFields,
        },
        config::Config,
        transaction::{
            send::safe_test::{
                Address as FFIAddress, OwnerSignature as YOwnerSignature,
                PrimitiveSignature,
            },
            Transaction as YTransaction,
        },
    },
};

#[derive(uniffi::Record)]
pub struct FFIAccountClientConfig {
    pub owner_address: String,
    pub chain_id: u64,
    pub config: Config,
}

#[derive(uniffi::Record)]
pub struct FFITransaction {
    pub to: String,
    pub value: String,
    pub data: String,
}

uniffi::custom_type!(FFIAddress, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

fn uint_to_hex<const BITS: usize, const LIMBS: usize>(
    obj: Uint<BITS, LIMBS>,
) -> String {
    format!("0x{obj:x}")
}

uniffi::custom_type!(FFIU64, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

uniffi::custom_type!(FFIU128, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

uniffi::custom_type!(FFIU256, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

uniffi::custom_type!(FFIBytes, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Record)]
pub struct Eip1559Estimation {
    /// The base fee per gas.
    pub max_fee_per_gas: FFIU128,
    /// The max priority fee per gas.
    pub max_priority_fee_per_gas: FFIU128,
}

impl From<alloy::providers::utils::Eip1559Estimation> for Eip1559Estimation {
    fn from(source: alloy::providers::utils::Eip1559Estimation) -> Self {
        Self {
            max_fee_per_gas: FFIU128::from(source.max_fee_per_gas),
            max_priority_fee_per_gas: FFIU128::from(
                source.max_priority_fee_per_gas,
            ),
        }
    }
}

#[derive(uniffi::Record)]
pub struct PreparedSendTransaction {
    pub hash: String,
    pub do_send_transaction_params: String,
}

#[derive(uniffi::Record)]
pub struct OwnerSignature {
    pub owner: String,
    pub signature: String,
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum FFIError {
    #[error("General {0}")]
    General(String),
}

#[derive(uniffi::Object)]
pub struct FFIAccountClient {
    pub owner_address: String,
    pub chain_id: u64,
    account_client: YAccountClient,
}

#[derive(uniffi::Object)]
pub struct ChainAbstractionClient {
    pub project_id: String,
    client: Client,
}

#[uniffi::export(async_runtime = "tokio")]
impl ChainAbstractionClient {
    #[uniffi::constructor]
    pub fn new(project_id: String) -> Self {
        let client = Client::new(ProjectId::from(project_id.clone()));
        Self { project_id, client }
    }

    pub async fn prepare(
        &self,
        initial_transaction: InitialTransaction,
    ) -> Result<PrepareResponse, FFIError> {
        self.client
            .prepare(initial_transaction)
            .await
            .map_err(|e| FFIError::General(e.to_string()))
    }

    pub async fn get_ui_fields(
        &self,
        route_response: RouteResponseAvailable,
        currency: Currency,
    ) -> Result<UiFields, FFIError> {
        self.client
            .get_ui_fields(route_response, currency)
            .await
            .map(Into::into)
            .map_err(|e| FFIError::General(e.to_string()))
    }

    pub async fn status(
        &self,
        orchestration_id: String,
    ) -> Result<StatusResponse, FFIError> {
        self.client
            .status(orchestration_id)
            .await
            .map_err(|e| FFIError::General(e.to_string()))
    }

    pub async fn wait_for_success_with_timeout(
        &self,
        orchestration_id: String,
        check_in: u64,
        timeout: u64,
    ) -> Result<StatusResponseCompleted, FFIError> {
        self.client
            .wait_for_success_with_timeout(
                orchestration_id,
                Duration::from_secs(check_in),
                Duration::from_secs(timeout),
            )
            .await
            .map_err(|e| FFIError::General(e.to_string()))
    }

    pub async fn estimate_fees(
        &self,
        chain_id: String,
    ) -> Result<Eip1559Estimation, FFIError> {
        let url = format!(
            "https://rpc.walletconnect.com/v1?chainId={chain_id}&projectId={}",
            self.project_id
        )
        .parse()
        .expect("Invalid RPC URL");
        let provider = ReqwestProvider::<Ethereum>::new_http(url);
        provider
            .estimate_eip1559_fees(None)
            .await
            .map(Into::into)
            .map_err(|e| FFIError::General(e.to_string()))
    }

    pub async fn erc20_token_balance(
        &self,
        chain_id: String,
        token: FFIAddress,
        owner: FFIAddress,
    ) -> Result<FFIU256, FFIError> {
        self.client
            .erc20_token_balance(chain_id, token, owner)
            .await
            .map_err(|e| FFIError::General(e.to_string()))
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl FFIAccountClient {
    #[uniffi::constructor]
    pub fn new(config: FFIAccountClientConfig) -> Self {
        let account_client = YAccountClient::new(
            config
                .owner_address
                .parse::<alloy::primitives::Address>()
                .unwrap()
                .into(),
            config.chain_id,
            config.config,
        );

        Self {
            owner_address: config.owner_address.clone(),
            chain_id: config.chain_id,
            account_client,
        }
    }

    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }

    pub async fn get_address(&self) -> Result<String, FFIError> {
        self.account_client
            .get_address()
            .await
            .map(|address| address.to_string())
            .map_err(|e| FFIError::General(e.to_string()))
    }

    pub async fn prepare_send_transactions(
        &self,
        transactions: Vec<FFITransaction>,
    ) -> Result<PreparedSendTransaction, FFIError> {
        let ytransactions: Vec<YTransaction> =
            transactions.into_iter().map(YTransaction::from).collect();

        let prepared_send_transaction = self
            .account_client
            .prepare_send_transactions(ytransactions)
            .await
            .map_err(|e| FFIError::General(e.to_string()))?;

        Ok(PreparedSendTransaction {
            hash: prepared_send_transaction.hash.to_string(),
            do_send_transaction_params: serde_json::to_string(
                &prepared_send_transaction.do_send_transaction_params,
            )
            .map_err(|e| FFIError::General(e.to_string()))?,
        })
    }

    pub async fn do_send_transactions(
        &self,
        signatures: Vec<OwnerSignature>,
        do_send_transaction_params: String,
    ) -> Result<String, FFIError> {
        let mut signatures2: Vec<YOwnerSignature> =
            Vec::with_capacity(signatures.len());

        for signature in signatures {
            signatures2.push(YOwnerSignature {
                owner: signature
                    .owner
                    .parse::<FFIAddress>()
                    .map_err(|e| FFIError::General(e.to_string()))?,
                signature: signature
                    .signature
                    .parse::<PrimitiveSignature>()
                    .map_err(|e| FFIError::General(e.to_string()))?,
            });
        }

        Ok(self
            .account_client
            .do_send_transactions(
                signatures2,
                serde_json::from_str(&do_send_transaction_params)
                    .map_err(|e| FFIError::General(e.to_string()))?,
            )
            .await
            .map_err(|e| FFIError::General(e.to_string()))?
            .to_string())
    }

    pub async fn wait_for_user_operation_receipt(
        &self,
        user_operation_hash: String,
    ) -> Result<String, FFIError> {
        self.account_client
            .wait_for_user_operation_receipt(
                user_operation_hash.parse().map_err(|e| {
                    FFIError::General(format!(
                        "Parsing user_operation_hash: {e}"
                    ))
                })?,
            )
            .await
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<String, serde_json::Error>>()
            .map_err(|e| FFIError::General(e.to_string()))
    }
}

impl From<FFITransaction> for YTransaction {
    fn from(transaction: FFITransaction) -> Self {
        YTransaction::new_from_strings(
            transaction.to,
            transaction.value,
            transaction.data,
        )
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        alloy::{
            network::Ethereum,
            primitives::{address, bytes},
            providers::{Provider, ReqwestProvider},
        },
    };

    #[tokio::test]
    #[ignore = "run manually"]
    async fn estimate_fees() {
        let chain_id = "eip155:42161";
        let project_id = std::env::var("REOWN_PROJECT_ID").unwrap();
        let url = format!(
            "https://rpc.walletconnect.com/v1?chainId={chain_id}&projectId={project_id}")
        .parse()
        .expect("Invalid RPC URL");
        let provider = ReqwestProvider::<Ethereum>::new_http(url);

        let estimate = provider.estimate_eip1559_fees(None).await.unwrap();

        println!("estimate: {estimate:?}");
    }

    #[test]
    fn test_address_lower() {
        let ffi_u64 = address!("abababababababababababababababababababab");
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(ffi_u64);
        let s: String =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(s, format!("0xABaBaBaBABabABabAbAbABAbABabababaBaBABaB"));
    }

    #[test]
    fn test_u64_lower() {
        let num = 1234567890;
        let ffi_u64 = FFIU64::from(num);
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(ffi_u64);
        let s: String =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(s, format!("0x{num:x}"));
    }

    #[test]
    fn test_u128_lower() {
        let num = 1234567890;
        let ffi_u64 = FFIU128::from(num);
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(ffi_u64);
        let s: String =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(s, format!("0x{num:x}"));
    }

    #[test]
    fn test_u256_lower() {
        let num = 1234567890;
        let ffi_u64 = FFIU256::from(num);
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(ffi_u64);
        let s: String =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(s, format!("0x{num:x}"));
    }

    #[test]
    fn test_bytes_lower() {
        let ffi_u64 = bytes!("aabbccdd");
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(ffi_u64);
        let s: String =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(s, format!("0xaabbccdd"));
    }
}
