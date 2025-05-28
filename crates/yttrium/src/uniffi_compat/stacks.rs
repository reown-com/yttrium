use {
    crate::{
        blockchain_api::BLOCKCHAIN_API_URL_PROD,
        chain_abstraction::pulse::PulseMetadata,
        provider_pool::{network, ProviderPool},
    },
    rand::{
        rngs::{OsRng, StdRng},
        SeedableRng,
    },
    relay_rpc::domain::ProjectId,
    reqwest::Client as ReqwestClient,
    stacks_rs::{
        clarity,
        crypto::c32::Version,
        transaction::{STXTokenTransfer, StacksMainnet},
        wallet::StacksWallet,
    },
    stacks_secp256k1::{
        ecdsa::Signature as StacksSignature, hashes::sha256, Message, Secp256k1,
    },
    url::Url,
};

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

#[uniffi::export]
fn stacks_get_address(
    wallet: &str,
    version: &str,
) -> Result<String, eyre::Report> {
    let mut wallet = StacksWallet::from_secret_key(wallet)?;
    wallet
        .get_account(0)
        .unwrap()
        .get_address(match version {
            "mainnet-p2pkh" => Version::MainnetP2PKH,
            "mainnet-p2sh" => Version::MainnetP2SH,
            "testnet-p2pkh" => Version::TestnetP2PKH,
            "testnet-p2sh" => Version::TestnetP2SH,
            _ => return Err(eyre::eyre!("Invalid version")),
        })
        .map_err(Into::into)
}

#[uniffi::export]
fn stacks_sign_message(
    wallet: &str,
    // A UTF-8 encoded string. NOT hex encoded, etc.
    message: &str,
) -> Result<StacksSignature, eyre::Report> {
    let wallet = StacksWallet::from_secret_key(wallet)?;
    let signature = Secp256k1::new().sign_ecdsa(
        &Message::from_hashed_data::<sha256::Hash>(message.as_bytes()),
        &wallet.private_key().unwrap(),
    );
    Ok(signature)
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
    ) -> Result<TransferStxResponse, eyre::Report> {
        let sender_key =
            StacksWallet::from_secret_key(wallet)?.private_key()?;
        // https://github.com/52/stacks.rs/blob/c6455fe5eeae04b2f0e3a88fe6e6c803949a8417/README.md?plain=1#L24
        let transfer = STXTokenTransfer::builder()
            .recipient(clarity!(PrincipalStandard, request.recipient))
            .amount(request.amount)
            .memo(request.memo)
            .sender(sender_key)
            .network(match network {
                network::stacks::MAINNET => StacksMainnet::new(),
                // network::stacks::TESTNET => StacksTestnet::new(),
                _ => return Err(eyre::eyre!("Invalid network")),
            })
            .build()
            .transaction();

        let (response, tx_encoded) = {
            // Scope `signed_transaction` so that it isn't held across the await boundary
            // This is needed because `Transaction` is not `Send`
            let signed_transaction = transfer.sign(sender_key)?;

            let txid = signed_transaction.hash()?;
            let tx_encoded = clarity::Codec::encode(&signed_transaction)?;
            (
                TransferStxResponse {
                    txid: hex::encode(txid),
                    transaction: hex::encode(&tx_encoded),
                },
                tx_encoded.into(),
            )
        };

        let broadcast_tx_response = self
            .provider_pool
            .get_stacks_client(None, None)
            .await
            .stacks_transactions(tx_encoded)
            .await?;
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
    use {
        crate::uniffi_compat::stacks::{
            stacks_generate_wallet, stacks_get_address, stacks_sign_message,
        },
        stacks_rs::wallet::StacksWallet,
        stacks_secp256k1::{hashes::sha256, Message, Secp256k1},
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
        Secp256k1::new()
            .verify_ecdsa(
                &Message::from_hashed_data::<sha256::Hash>(message.as_bytes()),
                &signature,
                &StacksWallet::from_secret_key(wallet.clone())
                    .unwrap()
                    .public_key()
                    .unwrap(),
            )
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn sign_message_should_panic() {
        let wallet = stacks_generate_wallet();
        let message = "Hello, world!";
        let message2 = "Hello, world2!";
        let signature = stacks_sign_message(&wallet, message).unwrap();
        println!("Signature: {}", signature);
        Secp256k1::new()
            .verify_ecdsa(
                &Message::from_hashed_data::<sha256::Hash>(message2.as_bytes()),
                &signature,
                &StacksWallet::from_secret_key(wallet.clone())
                    .unwrap()
                    .public_key()
                    .unwrap(),
            )
            .unwrap();
    }

    #[test]
    fn test_signature_uniffi() {
        let signature =
            stacks_sign_message(&stacks_generate_wallet(), "Hello, world!")
                .unwrap();
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(signature);
        let s =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(signature, s);
    }
}
