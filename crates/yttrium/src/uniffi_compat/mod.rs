#[cfg(feature = "sui")]
pub mod sui;

#[cfg(feature = "stacks")]
pub mod stacks;

#[cfg(feature = "solana")]
use {
    crate::chain_abstraction::solana::{
        self, SolanaKeypair, SolanaPubkey, SolanaSignature,
    },
    solana_sdk::{
        bs58,
        derivation_path::DerivationPath,
        signature::generate_seed_from_seed_phrase_and_passphrase,
        signer::{SeedDerivable, Signer},
        transaction::VersionedTransaction,
    },
};
#[cfg(feature = "chain_abstraction_client")]
use crate::chain_abstraction::{
    amount::Amount, 
    api::prepare::{Eip155OrSolanaAddress, FundingMetadata}
};
use {
    crate::{
        chain_abstraction::api::prepare::Eip155OrSolanaAddress,
        smart_accounts::account_address::AccountAddress,
        wallet_service_api::{
            AddressOrNative, Asset, AssetData, Erc20Metadata, Erc721Metadata,
            NativeMetadata,
        },
    },
    alloy::{
        contract::Error as AlloyError,
        dyn_abi::Eip712Domain,
        primitives::{
            aliases::U48, Address, Bytes, PrimitiveSignature, Uint, B256, U128,
            U256, U64, U8,
        },
        rpc::types::{Authorization, TransactionReceipt, UserOperationReceipt},
        signers::local::PrivateKeySigner,
        transports::{self, TransportErrorKind},
    },
    alloy_provider::PendingTransactionError,
    eyre::Report as EyreError,
    relay_rpc::domain::ProjectId,
    reqwest::{Error as ReqwestError, StatusCode, Url},
    serde_json::Error as SerdeJsonError,
    uniffi::deps::anyhow::Error as AnyhowError,
};


// TODO use https://mozilla.github.io/uniffi-rs/next/udl/remote_ext_types.html#remote-types when it's available

uniffi::custom_type!(Address, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});
uniffi::custom_type!(AccountAddress, Address, {
    try_lift: |val| Ok(val.into()),
    lower: |obj| obj.into(),
});

#[cfg(feature = "solana")]
uniffi::custom_type!(SolanaPubkey, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

#[cfg(feature = "solana")]
uniffi::custom_type!(SolanaKeypair, String, {
    remote,
    try_lift: |val| {
        let mut buf = [0u8; relay_rpc::auth::ed25519_dalek::KEYPAIR_LENGTH];
        bs58::decode(val).onto(&mut buf)?;
        SolanaKeypair::from_bytes(&buf).map_err(Into::into)
    },
    lower: |obj| obj.to_base58_string(),
});

uniffi::custom_type!(PrivateKeySigner, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| hex::encode(obj.to_bytes()),
});

uniffi::custom_type!(PrimitiveSignature, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| format!("0x{}", hex::encode(obj.as_bytes())),
});

#[cfg(feature = "solana")]
uniffi::custom_type!(SolanaSignature, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

uniffi::custom_type!(Eip712Domain, String, {
    remote,
    try_lift: |_val| unimplemented!("Does not support lifting Eip712Domain"),
    lower: |_obj| "Does not support lowering Eip712Domain".to_owned(),
});

fn uint_to_hex<const BITS: usize, const LIMBS: usize>(
    obj: Uint<BITS, LIMBS>,
) -> String {
    format!("0x{obj:x}")
}

uniffi::custom_type!(U8, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

uniffi::custom_type!(U48, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

uniffi::custom_type!(U64, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

uniffi::custom_type!(U128, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

type U128Primitive = u128;
uniffi::custom_type!(U128Primitive, String, {
    remote,
    try_lift: |val| Ok(val.parse::<U128>()?.to()),
    lower: |obj| uint_to_hex(U128::from(obj)),
});

uniffi::custom_type!(U256, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

uniffi::custom_type!(Bytes, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

uniffi::custom_type!(B256, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

uniffi::custom_type!(ProjectId, String, {
    remote,
    try_lift: |val| Ok(val.into()),
    lower: |obj| obj.to_string(),
});

uniffi::custom_type!(Url, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

pub type RpcError = transports::RpcError<TransportErrorKind>;

uniffi::custom_type!(RpcError, String, {
    remote,
    try_lift: |_val| unimplemented!("Does not support lifting RpcError"),
    lower: |obj| obj.to_string(),
});
uniffi::custom_type!(EyreError, String, {
    remote,
    try_lift: |_val| unimplemented!("Does not support lifting EyreError"),
    lower: |obj| obj.to_string(),
});
uniffi::custom_type!(AnyhowError, String, {
    remote,
    try_lift: |_val| unimplemented!("Does not support lifting AnyhowError"),
    lower: |obj| obj.to_string(),
});
uniffi::custom_type!(AlloyError, String, {
    remote,
    try_lift: |_val| unimplemented!("Does not support lifting AlloyError"),
    lower: |obj| obj.to_string(),
});
uniffi::custom_type!(TransactionReceipt, String, {
    remote,
    try_lift: |_val| unimplemented!("Does not support lifting TransactionReceipt"),
    lower: |obj| serde_json::to_string(&obj).unwrap(),
});
uniffi::custom_type!(UserOperationReceipt, String, {
    remote,
    try_lift: |_val| unimplemented!("Does not support lifting UserOperationReceipt"),
    lower: |obj| serde_json::to_string(&obj).unwrap(),
});
uniffi::custom_type!(PendingTransactionError, String, {
    remote,
    try_lift: |_val| unimplemented!("Does not support lifting PendingTransactionError"),
    lower: |obj| obj.to_string(),
});
uniffi::custom_type!(ReqwestError, String, {
    remote,
    try_lift: |_val| unimplemented!("Does not support lifting ReqwestError"),
    lower: |obj| obj.to_string(),
});
uniffi::custom_type!(SerdeJsonError, String, {
    remote,
    try_lift: |_val| unimplemented!("Does not support lifting SerdeJsonError"),
    lower: |obj| obj.to_string(),
});

// uniffi::custom_type!(Unit, u8, {
//     try_lift: |val| Ok(Unit::new(val).expect("Unit must be less than 77")),
//     lower: |obj| obj.get(),
// });

#[cfg(feature = "chain_abstraction_client")]
#[uniffi::export]
fn funding_metadata_to_amount(value: FundingMetadata) -> Amount {
    value.to_amount()
}

#[cfg(feature = "chain_abstraction_client")]
#[uniffi::export]
fn funding_metadata_to_bridging_fee_amount(value: FundingMetadata) -> Amount {
    value.to_bridging_fee_amount()
}

uniffi::custom_type!(Authorization, FfiAuthorization, {
    remote,
    try_lift: |val| Ok(Authorization {
        chain_id: val.chain_id,
        address: val.address,
        nonce: val.nonce,
    }),
    lower: |obj| FfiAuthorization {
        chain_id: obj.chain_id,
        address: obj.address,
        nonce: obj.nonce,
    },
});

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiAuthorization {
    /// The chain ID of the authorization.
    pub chain_id: U256,
    /// The address of the authorization.
    pub address: Address,
    /// The nonce for the authorization.
    pub nonce: u64,
}

uniffi::custom_type!(Eip155OrSolanaAddress, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

#[cfg(feature = "solana")]
uniffi::custom_type!(VersionedTransaction, String, {
    remote,
    try_lift: |data| Ok(bincode::deserialize::<VersionedTransaction>(&data_encoding::BASE64.decode(data.as_bytes())?)?),
    lower: |obj| data_encoding::BASE64.encode(&bincode::serialize(&obj).unwrap()),
});

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Error, thiserror::Error)]
pub enum SolanaDeriveKeypairFromMnemonicError {
    #[error("Derivation path: {0}")]
    DerivationPath(String),

    #[error("Derive: {0}")]
    Derive(String),
}

#[cfg(feature = "solana")]
#[uniffi::export]
fn solana_pubkey_for_keypair(keypair: SolanaKeypair) -> solana::SolanaPubkey {
    keypair.pubkey()
}

#[cfg(feature = "solana")]
#[uniffi::export]
fn solana_sign_prehash(
    keypair: SolanaKeypair,
    message: Bytes,
) -> SolanaSignature {
    keypair.sign_message(&message)
}

#[cfg(feature = "solana")]
#[uniffi::export]
fn solana_generate_keypair() -> SolanaKeypair {
    SolanaKeypair::new()
}

#[cfg(feature = "solana")]
#[uniffi::export]
fn solana_phantom_derivation_path_with_account(account: u32) -> String {
    format!("m/44'/501'/{account}'/0'")
}

#[cfg(feature = "solana")]
#[uniffi::export]
fn solana_derive_keypair_from_mnemonic(
    mnemonic: String,
    derivation_path: Option<String>,
) -> Result<SolanaKeypair, SolanaDeriveKeypairFromMnemonicError> {
    let seed = generate_seed_from_seed_phrase_and_passphrase(&mnemonic, "");

    let derivation_path = if let Some(path) = derivation_path {
        Some(DerivationPath::from_absolute_path_str(&path).map_err(|e| {
            SolanaDeriveKeypairFromMnemonicError::DerivationPath(e.to_string())
        })?)
    } else {
        None
    };

    SolanaKeypair::from_seed_and_derivation_path(&seed, derivation_path)
        .map_err(|e| {
            SolanaDeriveKeypairFromMnemonicError::Derive(e.to_string())
        })
}

uniffi::custom_type!(Asset, AssetFfi, {
    try_lift: |val| Ok(val.into()),
    lower: |obj| obj.into(),
});

#[derive(Debug, Clone, PartialEq, uniffi_macros::Enum)]
pub enum AssetFfi {
    Native { address: AddressOrNative, balance: U256, metadata: NativeMetadata },
    Erc20 { address: AddressOrNative, balance: U256, metadata: Erc20Metadata },
    Erc721 { address: AddressOrNative, balance: U256, metadata: Erc721Metadata },
}

impl From<AssetFfi> for Asset {
    fn from(value: AssetFfi) -> Self {
        match value {
            AssetFfi::Native { address, balance, metadata } => {
                Self::Native { data: AssetData { address, balance, metadata } }
            }
            AssetFfi::Erc20 { address, balance, metadata } => {
                Self::Erc20 { data: AssetData { address, balance, metadata } }
            }
            AssetFfi::Erc721 { address, balance, metadata } => {
                Self::Erc721 { data: AssetData { address, balance, metadata } }
            }
        }
    }
}

impl From<Asset> for AssetFfi {
    fn from(value: Asset) -> Self {
        match value {
            Asset::Native {
                data: AssetData { address, balance, metadata },
            } => Self::Native { address, balance, metadata },
            Asset::Erc20 { data: AssetData { address, balance, metadata } } => {
                Self::Erc20 { address, balance, metadata }
            }
            Asset::Erc721 {
                data: AssetData { address, balance, metadata },
            } => Self::Erc721 { address, balance, metadata },
        }
    }
}

uniffi::custom_type!(StatusCode, u16, {
    remote,
    try_lift: |val| StatusCode::from_u16(val).map_err(Into::into),
    lower: |obj| obj.as_u16(),
});

#[cfg(test)]
mod tests {
    use {
        super::*,
        alloy::primitives::{address, bytes, U32},
    };

    #[test]
    fn test_address_lower() {
        let ffi_u64 = address!("abababababababababababababababababababab");
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(ffi_u64);
        let s: String =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(s, format!("0xABaBaBaBABabABabAbAbABAbABabababaBaBABaB"));
    }

    #[test]
    fn test_u64_lower() {
        let num = 1234567890;
        let ffi_u64 = U64::from(num);
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(ffi_u64);
        let s: String =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(s, format!("0x{num:x}"));
    }

    #[test]
    fn test_u128_lower() {
        let num = 1234567890;
        let ffi_u64 = U128::from(num);
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(ffi_u64);
        let s: String =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(s, format!("0x{num:x}"));
    }

    #[test]
    fn test_u256_lower() {
        let num = 1234567890;
        let ffi_u64 = U256::from(num);
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(ffi_u64);
        let s: String =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(s, format!("0x{num:x}"));
    }

    #[test]
    fn test_bytes_lower() {
        let ffi_u64 = bytes!("aabbccdd");
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(ffi_u64);
        let s: String =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(s, format!("0xaabbccdd"));
    }

    #[cfg(feature = "solana")]
    #[test]
    fn test_solana_signature_lower() {
        let ffi_u64 = solana_sdk::signature::Signature::from([0xab; 64]);
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(ffi_u64);
        let s: String =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(s, format!("4S55ApgNWn8YKQL5J2uuxtfZrYXQZqBs8BUJTqGv3us4cAefggxxMLavbor7u47x4BfUhDRkfFBpW2rJTU6YMxux"));
    }

    #[test]
    fn test_u32_raise() {
        let s = "0x1";
        let n = s.parse::<U32>().unwrap();
        assert_eq!(n, Uint::from(1));
    }

    #[test]
    fn test_u64_raise() {
        let s = "0x1";
        let n = s.parse::<U64>().unwrap();
        assert_eq!(n, Uint::from(1));
    }

    #[test]
    fn test_u128_raise() {
        let s = "0x1";
        let n = s.parse::<U128>().unwrap();
        assert_eq!(n, Uint::from(1));
    }

    #[test]
    fn test_u256_raise() {
        let s = "0x1";
        let n = s.parse::<U256>().unwrap();
        assert_eq!(n, Uint::from(1));
    }
}
