pub mod log;

#[cfg(feature = "sign_client")]
pub mod sign;

#[cfg(feature = "clear_signing")]
pub mod clear_signing;

#[cfg(feature = "sui")]
pub mod sui;

#[cfg(feature = "stacks")]
pub mod stacks;

#[cfg(feature = "evm_signing")]
pub mod evm_signing;

#[cfg(feature = "ton")]
pub mod ton;

#[cfg(feature = "tron")]
pub mod tron;

#[cfg(feature = "chain_abstraction_client")]
use crate::chain_abstraction::{
    amount::Amount,
    api::prepare::{Eip155OrSolanaAddress, FundingMetadata},
};
#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client"
))]
use crate::smart_accounts::account_address::AccountAddress;
#[cfg(feature = "chain_abstraction_client")]
use crate::wallet_service_api::{
    AddressOrNative, Asset, AssetData, Erc20Metadata, Erc721Metadata,
    NativeMetadata,
};
#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
))]
use relay_rpc::domain::ProjectId;
#[cfg(feature = "solana")]
use {
    crate::chain_abstraction::solana::{
        self, SolanaKeypair, SolanaPubkey, SolanaSignTransactionError,
        SolanaSignature, SolanaSignedTransaction, sign_versioned_transaction,
    },
    solana_derivation_path::DerivationPath,
    solana_sdk::bs58,
    solana_seed_derivable::SeedDerivable,
    solana_seed_phrase::generate_seed_from_seed_phrase_and_passphrase,
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};
#[cfg(feature = "sign_client")]
use {
    alloy::rpc::json_rpc::Id,
    relay_rpc::domain::{ClientId, Topic},
};
#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "erc6492_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
    feature = "evm_signing",
))]
use {
    alloy::{
        contract::Error as AlloyError,
        dyn_abi::Eip712Domain,
        primitives::{
            Address, B256, Bytes, Signature as PrimitiveSignature, U8, U64,
            U128, U256, Uint, aliases::U48,
        },
        rpc::types::{Authorization, TransactionReceipt, UserOperationReceipt},
        signers::local::PrivateKeySigner,
        transports::{self, TransportErrorKind},
    },
    alloy_provider::PendingTransactionError,
};
use {
    eyre::Report as EyreError,
    reqwest::{Error as ReqwestError, StatusCode, Url},
    serde_json::Error as SerdeJsonError,
    uniffi::deps::anyhow::Error as AnyhowError,
};

// TODO use https://mozilla.github.io/uniffi-rs/next/udl/remote_ext_types.html#remote-types when it's available

#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "erc6492_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
    feature = "evm_signing",
))]
uniffi::custom_type!(Address, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});
#[cfg(any(feature = "account_client", feature = "chain_abstraction_client"))]
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
        SolanaKeypair::try_from(buf.as_ref()).map_err(Into::into)
    },
    lower: |obj| obj.to_base58_string(),
});

#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "erc6492_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
    feature = "evm_signing",
))]
uniffi::custom_type!(PrivateKeySigner, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| hex::encode(obj.to_bytes()),
});

#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "erc6492_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
    feature = "evm_signing",
))]
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

#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "erc6492_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
    feature = "evm_signing",
))]
uniffi::custom_type!(Eip712Domain, String, {
    remote,
    try_lift: |_val| unimplemented!("Does not support lifting Eip712Domain"),
    lower: |_obj| "Does not support lowering Eip712Domain".to_owned(),
});

#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "erc6492_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
    feature = "evm_signing",
))]
fn uint_to_hex<const BITS: usize, const LIMBS: usize>(
    obj: Uint<BITS, LIMBS>,
) -> String {
    format!("0x{obj:x}")
}

#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "erc6492_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
    feature = "evm_signing",
))]
uniffi::custom_type!(U8, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "erc6492_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
    feature = "evm_signing",
))]
uniffi::custom_type!(U48, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "erc6492_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
    feature = "evm_signing",
))]
uniffi::custom_type!(U64, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "erc6492_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
    feature = "evm_signing",
))]
uniffi::custom_type!(U128, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "erc6492_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
    feature = "evm_signing",
))]
type U128Primitive = u128;
#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "erc6492_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
    feature = "evm_signing",
))]
uniffi::custom_type!(U128Primitive, String, {
    remote,
    try_lift: |val| Ok(val.parse::<U128>()?.to()),
    lower: |obj| uint_to_hex(U128::from(obj)),
});

#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "erc6492_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
    feature = "evm_signing",
))]
uniffi::custom_type!(U256, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "erc6492_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
    feature = "evm_signing",
))]
uniffi::custom_type!(Bytes, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "erc6492_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
    feature = "evm_signing",
))]
uniffi::custom_type!(B256, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
))]
uniffi::custom_type!(ProjectId, String, {
    remote,
    try_lift: |val| Ok(val.into()),
    lower: |obj| obj.to_string(),
});

#[cfg(feature = "sign_client")]
uniffi::custom_type!(ClientId, String, {
    remote,
    try_lift: |val| Ok(val.into()),
    lower: |obj| obj.to_string(),
});

#[cfg(feature = "sign_client")]
uniffi::custom_type!(Topic, String, {
    remote,
    try_lift: |val| Ok(val.into()),
    lower: |obj| obj.to_string(),
});

#[cfg(feature = "sign_client")]
uniffi::custom_type!(Id, String, {
    remote,
    try_lift: |val| {
        use alloy::rpc::json_rpc::Id;
        if let Ok(num) = val.parse::<u64>() {
            Ok(Id::Number(num))
        } else {
            Ok(Id::String(val))
        }
    },
    lower: |obj| obj.to_string(),
});

uniffi::custom_type!(Url, String, {
    remote,
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "erc6492_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
    feature = "evm_signing",
))]
pub type RpcError = transports::RpcError<TransportErrorKind>;

#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "erc6492_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
    feature = "evm_signing",
))]
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
#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "erc6492_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
    feature = "evm_signing",
))]
uniffi::custom_type!(AlloyError, String, {
    remote,
    try_lift: |_val| unimplemented!("Does not support lifting AlloyError"),
    lower: |obj| obj.to_string(),
});
#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "erc6492_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
    feature = "evm_signing",
))]
uniffi::custom_type!(TransactionReceipt, String, {
    remote,
    try_lift: |_val| unimplemented!("Does not support lifting TransactionReceipt"),
    lower: |obj| serde_json::to_string(&obj).unwrap(),
});
#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "erc6492_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
    feature = "evm_signing",
))]
uniffi::custom_type!(UserOperationReceipt, String, {
    remote,
    try_lift: |_val| unimplemented!("Does not support lifting UserOperationReceipt"),
    lower: |obj| serde_json::to_string(&obj).unwrap(),
});
#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "erc6492_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
    feature = "evm_signing",
))]
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

#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "erc6492_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
    feature = "evm_signing",
))]
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

#[cfg(any(
    feature = "account_client",
    feature = "chain_abstraction_client",
    feature = "erc6492_client",
    feature = "transaction_sponsorship_client",
    feature = "sign_client",
    feature = "evm_signing",
))]
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct FfiAuthorization {
    /// The chain ID of the authorization.
    pub chain_id: U256,
    /// The address of the authorization.
    pub address: Address,
    /// The nonce for the authorization.
    pub nonce: u64,
}

#[cfg(feature = "chain_abstraction_client")]
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
fn solana_sign_transaction(
    keypair: SolanaKeypair,
    transaction: VersionedTransaction,
) -> Result<SolanaSignedTransaction, SolanaSignTransactionError> {
    sign_versioned_transaction(&keypair, transaction)
}

#[cfg(feature = "solana")]
#[uniffi::export]
fn solana_sign_all_transactions(
    keypair: SolanaKeypair,
    transactions: Vec<VersionedTransaction>,
) -> Result<Vec<SolanaSignedTransaction>, SolanaSignTransactionError> {
    transactions
        .into_iter()
        .map(|tx| sign_versioned_transaction(&keypair, tx))
        .collect()
}

// Alias of solana_sign_prehash matching the WalletConnect solana_signMessage
// method name. Delegates to keep behavior in lockstep if signing ever changes.
#[cfg(feature = "solana")]
#[uniffi::export]
fn solana_sign_message(
    keypair: SolanaKeypair,
    message: Bytes,
) -> SolanaSignature {
    solana_sign_prehash(keypair, message)
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

#[cfg(feature = "chain_abstraction_client")]
uniffi::custom_type!(Asset, AssetFfi, {
    try_lift: |val| Ok(val.into()),
    lower: |obj| obj.into(),
});

#[cfg(feature = "chain_abstraction_client")]
#[derive(Debug, Clone, PartialEq, uniffi_macros::Enum)]
pub enum AssetFfi {
    Native { address: AddressOrNative, balance: U256, metadata: NativeMetadata },
    Erc20 { address: AddressOrNative, balance: U256, metadata: Erc20Metadata },
    Erc721 { address: AddressOrNative, balance: U256, metadata: Erc721Metadata },
}

#[cfg(feature = "chain_abstraction_client")]
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

#[cfg(feature = "chain_abstraction_client")]
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

#[cfg(all(
    test,
    any(
        feature = "account_client",
        feature = "chain_abstraction_client",
        feature = "erc6492_client",
        feature = "transaction_sponsorship_client",
        feature = "sign_client",
        feature = "evm_signing",
    )
))]
mod tests {
    use {
        super::*,
        alloy::primitives::{U32, address, bytes},
    };

    #[test]
    fn test_address_lower() {
        let ffi_u64 = address!("abababababababababababababababababababab");
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(ffi_u64);
        let s: String =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(s, "0xABaBaBaBABabABabAbAbABAbABabababaBaBABaB");
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
        assert_eq!(s, "0xaabbccdd");
    }

    #[cfg(feature = "solana")]
    #[test]
    fn test_solana_signature_lower() {
        let ffi_u64 = solana_signature::Signature::from([0xab; 64]);
        let u = ::uniffi::FfiConverter::<crate::UniFfiTag>::lower(ffi_u64);
        let s: String =
            ::uniffi::FfiConverter::<crate::UniFfiTag>::try_lift(u).unwrap();
        assert_eq!(
            s,
            "4S55ApgNWn8YKQL5J2uuxtfZrYXQZqBs8BUJTqGv3us4cAefggxxMLavbor7u47x4BfUhDRkfFBpW2rJTU6YMxux"
        );
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

#[cfg(all(test, feature = "solana"))]
mod solana_sign_tests {
    use {
        super::*,
        solana_sdk::{
            hash::Hash,
            message::{Message, VersionedMessage},
            pubkey::Pubkey,
        },
        solana_system_interface::instruction as system_instruction,
    };

    fn unsigned_transfer_tx(
        payer: &Pubkey,
        recipient: &Pubkey,
        lamports: u64,
    ) -> VersionedTransaction {
        let ix = system_instruction::transfer(payer, recipient, lamports);
        let mut msg = Message::new(&[ix], Some(payer));
        msg.recent_blockhash = Hash::new_from_array([7u8; 32]);
        VersionedTransaction {
            signatures: vec![SolanaSignature::default()],
            message: VersionedMessage::Legacy(msg),
        }
    }

    #[test]
    fn signs_and_verifies() {
        let payer = SolanaKeypair::new();
        let recipient = SolanaKeypair::new();
        let tx = unsigned_transfer_tx(&payer.pubkey(), &recipient.pubkey(), 1);

        let signed =
            solana_sign_transaction(payer.insecure_clone(), tx).expect("sign");

        assert_eq!(signed.transaction.signatures.len(), 1);
        assert_eq!(signed.transaction.signatures[0], signed.signature);
        assert!(signed.signature.verify(
            payer.pubkey().as_ref(),
            &signed.transaction.message.serialize(),
        ));
    }

    #[test]
    fn rejects_non_signer_keypair() {
        let payer = SolanaKeypair::new();
        let recipient = SolanaKeypair::new();
        let stranger = SolanaKeypair::new();
        let tx = unsigned_transfer_tx(&payer.pubkey(), &recipient.pubkey(), 1);

        let err = solana_sign_transaction(stranger.insecure_clone(), tx)
            .expect_err("should reject");
        assert!(matches!(
            err,
            SolanaSignTransactionError::SignerNotRequired { .. }
        ));
    }

    #[test]
    fn sign_all_preserves_order_and_signs_each() {
        let payer = SolanaKeypair::new();
        let recipient = SolanaKeypair::new();
        let txs = (1..=3u64)
            .map(|n| {
                unsigned_transfer_tx(&payer.pubkey(), &recipient.pubkey(), n)
            })
            .collect::<Vec<_>>();

        let signed =
            solana_sign_all_transactions(payer.insecure_clone(), txs.clone())
                .expect("sign all");

        assert_eq!(signed.len(), 3);
        for (i, s) in signed.iter().enumerate() {
            assert!(s.signature.verify(
                payer.pubkey().as_ref(),
                &s.transaction.message.serialize(),
            ));
            assert_eq!(s.transaction.signatures[0], s.signature);
            let expected = &txs[i].message.serialize();
            assert_eq!(&s.transaction.message.serialize(), expected);
        }
    }

    #[test]
    fn sign_message_matches_sign_prehash() {
        let kp = SolanaKeypair::new();
        let msg: Bytes = Bytes::from_static(b"hello solana");
        let a = solana_sign_message(kp.insecure_clone(), msg.clone());
        let b = solana_sign_prehash(kp.insecure_clone(), msg);
        assert_eq!(a, b);
    }

    #[test]
    fn pads_signatures_when_short() {
        let payer = SolanaKeypair::new();
        let recipient = SolanaKeypair::new();
        let mut tx =
            unsigned_transfer_tx(&payer.pubkey(), &recipient.pubkey(), 42);
        tx.signatures.clear();

        let signed =
            solana_sign_transaction(payer.insecure_clone(), tx).expect("sign");
        assert_eq!(signed.transaction.signatures.len(), 1);
        assert_eq!(signed.transaction.signatures[0], signed.signature);
    }
}
