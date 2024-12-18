use {
    crate::{
        chain_abstraction::{amount::Amount, api::prepare::FundingMetadata},
        smart_accounts::{account_address::AccountAddress, safe::SafeOp},
    },
    alloy::{
        dyn_abi::Eip712Domain,
        primitives::{
            aliases::U48, Address, Bytes, PrimitiveSignature, Uint, B256, U128,
            U256, U64,
        },
    },
};

// TODO use https://mozilla.github.io/uniffi-rs/next/udl/remote_ext_types.html#remote-types when it's available

uniffi::custom_type!(Address, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

uniffi::custom_type!(AccountAddress, String, {
    try_lift: |val| Ok(val.parse::<Address>()?.into()),
    lower: |obj| obj.to_string(),
});

uniffi::custom_type!(PrimitiveSignature, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| format!("0x{}", hex::encode(obj.as_bytes())),
});

uniffi::custom_type!(SafeOp, String, {
    try_lift: |_val| unimplemented!("Does not support lifting SafeOp"),
    lower: |_obj| "".to_owned(),
});

uniffi::custom_type!(Eip712Domain, String, {
    try_lift: |_val| unimplemented!("Does not support lifting Eip712Domain"),
    lower: |_obj| "".to_owned(),
});

fn uint_to_hex<const BITS: usize, const LIMBS: usize>(
    obj: Uint<BITS, LIMBS>,
) -> String {
    format!("0x{obj:x}")
}

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
