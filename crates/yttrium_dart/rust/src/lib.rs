mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */
use {
    alloy::{
        network::Ethereum,
        primitives::{U128 as FFIU128, U256 as FFIU256},
        providers::{Provider, ReqwestProvider},
    },
    flutter_rust_bridge::frb,
    relay_rpc::domain::ProjectId,
    serde::{Deserialize, Serialize},
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
                Address, OwnerSignature as YOwnerSignature, PrimitiveSignature,
            },
            Transaction as YTransaction,
        },
    },
};

#[frb]
pub struct AccountClientConfig {
    pub owner_address: String,
    pub chain_id: u64,
    pub config: Config,
}

#[frb]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub to: String,
    pub value: String,
    pub data: String,
}

// uniffi::custom_type!(FFIAddress, String, {
//     try_lift: |val| Ok(val.parse()?),
//     lower: |obj| obj.to_string(),
// });

// fn uint_to_hex<const BITS: usize, const LIMBS: usize>(
//     obj: Uint<BITS, LIMBS>,
// ) -> String {
//     format!("0x{obj:x}")
// }

// uniffi::custom_type!(FFIU64, String, {
//     try_lift: |val| Ok(val.parse()?),
//     lower: |obj| uint_to_hex(obj),
// });

// uniffi::custom_type!(FFIU128, String, {
//     try_lift: |val| Ok(val.parse()?),
//     lower: |obj| uint_to_hex(obj),
// });

// uniffi::custom_type!(FFIU256, String, {
//     try_lift: |val| Ok(val.parse()?),
//     lower: |obj| uint_to_hex(obj),
// });

// uniffi::custom_type!(FFIBytes, String, {
//     try_lift: |val| Ok(val.parse()?),
//     lower: |obj| obj.to_string(),
// });

#[frb]
#[derive(Clone, Debug)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreparedSendTransaction {
    pub hash: String,
    pub do_send_transaction_params: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OwnerSignature {
    pub owner: String,
    pub signature: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("General {0}")]
    General(String),
}

#[frb]
pub struct AccountClient {
    pub owner_address: String,
    pub chain_id: u64,
    account_client: YAccountClient,
}

#[frb]
pub struct ChainAbstractionClient {
    pub project_id: String,
    client: Client,
}

#[frb]
impl ChainAbstractionClient {
    // #[uniffi::constructor]
    pub fn new(project_id: String) -> Self {
        let client = Client::new(ProjectId::from(project_id.clone()));
        Self { project_id, client }
    }

    #[frb]
    pub async fn prepare(
        &self,
        initial_transaction: InitialTransaction,
    ) -> Result<PrepareResponse, Error> {
        self.client
            .prepare(initial_transaction)
            .await
            .map_err(|e| Error::General(e.to_string()))
    }

    #[frb]
    pub async fn get_ui_fields(
        &self,
        route_response: RouteResponseAvailable,
        currency: Currency,
    ) -> Result<UiFields, Error> {
        self.client
            .get_ui_fields(route_response, currency)
            .await
            .map(Into::into)
            .map_err(|e| Error::General(e.to_string()))
    }

    #[frb]
    pub async fn status(
        &self,
        orchestration_id: String,
    ) -> Result<StatusResponse, Error> {
        self.client
            .status(orchestration_id)
            .await
            .map_err(|e| Error::General(e.to_string()))
    }

    #[frb]
    pub async fn wait_for_success_with_timeout(
        &self,
        orchestration_id: String,
        check_in: u64,
        timeout: u64,
    ) -> Result<StatusResponseCompleted, Error> {
        self.client
            .wait_for_success_with_timeout(
                orchestration_id,
                Duration::from_secs(check_in),
                Duration::from_secs(timeout),
            )
            .await
            .map_err(|e| Error::General(e.to_string()))
    }

    #[frb]
    pub async fn estimate_fees(
        &self,
        chain_id: String,
    ) -> Result<Eip1559Estimation, Error> {
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
            .map_err(|e| Error::General(e.to_string()))
    }

    #[frb]
    pub async fn erc20_token_balance(
        &self,
        chain_id: String,
        token: Address,
        owner: Address,
    ) -> Result<FFIU256, Error> {
        self.client
            .erc20_token_balance(chain_id, token, owner)
            .await
            .map_err(|e| Error::General(e.to_string()))
    }
}

#[frb]
impl AccountClient {
    // #[uniffi::constructor]
    pub fn new(config: AccountClientConfig) -> Self {
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

    pub fn get_chain_id(&self) -> u64 {
        self.chain_id
    }

    // Async method for fetching address
    #[frb]
    pub async fn get_address(&self) -> Result<String, Error> {
        self.account_client
            .get_address()
            .await
            .map(|address| address.to_string())
            .map_err(|e| Error::General(e.to_string()))
    }

    #[frb]
    pub async fn prepare_send_transactions(
        &self,
        transactions: Vec<Transaction>,
    ) -> Result<PreparedSendTransaction, Error> {
        let ytransactions: Vec<YTransaction> =
            transactions.into_iter().map(YTransaction::from).collect();

        let prepared_send_transaction = self
            .account_client
            .prepare_send_transactions(ytransactions)
            .await
            .map_err(|e| Error::General(e.to_string()))?;

        Ok(PreparedSendTransaction {
            hash: prepared_send_transaction.hash.to_string(),
            do_send_transaction_params: serde_json::to_string(
                &prepared_send_transaction.do_send_transaction_params,
            )
            .map_err(|e| Error::General(e.to_string()))?,
        })
    }

    #[frb]
    pub async fn do_send_transactions(
        &self,
        signatures: Vec<OwnerSignature>,
        do_send_transaction_params: String,
    ) -> Result<String, Error> {
        let mut signatures2: Vec<YOwnerSignature> =
            Vec::with_capacity(signatures.len());

        for signature in signatures {
            signatures2.push(YOwnerSignature {
                owner: signature
                    .owner
                    .parse::<Address>()
                    .map_err(|e| Error::General(e.to_string()))?,
                signature: signature
                    .signature
                    .parse::<PrimitiveSignature>()
                    .map_err(|e| Error::General(e.to_string()))?,
            });
        }

        Ok(self
            .account_client
            .do_send_transactions(
                signatures2,
                serde_json::from_str(&do_send_transaction_params)
                    .map_err(|e| Error::General(e.to_string()))?,
            )
            .await
            .map_err(|e| Error::General(e.to_string()))?
            .to_string())
    }

    #[frb]
    pub async fn wait_for_user_operation_receipt(
        &self,
        user_operation_hash: String,
    ) -> Result<String, Error> {
        self.account_client
            .wait_for_user_operation_receipt(
                user_operation_hash.parse().map_err(|e| {
                    Error::General(format!("Parsing user_operation_hash: {e}"))
                })?,
            )
            .await
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<String, serde_json::Error>>()
            .map_err(|e| Error::General(e.to_string()))
    }
}

#[frb]
impl From<Transaction> for YTransaction {
    fn from(transaction: Transaction) -> Self {
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
    use alloy::{
        network::Ethereum,
        providers::{Provider, ReqwestProvider},
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
}
