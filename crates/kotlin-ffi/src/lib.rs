// use super::ffi;
// use super::ffi::{FFIAccountClientConfig, FFIError};
use yttrium::config::Config;
use yttrium::config::Endpoint;
use yttrium::config::Endpoints;
use yttrium::{
    account_client::{AccountClient as YAccountClient, SignerType},
    private_key_service::PrivateKeyService,
    sign_service::address_from_string,
    transaction::Transaction as YTransaction,
};

pub struct AccountClient {
    pub owner_address: String,
    pub chain_id: u64,
    account_client: YAccountClient,
}

pub struct AccountClientConfig {
    pub owner_address: String,
    pub chain_id: u64,
    pub config: Config,
    pub signer_type: String,
    pub safe: bool,
    pub private_key: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unknown {0}")]
    Unknown(String),
}

impl AccountClient {
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
                    config.config.into(),
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
            .map_err(|e| Error::Unknown(e.to_string()))
    }

    pub async fn send_transaction(
        &self,
        transaction: Transaction,
    ) -> Result<String, Error> {
        let transaction = YTransaction::from(transaction);
        self.account_client
            .send_transaction(transaction)
            .await
            .map_err(|e| Error::Unknown(e.to_string()))
    }

    pub fn sign_message_with_mnemonic(
        &self,
        message: String,
        mnemonic: String,
    ) -> Result<String, Error> {
        self.account_client
            .sign_message_with_mnemonic(message, mnemonic)
            .map_err(|e| Error::Unknown(e.to_string()))
    }

    pub async fn wait_for_user_operation_receipt(
        &self,
        user_operation_hash: String,
    ) -> Result<String, Error> {
        self.account_client
            .wait_for_user_operation_receipt(user_operation_hash)
            .await
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<String, serde_json::Error>>()
            .map_err(|e| Error::Unknown(e.to_string()))
    }
}

pub struct Transaction {
    pub to: String,
    pub value: String,
    pub data: String,
}

uniffi::include_scaffolding!("yttrium");

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
