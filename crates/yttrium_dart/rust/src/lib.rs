mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */
use {
    alloy::{
        network::Ethereum,
        primitives::{Address, PrimitiveSignature},
        providers::{Provider, ReqwestProvider},
    },
    relay_rpc::domain::ProjectId,
    serde::{Deserialize, Serialize},
    std::time::Duration,
    yttrium::{
        account_client::AccountClient as YAccountClient,
        call::{send::safe_test::OwnerSignature as YOwnerSignature, Call},
        chain_abstraction::{
            api::{
                prepare::PrepareResponse,
                status::{StatusResponse, StatusResponseCompleted},
            },
            client::Client,
        },
        config::Config,
    },
};

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

#[derive(Clone, Debug)]
pub struct Eip1559Estimation {
    pub max_fee_per_gas: String,
    pub max_priority_fee_per_gas: String,
}

pub struct ChainAbstractionClient {
    pub project_id: String,
    client: Client,
}

impl ChainAbstractionClient {
    // #[frb(constructor)]
    pub fn new(project_id: String) -> Self {
        let client = Client::new(ProjectId::from(project_id.clone()));
        Self { project_id, client }
    }

    pub async fn route(
        &self,
        chain_id: String,
        from: Address,
        call: Call,
    ) -> Result<PrepareResponse, Error> {
        self.client
            .prepare(chain_id, from, call)
            .await
            .map_err(|e| Error::General(e.to_string()))
    }

    pub async fn status(
        &self,
        orchestration_id: String,
    ) -> Result<StatusResponse, Error> {
        self.client
            .status(orchestration_id)
            .await
            .map_err(|e| Error::General(e.to_string()))
    }

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
        // Ensure async execution
        provider
            .estimate_eip1559_fees(None)
            .await
            .map_err(|e| Error::General(e.to_string()))
            .map(|fees| Eip1559Estimation {
                max_fee_per_gas: fees.max_fee_per_gas.to_string(),
                max_priority_fee_per_gas: fees
                    .max_priority_fee_per_gas
                    .to_string(),
            })
    }
}

pub struct AccountClientConfig {
    pub owner_address: String,
    pub chain_id: u64,
    pub config: Config,
}

pub struct AccountClient {
    pub owner_address: String,
    pub chain_id: u64,
    account_client: YAccountClient,
}

impl AccountClient {
    // #[frb(constructor)]
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
    pub async fn get_address(&self) -> Result<String, Error> {
        self.account_client
            .get_address()
            .await
            .map(|address| address.to_string())
            .map_err(|e| Error::General(e.to_string()))
    }

    pub async fn prepare_send_transactions(
        &self,
        transactions: Vec<Call>,
    ) -> Result<PreparedSendTransaction, Error> {
        let ytransactions: Vec<Call> =
            transactions.into_iter().map(Call::from).collect();

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
