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
        rpc_types::Balance,
        types::{
            base_types::SuiAddress,
            crypto::{PublicKey, SuiKeyPair},
        },
    },
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
pub fn sui_get_address(keypair: &SuiKeyPair) -> SuiAddress {
    SuiAddress::from(&keypair.public())
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
        let address = sui_get_address(&keypair);
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(address);
        let s =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(address, s);
    }

    #[test]
    fn test_sui_get_address() {
        let keypair = sui_generate_keypair();
        let address = sui_get_address(&keypair);
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
        let address = sui_get_address(&keypair);
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
        let address = sui_get_address(&keypair);
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
        let address = sui_get_address(&keypair);
        let client = SuiClient::new(
            std::env::var("REOWN_PROJECT_ID").unwrap().into(),
            get_pulse_metadata(),
        );
        let balances =
            client.get_all_balances(MAINNET.to_owned(), address).await.unwrap();
        assert!(balances.is_empty());
    }
}
