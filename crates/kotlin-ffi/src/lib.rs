uniffi::setup_scaffolding!();

use alloy::{
    network::Ethereum,
    primitives::{Address as YAddress, Bytes, B256},
    providers::ReqwestProvider,
};
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

#[derive(uniffi::Object)]
pub struct AccountClient {
    pub owner_address: String,
    pub chain_id: u64,
    account_client: YAccountClient,
}

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
    #[error("Unknown {0}")]
    Unknown(String),
}

#[derive(uniffi::Object)]
pub struct Erc6492Client {
    provider: ReqwestProvider<Ethereum>,
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum Erc6492Error {
    #[error("InvalidSignature")]
    InvalidSignature(String),
    #[error("InvalidAddress")]
    InvalidAddress(String),
    #[error("InvalidMessageHash")]
    InvalidMessageHash(String),
    #[error("Verification")]
    Verification(String),
}

#[uniffi::export(async_runtime = "tokio")]
impl Erc6492Client {
    #[uniffi::constructor]
    pub fn new(rpc_url: String) -> Self {
        let url = rpc_url.parse().expect("Invalid RPC URL");
        let provider = ReqwestProvider::<Ethereum>::new_http(url);
        Self { provider }
    }

    pub async fn verify_signature(
        &self,
        signature: String,
        address: String,
        message_hash: String,
    ) -> Result<bool, Erc6492Error> {
        let signature = signature
            .parse::<Bytes>()
            .map_err(|e| Erc6492Error::InvalidSignature(e.to_string()))?;
        let address = address
            .parse::<YAddress>()
            .map_err(|e| Erc6492Error::InvalidAddress(e.to_string()))?;
        let message_hash = message_hash
            .parse::<B256>()
            .map_err(|e| Erc6492Error::InvalidMessageHash(e.to_string()))?;

        let verification = erc6492::verify_signature(
            signature,
            address,
            message_hash,
            &self.provider,
        )
        .await
        .map_err(|e| Erc6492Error::Verification(e.to_string()))?;

        Ok(verification.is_valid())
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
            .map_err(|e| Error::Unknown(e.to_string()))
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
            .map_err(|e| Error::Unknown(e.to_string()))?
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
            .map_err(|e| Error::Unknown(e.to_string()))?;

        Ok(PreparedSendTransaction {
            hash: prepared_send_transaction.hash.to_string(),
            do_send_transaction_params: serde_json::to_string(
                &prepared_send_transaction.do_send_transaction_params,
            )
            .map_err(|e| Error::Unknown(e.to_string()))?,
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
                    .map_err(|e| Error::Unknown(e.to_string()))?,
                signature: signature
                    .signature
                    .parse::<PrimitiveSignature>()
                    .map_err(|e| Error::Unknown(e.to_string()))?,
            });
        }

        Ok(self
            .account_client
            .do_send_transactions(
                signatures2,
                serde_json::from_str(&do_send_transaction_params)
                    .map_err(|e| Error::Unknown(e.to_string()))?,
            )
            .await
            .map_err(|e| Error::Unknown(e.to_string()))?
            .to_string())
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
            .wait_for_user_operation_receipt(
                user_operation_hash.parse().map_err(|e| {
                    Error::Unknown(format!("Parsing user_operation_hash: {e}"))
                })?,
            )
            .await
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<String, serde_json::Error>>()
            .map_err(|e| Error::Unknown(e.to_string()))
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
