use {
    fastcrypto::{
        ed25519::Ed25519KeyPair,
        traits::{EncodeDecodeBase64, KeyPair},
    },
    rand::{
        rngs::{OsRng, StdRng},
        SeedableRng,
    },
    sui_sdk::{
        types::{
            base_types::SuiAddress,
            crypto::{PublicKey, SuiKeyPair},
        },
        SuiClientBuilder,
    },
    uniffi::deps::anyhow,
};

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

// TODO support other coins
// TODO get all_balances?
#[uniffi::export(async_runtime = "tokio")]
pub async fn sui_get_balance(
    address: SuiAddress,
) -> Result<u64, anyhow::Error> {
    let sui = SuiClientBuilder::default().build_mainnet().await?;
    let balance = sui.coin_read_api().get_balance(address, None).await?;
    Ok(u64::try_from(balance.total_balance)?)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    async fn test_sui_get_balance() {
        let keypair = sui_generate_keypair();
        let address = sui_get_address(&keypair);
        let balance = sui_get_balance(address).await.unwrap();
        assert_eq!(balance, 0);
    }

    #[tokio::test]
    async fn test_sui_get_balance_zero_address() {
        let zero_address = SuiAddress::ZERO;
        let balance = sui_get_balance(zero_address).await.unwrap();
        assert_ne!(balance, 0);
    }
}
