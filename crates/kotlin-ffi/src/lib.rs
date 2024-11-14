uniffi::setup_scaffolding!();

use yttrium::chain_abstraction::api::route::RouteResponse;
use yttrium::chain_abstraction::api::status::StatusResponse;
use yttrium::chain_abstraction::api::Transaction as CATransaction;

use relay_rpc::domain::ProjectId;
use yttrium::chain_abstraction::client::Client;
use yttrium::config::Config;
use yttrium::transaction::send::safe_test::{
    Address, OwnerSignature as YOwnerSignature, PrimitiveSignature,
};
use yttrium::{
    account_client::{AccountClient as YAccountClient, SignerType},
    private_key_service::PrivateKeyService,
    sign_service::address_from_string,
    transaction::Transaction as YTransaction,
};

#[derive(uniffi::Record)]
pub struct AccountClientConfig {
    pub owner_address: String,
    pub chain_id: u64,
    pub config: Config,
    pub signer_type: String,
    pub safe: bool,
    pub private_key: String,
}

#[derive(uniffi::Record)]
pub struct Transaction {
    pub to: String,
    pub value: String,
    pub data: String,
}

#[derive(uniffi::Record)]
pub struct InitTransaction {
    pub from: String,
    pub to: String,
    pub value: String,
    pub gas: String,
    pub gas_price: String,
    pub data: String,
    pub nonce: String,
    pub max_fee_per_gas: String,
    pub max_priority_fee_per_gas: String,
    pub chain_id: String,
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
pub enum Error {
    #[error("General {0}")]
    General(String),
}

#[derive(uniffi::Object)]
pub struct AccountClient {
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

    pub async fn route(
        &self,
        transaction: InitTransaction,
    ) -> Result<RouteResponse, Error> {
        let ca_transaction = CATransaction::from(transaction);
        self.client
            .route(ca_transaction)
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
}

#[uniffi::export(async_runtime = "tokio")]
impl AccountClient {
    #[uniffi::constructor]
    pub fn new(config: AccountClientConfig) -> Self {
        let owner_address = config.owner_address.clone();
        let signer_type = config.signer_type.clone();
        let signer = SignerType::from(signer_type).unwrap();
        let account_client = match signer {
            SignerType::PrivateKey => {
                let private_key_fn =
                    Box::new(move || Ok(config.private_key.clone()));
                let owner = address_from_string(&owner_address).unwrap();
                let service = PrivateKeyService::new(private_key_fn, owner);
                YAccountClient::new_with_private_key_service(
                    config.owner_address.clone(),
                    config.chain_id,
                    config.config,
                    service,
                    config.safe,
                )
            }
            SignerType::Native => todo!(),
        };

        Self {
            owner_address: config.owner_address.clone(),
            chain_id: config.chain_id,
            account_client,
        }
    }

    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }

    pub async fn get_address(&self) -> Result<String, Error> {
        self.account_client
            .get_address()
            .await
            .map_err(|e| Error::General(e.to_string()))
    }

    pub async fn send_transactions(
        &self,
        transactions: Vec<Transaction>,
    ) -> Result<String, Error> {
        let ytransactions: Vec<YTransaction> =
            transactions.into_iter().map(YTransaction::from).collect();

        Ok(self
            .account_client
            .send_transactions(ytransactions)
            .await
            .map_err(|e| Error::General(e.to_string()))?
            .to_string())
    }

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

    pub fn sign_message_with_mnemonic(
        &self,
        message: String,
        mnemonic: String,
    ) -> Result<String, Error> {
        self.account_client
            .sign_message_with_mnemonic(message, mnemonic)
            .map_err(|e| Error::General(e.to_string()))
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

impl From<InitTransaction> for CATransaction {
    fn from(source: InitTransaction) -> Self {
        CATransaction {
            from: source.from,
            to: source.to,
            value: source.value,
            gas: source.gas,
            gas_price: source.gas_price,
            data: source.data,
            nonce: source.nonce,
            max_fee_per_gas: source.max_fee_per_gas,
            max_priority_fee_per_gas: source.max_priority_fee_per_gas,
            chain_id: source.chain_id,
        }
    }
}
