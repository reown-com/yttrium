use super::ffi;
use super::ffi::{FFIAccountClientConfig, FFIError};
use crate::ffi::{
    FFIOwnerSignature, FFIPreparedSendTransaction, FFIPreparedSignature,
};
use alloy::primitives::B256;
use alloy::sol_types::SolStruct;
use std::str::FromStr;
use yttrium::transaction::send::safe_test::{
    Address, OwnerSignature, Signature,
};
use yttrium::{
    account_client::{AccountClient, SignerType},
    error::YttriumError,
    private_key_service::PrivateKeyService,
    sign_service::{address_from_string, SignService},
    transaction::Transaction,
};

pub struct FFIAccountClient {
    pub owner_address: String,
    pub chain_id: u64,
    account_client: AccountClient,
}

impl FFIAccountClient {
    pub fn new(config: FFIAccountClientConfig) -> Self {
        #[cfg(target_os = "ios")]
        match crate::log::init_os_logger() {
            Ok(_) => {
                log::debug!("log::debug! Logging setup successfully");
            }
            Err(err) => {
                println!("Logging setup failure e: {:?}", err.to_string());
            }
        }

        let owner_address = config.owner_address.clone();
        let chain_id = config.chain_id;
        let signer_type = config.signer_type.clone();
        let signer_id =
            format!("{}-{}-{}", signer_type, owner_address, chain_id);

        let signer = SignerType::from(signer_type).unwrap();

        let account_client = match signer {
            SignerType::Native => {
                let sign_fn = Box::new(move |message: String| {
                    let signer_service =
                        ffi::NativeSignerFFI::new(signer_id.clone());
                    let sign = signer_service.sign(message);
                    match sign {
                        ffi::FFIStringResult::Ok(signed_message) => {
                            Ok(signed_message)
                        }
                        ffi::FFIStringResult::Err(error) => {
                            Err(YttriumError { message: error })
                        }
                    }
                });
                let owner = address_from_string(&owner_address).unwrap();
                let signer = SignService::new(sign_fn, owner);
                AccountClient::new_with_sign_service(
                    config.owner_address.clone(),
                    config.chain_id,
                    config.config.into(),
                    signer,
                )
            }
            SignerType::PrivateKey => {
                let private_key_fn = Box::new(move || {
                    let private_key_service =
                        ffi::PrivateKeySignerFFI::new(signer_id.clone());
                    let private_key = private_key_service.private_key();
                    match private_key {
                        ffi::FFIStringResult::Ok(private_key) => {
                            Ok(private_key)
                        }
                        ffi::FFIStringResult::Err(error) => {
                            Err(YttriumError { message: error })
                        }
                    }
                });
                let owner = address_from_string(&owner_address).unwrap();
                let service = PrivateKeyService::new(private_key_fn, owner);
                AccountClient::new_with_private_key_service(
                    config.owner_address.clone(),
                    config.chain_id,
                    config.config.into(),
                    service,
                    config.safe,
                )
            }
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

    pub async fn get_address(&self) -> Result<String, FFIError> {
        self.account_client
            .get_address()
            .await
            .map_err(|e| FFIError::Unknown(e.to_string()))
    }

    pub async fn prepare_sign_message(
        &self,
        message_hash: String,
    ) -> Result<FFIPreparedSignature, FFIError> {
        let message_hash = B256::from_str(&message_hash)
            .map_err(|e| FFIError::Unknown(e.to_string()))?;

        let prepared_signature =
            self.account_client.prepare_sign_message(message_hash);

        Ok(FFIPreparedSignature {
            hash: prepared_signature
                .safe_message
                .eip712_signing_hash(&prepared_signature.domain)
                .to_string(),
        })
    }

    pub async fn do_sign_message(
        &self,
        signatures: Vec<String>,
    ) -> Result<String, FFIError> {
        let signatures: Result<Vec<FFIOwnerSignature>, _> = signatures
            .into_iter()
            .map(|json| serde_json::from_str::<FFIOwnerSignature>(&json))
            .collect();

        let signatures = match signatures {
            Ok(sigs) => sigs,
            Err(e) => {
                return Err(FFIError::Unknown(format!(
                    "Failed to deserialize signatures: {}",
                    e
                )));
            }
        };

        let mut signatures2 = Vec::with_capacity(signatures.len());
        for signature in signatures {
            signatures2.push(OwnerSignature {
                owner: signature
                    .owner
                    .parse::<Address>()
                    .map_err(|e| FFIError::Unknown(e.to_string()))?,
                signature: signature
                    .signature
                    .parse::<Signature>()
                    .map_err(|e| FFIError::Unknown(e.to_string()))?,
            });
        }

        Ok(self.account_client.do_sign_message(signatures2).to_string())
    }

    pub async fn send_transactions(
        &self,
        transactions: Vec<String>,
    ) -> Result<String, FFIError> {
        // Map the JSON strings to Transaction objects
        let transactions: Result<Vec<Transaction>, _> = transactions
            .into_iter()
            .map(|json| serde_json::from_str::<Transaction>(&json))
            .collect();

        // Handle any errors that occurred during deserialization
        let transactions = match transactions {
            Ok(transactions) => transactions,
            Err(e) => {
                return Err(FFIError::Unknown(format!(
                    "Failed to deserialize transactions: {}",
                    e
                )));
            }
        };

        // Proceed to send transactions using account_client
        Ok(self
            .account_client
            .send_transactions(transactions)
            .await
            .map_err(|e| FFIError::Unknown(e.to_string()))?
            .to_string())
    }

    pub async fn prepare_send_transactions(
        &self,
        transactions: Vec<String>,
    ) -> Result<FFIPreparedSendTransaction, FFIError> {
        // Map the JSON strings to Transaction objects
        let transactions: Result<Vec<Transaction>, _> = transactions
            .into_iter()
            .map(|json| serde_json::from_str::<Transaction>(&json))
            .collect();

        // Handle any errors that occurred during deserialization
        let transactions = match transactions {
            Ok(transactions) => transactions,
            Err(e) => {
                return Err(FFIError::Unknown(format!(
                    "Failed to deserialize transactions: {}",
                    e
                )));
            }
        };

        // Proceed to send transactions using account_client
        let prepared_send_transaction = self
            .account_client
            .prepare_send_transactions(transactions)
            .await
            .map_err(|e| FFIError::Unknown(e.to_string()))?;
        Ok(FFIPreparedSendTransaction {
            hash: prepared_send_transaction.hash.to_string(),
            do_send_transaction_params: serde_json::to_string(
                &prepared_send_transaction.do_send_transaction_params,
            )
            .map_err(|e| FFIError::Unknown(e.to_string()))?,
        })
    }

    pub async fn do_send_transaction(
        &self,
        signatures: Vec<String>,
        do_send_transaction_params: String,
    ) -> Result<String, FFIError> {
        let signatures: Result<Vec<FFIOwnerSignature>, _> = signatures
            .into_iter()
            .map(|json| serde_json::from_str::<FFIOwnerSignature>(&json))
            .collect();

        let signatures = match signatures {
            Ok(sigs) => sigs,
            Err(e) => {
                return Err(FFIError::Unknown(format!(
                    "Failed to deserialize signatures: {}",
                    e
                )));
            }
        };

        let mut signatures2 = Vec::with_capacity(signatures.len());
        for signature in signatures {
            signatures2.push(OwnerSignature {
                owner: signature
                    .owner
                    .parse::<Address>()
                    .map_err(|e| FFIError::Unknown(e.to_string()))?,
                signature: signature
                    .signature
                    .parse::<Signature>()
                    .map_err(|e| FFIError::Unknown(e.to_string()))?,
            });
        }

        Ok(self
            .account_client
            .do_send_transactions(
                signatures2,
                serde_json::from_str(&do_send_transaction_params)
                    .map_err(|e| FFIError::Unknown(e.to_string()))?,
            )
            .await
            .map_err(|e| FFIError::Unknown(e.to_string()))?
            .to_string())
    }

    pub fn sign_message_with_mnemonic(
        &self,
        message: String,
        mnemonic: String,
    ) -> Result<String, FFIError> {
        self.account_client
            .sign_message_with_mnemonic(message, mnemonic)
            .map_err(|e| FFIError::Unknown(e.to_string()))
    }

    pub async fn wait_for_user_operation_receipt(
        &self,
        user_operation_hash: String,
    ) -> Result<String, FFIError> {
        self.account_client
            .wait_for_user_operation_receipt(
                user_operation_hash.parse().map_err(|e| {
                    FFIError::Unknown(format!(
                        "Parsing user_operation_hash: {e}"
                    ))
                })?,
            )
            .await
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<String, serde_json::Error>>()
            .map_err(|e| FFIError::Unknown(e.to_string()))
    }
}

impl From<ffi::FFITransaction> for Transaction {
    fn from(transaction: ffi::FFITransaction) -> Self {
        Transaction::new_from_strings(
            transaction._to,
            transaction._value,
            transaction._data,
        )
        .unwrap()
    }
}
