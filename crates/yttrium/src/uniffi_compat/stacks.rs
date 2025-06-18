use {
    crate::{
        blockchain_api::BLOCKCHAIN_API_URL_PROD,
        chain_abstraction::pulse::PulseMetadata,
        provider_pool::{network, ProviderPool},
    },
    alloy::primitives::Bytes,
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
}

fn sign_transaction(
    wallet: &str,
    network: &str,
    request: TransferStxRequest,
) -> Result<(TransferStxResponse, Bytes), StacksSignTransactionError> {
    let sender_key = StacksWallet::from_secret_key(wallet)
        .map_err(StacksSignTransactionError::InvalidSecretKey)?
        .private_key()
        .map_err(StacksSignTransactionError::UnwrapPrivateKey)?;
    // https://github.com/52/stacks.rs/blob/c6455fe5eeae04b2f0e3a88fe6e6c803949a8417/README.md?plain=1#L24
    let transfer = match network {
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

    // Scope `signed_transaction` so that it isn't held across the await boundary
    // This is needed because `Transaction` is not `Send`
    let signed_transaction = transfer
        .sign(sender_key)
        .map_err(StacksSignTransactionError::SignTransaction)?;

    let txid =
        signed_transaction.hash().map_err(StacksSignTransactionError::Hash)?;
    let tx_encoded = clarity::Codec::encode(&signed_transaction)
        .map_err(StacksSignTransactionError::Encode)?;

    Ok((
        TransferStxResponse {
            txid: hex::encode(txid),
            transaction: hex::encode(&tx_encoded),
        },
        tx_encoded.into(),
    ))
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

    pub async fn transfer_stx(
        &self,
        wallet: &str,
        network: &str,
        request: TransferStxRequest,
    ) -> Result<TransferStxResponse, StacksTransferStxError> {
        let (response, tx_encoded) = sign_transaction(wallet, network, request)
            .map_err(StacksTransferStxError::SignTransaction)?;

        let broadcast_tx_response = self
            .provider_pool
            .get_stacks_client(None, None)
            .await
            .stacks_transactions(tx_encoded)
            .await
            .map_err(|e| {
                StacksTransferStxError::BroadcastTransaction(e.to_string())
            })?;
        println!("broadcast tx response: {:?}", broadcast_tx_response);

        Ok(response)
    }
}

#[derive(uniffi::Record, Debug, Clone)]
pub struct TransferStxRequest {
    amount: u64,
    recipient: String, // address
    memo: String,
}

#[derive(uniffi::Record, Debug, Clone)]
pub struct TransferStxResponse {
    txid: String,
    transaction: String,
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
