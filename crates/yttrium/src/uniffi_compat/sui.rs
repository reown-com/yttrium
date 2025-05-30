use {
    crate::{
        blockchain_api::BLOCKCHAIN_API_URL_PROD,
        chain_abstraction::pulse::PulseMetadata, provider_pool::ProviderPool,
    },
    fastcrypto::{
        ed25519::Ed25519KeyPair,
        traits::{EncodeDecodeBase64, KeyPair},
    },
    rand::{
        rngs::{OsRng, StdRng},
        SeedableRng,
    },
    relay_rpc::domain::ProjectId,
    reqwest::Client as ReqwestClient,
    sui_sdk::{
        rpc_types::{Balance, SuiTransactionBlockResponseOptions},
        types::{
            base_types::SuiAddress,
            crypto::{PublicKey, Signature, SuiKeyPair},
            digests::TransactionDigest,
            signature::GenericSignature,
            transaction::TransactionData,
        },
    },
    sui_shared_crypto::intent::{Intent, IntentMessage, PersonalMessage},
    uniffi::deps::anyhow,
    url::Url,
};

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum SuiError {
    #[error("General {0}")]
    General(String),
}

uniffi::custom_type!(SuiKeyPair, String, {
    remote,
    try_lift: |val| SuiKeyPair::decode(&val).map_err(|e| anyhow::anyhow!(e)),
    lower: |obj| obj.encode().unwrap(),
});

uniffi::custom_type!(PublicKey, String, {
    remote,
    try_lift: |val| val.parse::<PublicKey>().map_err(|e| anyhow::anyhow!(e)),
    lower: |obj| obj.encode_base64(),
});

uniffi::custom_type!(SuiAddress, String, {
    remote,
    try_lift: |val| val.parse(),
    lower: |obj| obj.to_string(),
});

uniffi::custom_type!(Signature, String, {
    remote,
    try_lift: |val| val.parse::<Signature>().map_err(|e| anyhow::anyhow!(e)),
    lower: |obj| obj.encode_base64(),
});

uniffi::custom_type!(TransactionDigest, String, {
    remote,
    try_lift: |val| val.parse::<TransactionDigest>().map_err(|e| anyhow::anyhow!(e)),
    lower: |obj| obj.to_string(),
});

// TODO support other key types
#[uniffi::export]
pub fn sui_generate_keypair() -> SuiKeyPair {
    SuiKeyPair::Ed25519(Ed25519KeyPair::generate(
        &mut StdRng::from_rng(OsRng).unwrap(),
    ))
}

#[uniffi::export]
pub fn sui_get_public_key(keypair: &SuiKeyPair) -> PublicKey {
    keypair.public()
}

#[uniffi::export]
pub fn sui_get_address(public_key: &PublicKey) -> SuiAddress {
    SuiAddress::from(public_key)
}

#[uniffi::export]
pub fn sui_personal_sign(keypair: &SuiKeyPair, message: Vec<u8>) -> Signature {
    let intent_msg = IntentMessage::new(
        Intent::personal_message(),
        PersonalMessage { message },
    );
    Signature::new_secure(&intent_msg, keypair)
}

#[uniffi::export]
pub fn sui_sign_transaction(
    keypair: &SuiKeyPair,
    tx_data: Vec<u8>,
) -> Signature {
    let data = bcs::from_bytes::<TransactionData>(&tx_data).unwrap();
    let intent_msg = IntentMessage::new(Intent::sui_transaction(), data);
    Signature::new_secure(&intent_msg, keypair)
}

#[derive(uniffi::Object)]
pub struct SuiClient {
    provider_pool: ProviderPool,
    #[allow(unused)]
    http_client: ReqwestClient,
    #[allow(unused)]
    project_id: ProjectId,
    #[allow(unused)]
    pulse_metadata: PulseMetadata,
}

#[uniffi::export(async_runtime = "tokio")]
impl SuiClient {
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

    pub async fn get_all_balances(
        &self,
        chain_id: String,
        address: SuiAddress,
    ) -> Result<Vec<Balance>, SuiError> {
        self.provider_pool
            .get_sui_client(chain_id)
            .await
            .coin_read_api()
            .get_all_balances(address)
            .await
            .map_err(|e| SuiError::General(e.to_string()))
    }

    pub async fn sign_and_execute_transaction(
        &self,
        chain_id: String,
        keypair: &SuiKeyPair,
        tx_data: &[u8],
    ) -> Result<TransactionDigest, SuiError> {
        let data = bcs::from_bytes::<TransactionData>(tx_data).unwrap();
        let intent_msg = IntentMessage::new(Intent::sui_transaction(), data);
        let sig = Signature::new_secure(&intent_msg, keypair);
        let sui_client = self.provider_pool.get_sui_client(chain_id).await;
        let result = sui_client
            .quorum_driver_api()
            .execute_transaction_block(
                sui_sdk::types::transaction::Transaction::from_generic_sig_data(
                    intent_msg.value,
                    vec![GenericSignature::Signature(sig)],
                ),
                SuiTransactionBlockResponseOptions::default(),
                None,
            )
            .await
            .map_err(|e| SuiError::General(e.to_string()))?;
        Ok(result.digest)
    }
}

#[derive(uniffi::Record)]
pub struct BalanceFfi {
    pub coin_type: String,
    pub total_balance: u64,
}

uniffi::custom_type!(Balance, BalanceFfi, {
    remote,
    try_lift: |_val| unimplemented!("Does not support lifting Balance"),
    lower: |obj| BalanceFfi {
        coin_type: obj.coin_type,
        total_balance: u64::try_from(obj.total_balance).unwrap(),
    },
});

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{
            chain_abstraction::pulse::get_pulse_metadata,
            provider_pool::network::sui::{DEVNET, MAINNET, TESTNET},
        },
        sui_sdk::types::crypto::{SignatureScheme, SuiSignature},
    };

    #[test]
    fn test_sui_generate_keypair() {
        let _keypair = sui_generate_keypair();
    }

    #[test]
    fn test_sui_keypair_uniffi() {
        let keypair = sui_generate_keypair();
        let u =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(keypair.copy());
        let s =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(keypair, s);
    }

    #[test]
    fn test_sui_public_key_uniffi() {
        let keypair = sui_generate_keypair();
        let public_key = sui_get_public_key(&keypair);
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(
            public_key.clone(),
        );
        let s =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(public_key, s);
    }

    #[test]
    fn test_sui_get_public_key() {
        let keypair = sui_generate_keypair();
        let public_key = sui_get_public_key(&keypair);
        assert_eq!(public_key, keypair.public());
    }

    #[test]
    fn test_sui_address_uniffi() {
        let keypair = sui_generate_keypair();
        let address = sui_get_address(&keypair.public());
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(address);
        let s =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(address, s);
    }

    #[test]
    fn test_sui_get_address() {
        let keypair = sui_generate_keypair();
        let address = sui_get_address(&keypair.public());
        assert_eq!(
            address.to_string(),
            SuiAddress::from(&keypair.public()).to_string()
        );
    }

    #[tokio::test]
    #[cfg(feature = "test_depends_on_env_REOWN_PROJECT_ID")]
    async fn test_sui_get_balance_random_address() {
        let zero_address = SuiAddress::random_for_testing_only();
        let client = SuiClient::new(
            std::env::var("REOWN_PROJECT_ID").unwrap().into(),
            get_pulse_metadata(),
        );
        let balances = client
            .get_all_balances("sui:mainnet".to_owned(), zero_address)
            .await
            .unwrap();
        assert!(balances.is_empty());
    }

    #[tokio::test]
    #[cfg(feature = "test_depends_on_env_REOWN_PROJECT_ID")]
    async fn test_sui_get_balance_devnet() {
        let keypair = sui_generate_keypair();
        let address = sui_get_address(&keypair.public());
        let client = SuiClient::new(
            std::env::var("REOWN_PROJECT_ID").unwrap().into(),
            get_pulse_metadata(),
        );
        let balances =
            client.get_all_balances(DEVNET.to_owned(), address).await.unwrap();
        assert!(balances.is_empty());
    }

    #[tokio::test]
    #[cfg(feature = "test_depends_on_env_REOWN_PROJECT_ID")]
    async fn test_sui_get_balance_testnet() {
        let keypair = sui_generate_keypair();
        let address = sui_get_address(&keypair.public());
        let client = SuiClient::new(
            std::env::var("REOWN_PROJECT_ID").unwrap().into(),
            get_pulse_metadata(),
        );
        let balances =
            client.get_all_balances(TESTNET.to_owned(), address).await.unwrap();
        assert!(balances.is_empty());
    }

    #[tokio::test]
    #[cfg(feature = "test_depends_on_env_REOWN_PROJECT_ID")]
    async fn test_sui_get_balance_mainnet() {
        let keypair = sui_generate_keypair();
        let address = sui_get_address(&keypair.public());
        let client = SuiClient::new(
            std::env::var("REOWN_PROJECT_ID").unwrap().into(),
            get_pulse_metadata(),
        );
        let balances =
            client.get_all_balances(MAINNET.to_owned(), address).await.unwrap();
        assert!(balances.is_empty());
    }

    #[test]
    fn test_signature_uniffi() {
        let signature = sui_personal_sign(
            &sui_generate_keypair(),
            "Hello, world!".as_bytes().to_vec(),
        );
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(
            signature.clone(),
        );
        let s =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(signature, s);
    }

    #[test]
    fn test_sui_sign() {
        let keypair = sui_generate_keypair();
        let message = "Hello, world!".as_bytes().to_vec();
        let signature = sui_personal_sign(&keypair, message.clone());
        let verification = signature.verify_secure(
            &IntentMessage::new(
                Intent::personal_message(),
                PersonalMessage { message },
            ),
            sui_get_address(&keypair.public()),
            SignatureScheme::ED25519,
        );
        assert!(verification.is_ok());
    }
}
