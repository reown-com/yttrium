use {
    crate::{
        blockchain_api::BLOCKCHAIN_API_URL_PROD,
        chain_abstraction::pulse::PulseMetadata,
        provider_pool::{network, ProviderPool},
    },
    clarity::Error as ClarityError,
    rand::{
        rngs::{OsRng, StdRng},
        SeedableRng,
    },
    relay_rpc::domain::ProjectId,
    reqwest::Client as ReqwestClient,
    stacks_rs::{
        clarity,
        crypto::c32::Version,
        transaction::{
            Error as StacksTransactionError, STXTokenTransfer, StacksMainnet,
            StacksTestnet,
        },
        wallet::{Error as StacksWalletError, StacksWallet},
    },
    stacks_secp256k1::{
        ecdsa::Signature as StacksSignature, hashes::sha256, Message, Secp256k1,
    },
    url::Url,
};

uniffi::custom_type!(StacksWalletError, String, {
    remote,
    try_lift: |_val| unimplemented!("Does not support lifting StacksWalletError"),
    lower: |obj| obj.to_string(),
});

uniffi::custom_type!(StacksTransactionError, String, {
    remote,
    try_lift: |_val| unimplemented!("Does not support lifting StacksTransactionError"),
    lower: |obj| obj.to_string(),
});

uniffi::custom_type!(ClarityError, String, {
    remote,
    try_lift: |_val| unimplemented!("Does not support lifting ClarityError"),
    lower: |obj| obj.to_string(),
});

uniffi::custom_type!(StacksSignature, String, {
    remote,
    try_lift: |val| val.parse::<StacksSignature>().map_err(Into::into),
    lower: |obj| obj.to_string(),
});

#[uniffi::export]
fn stacks_generate_wallet() -> String {
    bip32::Mnemonic::random(
        StdRng::from_rng(OsRng).unwrap(),
        bip32::Language::English,
    )
    .phrase()
    .to_owned()
    // Return the mnemonic instead of StacksWallet because we can't UniFFI lower a StacksWallet (i.e. can't get the mnemonic from it)
}

#[derive(thiserror::Error, Debug, uniffi::Error)]
pub enum StacksGetAddressError {
    #[error("Invalid secret key: {0}")]
    InvalidSecretKey(StacksWalletError),

    #[error("Failed to get account: {0}")]
    GetAccount(StacksWalletError),

    #[error("Failed to get address: {0}")]
    GetAddress(StacksWalletError),

    #[error("Invalid version: {0}")]
    InvalidVersion(String),
}

#[uniffi::export]
fn stacks_get_address(
    wallet: &str,
    version: &str,
) -> Result<String, StacksGetAddressError> {
    let mut wallet = StacksWallet::from_secret_key(wallet)
        .map_err(StacksGetAddressError::InvalidSecretKey)?;
    wallet
        .get_account(0)
        .map_err(StacksGetAddressError::GetAccount)?
        .get_address(match version {
            "mainnet-p2pkh" => Version::MainnetP2PKH,
            "mainnet-p2sh" => Version::MainnetP2SH,
            "testnet-p2pkh" => Version::TestnetP2PKH,
            "testnet-p2sh" => Version::TestnetP2SH,
            _ => {
                return Err(StacksGetAddressError::InvalidVersion(
                    version.to_string(),
                ))
            }
        })
        .map_err(StacksGetAddressError::GetAddress)
}

#[derive(thiserror::Error, Debug, uniffi::Error)]
pub enum StacksSignMessageError {
    #[error("Invalid secret key: {0}")]
    InvalidSecretKey(StacksWalletError),

    #[error("Failed to get account: {0}")]
    GetAccount(StacksWalletError),

    #[error("Failed to get private key: {0}")]
    UnwrapPrivateKey(StacksWalletError),
}

#[uniffi::export]
pub fn stacks_sign_message(
    wallet: &str,
    message: &str, // A UTF-8 encoded string. NOT hex encoded, etc.
) -> Result<String, StacksSignMessageError> {
    let mut wallet = StacksWallet::from_secret_key(wallet)
        .map_err(StacksSignMessageError::InvalidSecretKey)?;
    let sk = wallet
        .get_account(0)
        .map_err(StacksSignMessageError::GetAccount)?
        .private_key()
        .map_err(StacksSignMessageError::UnwrapPrivateKey)?;

    // Use recoverable signature which includes the recovery bit
    let signature = Secp256k1::new().sign_ecdsa_recoverable(
        &Message::from_hashed_data::<sha256::Hash>(message.as_bytes()),
        &sk,
    );

    // Convert to RSV format (r + s + v) where v is the recovery bit
    let (recovery_id, signature_compact) = signature.serialize_compact();
    let rsv = [
        &signature_compact[..32],
        &signature_compact[32..],
        &[recovery_id.to_i32() as u8],
    ]
    .concat();
    let rsv_hex = hex::encode(rsv);

    Ok(rsv_hex)
}

#[derive(thiserror::Error, Debug, uniffi::Error)]
pub enum StacksSignTransactionError {
    #[error("Invalid secret key: {0}")]
    InvalidSecretKey(StacksWalletError),

    #[error("Failed to get private key: {0}")]
    UnwrapPrivateKey(StacksWalletError),

    #[error("Invalid network: {0}")]
    InvalidNetwork(String),

    #[error("Failed to sign transaction: {0}")]
    SignTransaction(StacksTransactionError),

    #[error("Failed to hash transaction: {0}")]
    Hash(StacksTransactionError),

    #[error("Failed to encode transaction: {0}")]
    Encode(ClarityError),

    #[error("Failed to fetch account: {0}")]
    FetchAccount(String),

    #[error("Failed to fetch fee rate: {0}")]
    TransferFees(String),
}

#[derive(uniffi::Object)]
pub struct StacksClient {
    #[allow(unused)]
    provider_pool: ProviderPool,
    #[allow(unused)]
    http_client: ReqwestClient,
    #[allow(unused)]
    project_id: ProjectId,
    #[allow(unused)]
    pulse_metadata: PulseMetadata,
}

#[derive(thiserror::Error, Debug, uniffi::Error)]
pub enum StacksTransferStxError {
    #[error("Failed to sign transaction: {0}")]
    SignTransaction(StacksSignTransactionError),

    #[error("Failed to broadcast transaction: {0}")]
    BroadcastTransaction(String),
}

#[derive(thiserror::Error, Debug, uniffi::Error)]
pub enum StacksFeesError {
    #[error("Failed to fetch fee rate: {0}")]
    TransferFees(String),

    #[error("Failed to parse response: {0}")]
    InvalidResponse(String),
}

#[derive(thiserror::Error, Debug, uniffi::Error)]
pub enum StacksAccountError {
    #[error("Failed to fetch account: {0}")]
    FetchAccount(String),
}

#[derive(thiserror::Error, Debug, uniffi::Error)]
pub enum TransferFeesError {
    #[error("Failed to get transfer fees: {0}")]
    FeeRate(String),
}

#[uniffi::export(async_runtime = "tokio")]
impl StacksClient {
    #[uniffi::constructor]
    pub fn new(project_id: ProjectId, pulse_metadata: PulseMetadata) -> Self {
        Self::with_blockchain_api_url(
            project_id,
            pulse_metadata,
            BLOCKCHAIN_API_URL_PROD.parse().unwrap(),
        )
    }

    #[uniffi::constructor]
    pub fn with_blockchain_api_url(
        project_id: ProjectId,
        pulse_metadata: PulseMetadata,
        blockchain_api_base_url: Url,
    ) -> Self {
        let client = ReqwestClient::builder().build();
        let client = match client {
            Ok(client) => client,
            Err(e) => {
                panic!("Failed to create reqwest client: {} ... {:?}", e, e)
            }
        };
        Self {
            provider_pool: ProviderPool::new(
                project_id.clone(),
                client.clone(),
                pulse_metadata.clone(),
                blockchain_api_base_url,
            ),
            http_client: client,
            project_id,
            pulse_metadata,
        }
    }

    async fn sign_transaction(
        &self,
        wallet: &str,
        network: &str,
        request: TransferStxRequest,
    ) -> Result<TransferStxResponse, StacksSignTransactionError> {
        let mut stacks_wallet = StacksWallet::from_secret_key(wallet)
            .map_err(StacksSignTransactionError::InvalidSecretKey)?;
        let sender_key = stacks_wallet
            .get_account(0)
            .map_err(StacksSignTransactionError::UnwrapPrivateKey)?
            .private_key()
            .map_err(StacksSignTransactionError::UnwrapPrivateKey)?;

        let account = self.get_account(network, &request.sender).await
            .map_err(|e| StacksSignTransactionError::FetchAccount(e.to_string()))?;

        // Use fallback fee rate of 180 if transfer_fees fails
        let fee_rate = match self.transfer_fees(network).await {
            Ok(result) => result,
            Err(e) => {
                println!("Failed to fetch transfer fee rate: {}. Using fallback value of 1 fee rate / byte.", e);
                1
            }
        };

        // https://github.com/52/stacks.rs/blob/c6455fe5eeae04b2f0e3a88fe6e6c803949a8417/README.md?plain=1#L24
        let mut transfer = match network {
            network::stacks::MAINNET => STXTokenTransfer::builder()
                .recipient(clarity!(PrincipalStandard, request.recipient))
                .amount(request.amount)
                .memo(request.memo)
                .sender(sender_key)
                .network(StacksMainnet::new())
                .build()
                .transaction(),
            network::stacks::TESTNET => STXTokenTransfer::builder()
                .recipient(clarity!(PrincipalStandard, request.recipient))
                .amount(request.amount)
                .memo(request.memo)
                .sender(sender_key)
                .network(StacksTestnet::new())
                .build()
                .transaction(),
            _ => {
                return Err(StacksSignTransactionError::InvalidNetwork(
                    network.to_string(),
                ))
            }
        };

        let nonce = account.nonce;
        transfer.set_nonce(nonce);

        let bytes_length = clarity::Codec::encode(&transfer)
            .map_err(StacksSignTransactionError::Encode)?
            .len();
        let mut fees = bytes_length as u64 * fee_rate;
        // fees can't be lower than 180 microSTX
        // A typical single-signature STX transfer is ~180–200 bytes and lower fee_rate is 1
        // Fee is calculated as fee_rate * transaction_bytes
        // Anything below this will often fail with "FeeTooLow"
        fees = std::cmp::max(fees, 180); 
        transfer.set_fee(fees);

        // Scope `signed_transaction` so that it isn't held across the await boundary
        // This is needed because `Transaction` is not `Send`
        let signed_transaction = transfer
            .sign(sender_key)
            .map_err(StacksSignTransactionError::SignTransaction)?;

        let txid = signed_transaction
            .hash()
            .map_err(StacksSignTransactionError::Hash)?;
        let tx_encoded = clarity::Codec::encode(&signed_transaction)
            .map_err(StacksSignTransactionError::Encode)?;

        Ok(TransferStxResponse {
            txid: hex::encode(txid),
            transaction: hex::encode(&tx_encoded),
        })
    }

    pub async fn transfer_stx(
        &self,
        wallet: &str,
        network: &str,
        request: TransferStxRequest,
    ) -> Result<TransferStxResponse, StacksTransferStxError> {
        let stacks_client =
            self.provider_pool.get_stacks_client(network, None, None).await;

        let sign_response = self
            .sign_transaction(wallet, network, request)
            .await
            .map_err(StacksTransferStxError::SignTransaction)?;

        let tx_hex = sign_response.transaction.clone();

        let broadcast_tx_response = match stacks_client
            .stacks_transactions(tx_hex.clone())
            .await
        {
            Ok(result) => result,
            Err(e) => {
                let error_string = format!("Broadcast transaction failed - Chain ID: {}, Transaction: {}, Error: {}", network, tx_hex, e);
                return Err(StacksTransferStxError::BroadcastTransaction(
                    error_string,
                ));
            }
        };
        println!("broadcast tx response: {:?}", broadcast_tx_response);

        Ok(sign_response)
    }

    pub async fn get_account(
        &self,
        network: &str,
        principal: &str,
    ) -> Result<StacksAccount, StacksAccountError> {
        let stacks_client =
            self.provider_pool.get_stacks_client(network, None, None).await;

        let response =
            match stacks_client.stacks_accounts(principal.to_string()).await {
                Ok(result) => result,
                Err(e) => {
                    let error_string =
                        format!("Failed to fecth account: {}", e);
                    return Err(StacksAccountError::FetchAccount(error_string));
                }
            };
        println!("account response: {:?}", response);

        let account: StacksAccount =
            serde_json::from_value(response).map_err(|e| {
                StacksAccountError::FetchAccount(format!(
                    "Failed to parse response: {}",
                    e
                ))
            })?;

        Ok(account)
    }

    pub async fn transfer_fees(
        &self,
        network: &str,
    ) -> Result<u64, StacksFeesError> {
        let stacks_client =
            self.provider_pool.get_stacks_client(network, None, None).await;

        let response = match stacks_client.stacks_transfer_fees().await {
            Ok(result) => result,
            Err(e) => {
                let error_string = format!("Failed to fetch fee rate: {}", e);
                return Err(StacksFeesError::TransferFees(error_string));
            }
        };

        // Check if response is already the result value
        if let Some(fee_rate) = response.as_u64() {
            return Ok(fee_rate);
        }

        Err(StacksFeesError::InvalidResponse(format!(
            "Unexpected response format: {:?}",
            response
        )))
    }
}

#[derive(uniffi::Record, Debug, Clone)]
pub struct TransferStxRequest {
    sender: String, // address
    amount: u64,
    recipient: String, // address
    memo: String,
}

#[derive(uniffi::Record, Debug, Clone)]
pub struct TransferStxResponse {
    txid: String,
    transaction: String,
}

#[derive(
    Debug, Clone, serde::Serialize, serde::Deserialize, uniffi::Record,
)]
pub struct StacksAccount {
    pub balance: String,
    pub locked: String,
    pub unlock_height: u64,
    pub nonce: u64,
    pub balance_proof: String,
    pub nonce_proof: String,
}

#[derive(Debug, serde::Deserialize, uniffi::Record)]
pub struct FeeEstimation {
    pub cost_scalar_change_by_byte: f64,
    pub estimated_cost: EstimatedCost,
    pub estimated_cost_scalar: u64,
    pub estimations: Vec<Estimation>,
}

#[derive(Debug, serde::Deserialize, uniffi::Record)]
pub struct EstimatedCost {
    pub read_count: u64,
    pub read_length: u64,
    pub runtime: u64,
    pub write_count: u64,
    pub write_length: u64,
}

#[derive(Debug, serde::Deserialize, uniffi::Record)]
pub struct Estimation {
    pub fee: u64,
    pub fee_rate: f64,
}

#[cfg(test)]
mod tests {
    use crate::uniffi_compat::stacks::{
        stacks_generate_wallet, stacks_get_address, stacks_sign_message,
    };

    #[test]
    fn generate_wallet() {
        let wallet = stacks_generate_wallet();
        println!("Wallet: {}", wallet);
    }

    #[test]
    fn get_address() {
        let wallet = stacks_generate_wallet();
        let address = stacks_get_address(&wallet, "mainnet-p2pkh").unwrap();
        println!("Address: {}", address);
    }

    #[test]
    fn sign_message() {
        let wallet = stacks_generate_wallet();
        let message = "Hello, world!";
        let signature = stacks_sign_message(&wallet, message).unwrap();
        println!("Signature: {}", signature);
        // Note: We can't verify RSV signatures with the stacks_secp256k1 library directly
        // The JavaScript side will handle verification with publicKeyFromSignatureRsv
        assert!(!signature.is_empty());
    }

    #[test]
    fn sign_message_should_panic() {
        let wallet = stacks_generate_wallet();
        let message = "Hello, world!";
        let signature = stacks_sign_message(&wallet, message).unwrap();
        println!("Signature: {}", signature);
        // Note: We can't verify RSV signatures with the stacks_secp256k1 library directly
        // The JavaScript side will handle verification with publicKeyFromSignatureRsv
        assert!(!signature.is_empty());
    }

    #[test]
    fn test_signature_uniffi() {
        let signature =
            stacks_sign_message(&stacks_generate_wallet(), "Hello, world!")
                .unwrap();
        // Test that the signature is valid
        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 130);
    }

    #[test]
    fn test_signature_format() {
        let wallet = stacks_generate_wallet();
        let message = "Test message for signature format";
        let signature = stacks_sign_message(&wallet, message).unwrap();

        // The signature should be a hex string of 130 characters (65 bytes)
        // Format: r (32 bytes) + s (32 bytes) + v (1 byte) = 65 bytes = 130 hex chars
        assert_eq!(signature.len(), 130);

        // Verify it's valid hex
        let sig_bytes = hex::decode(&signature).expect("Valid hex");
        assert_eq!(sig_bytes.len(), 65);

        // The last byte should be the recovery bit (0 or 1)
        let recovery_bit = sig_bytes[64];
        assert!(recovery_bit == 0 || recovery_bit == 1);

        println!("Signature: {}", signature);
        println!("Recovery bit: {}", recovery_bit);
    }

    #[test]
    fn test_simple_signature() {
        // This is a simple test to verify our function works
        let wallet = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let message = "test";
        let signature = stacks_sign_message(wallet, message).unwrap();

        println!("Test signature: {}", signature);
        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 130);
    }
}
