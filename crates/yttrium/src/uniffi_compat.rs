use {
    crate::{
        chain_abstraction::{amount::Amount, api::prepare::FundingMetadata},
        smart_accounts::account_address::AccountAddress,
    },
    alloy::{
        contract::Error as AlloyError,
        dyn_abi::Eip712Domain,
        primitives::{
            aliases::U48, Address, Bytes, PrimitiveSignature, Uint, B256, U128,
            U256, U64, U8,
        },
        rpc::types::{Authorization, TransactionReceipt},
        signers::local::PrivateKeySigner,
        transports::{self, TransportErrorKind},
    },
    alloy_provider::PendingTransactionError,
    eyre::Report as EyreError,
    relay_rpc::domain::ProjectId,
    reqwest::Url,
};

// TODO use https://mozilla.github.io/uniffi-rs/next/udl/remote_ext_types.html#remote-types when it's available

uniffi::custom_type!(Address, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});
uniffi::custom_type!(AccountAddress, Address, {
    try_lift: |val| Ok(val.into()),
    lower: |obj| obj.into(),
});

uniffi::custom_type!(PrivateKeySigner, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| hex::encode(obj.to_bytes()),
});

uniffi::custom_type!(PrimitiveSignature, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| format!("0x{}", hex::encode(obj.as_bytes())),
});

uniffi::custom_type!(Eip712Domain, String, {
    try_lift: |_val| unimplemented!("Does not support lifting Eip712Domain"),
    lower: |_obj| "Does not support lowering Eip712Domain".to_owned(),
});

fn uint_to_hex<const BITS: usize, const LIMBS: usize>(
    obj: Uint<BITS, LIMBS>,
) -> String {
    format!("0x{obj:x}")
}

uniffi::custom_type!(U8, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

uniffi::custom_type!(U48, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

uniffi::custom_type!(U64, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

uniffi::custom_type!(U128, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

uniffi::custom_type!(u128, U128, {
    try_lift: |val| Ok(val.to()),
    lower: |obj| U128::from(obj),
});

uniffi::custom_type!(U256, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

uniffi::custom_type!(Bytes, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

uniffi::custom_type!(B256, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

uniffi::custom_type!(ProjectId, String, {
    try_lift: |val| Ok(val.into()),
    lower: |obj| obj.to_string(),
});

uniffi::custom_type!(Url, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

pub type RpcError = transports::RpcError<TransportErrorKind>;

uniffi::custom_type!(RpcError, String, {
    try_lift: |_val| unimplemented!("Does not support lifting RpcError"),
    lower: |obj| obj.to_string(),
});
uniffi::custom_type!(EyreError, String, {
    try_lift: |_val| unimplemented!("Does not support lifting EyreError"),
    lower: |obj| obj.to_string(),
});
uniffi::custom_type!(AlloyError, String, {
    try_lift: |_val| unimplemented!("Does not support lifting AlloyError"),
    lower: |obj| obj.to_string(),
});
uniffi::custom_type!(TransactionReceipt, String, {
    try_lift: |_val| unimplemented!("Does not support lifting TransactionReceipt"),
    lower: |obj| serde_json::to_string(&obj).unwrap(),
});
uniffi::custom_type!(PendingTransactionError, String, {
    try_lift: |_val| unimplemented!("Does not support lifting PendingTransactionError"),
    lower: |obj| obj.to_string(),
});

// uniffi::custom_type!(Unit, u8, {
//     try_lift: |val| Ok(Unit::new(val).expect("Unit must be less than 77")),
//     lower: |obj| obj.get(),
// });

#[uniffi::export]
fn funding_metadata_to_amount(value: FundingMetadata) -> Amount {
    value.to_amount()
}

#[uniffi::export]
fn funding_metadata_to_bridging_fee_amount(value: FundingMetadata) -> Amount {
    value.to_bridging_fee_amount()
}

uniffi::custom_type!(Authorization, FfiAuthorization, {
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

#[cfg(test)]
mod tests {
    use {
        super::*,
        alloy::primitives::{address, bytes},
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
}
