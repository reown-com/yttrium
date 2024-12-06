use {
    crate::chain_abstraction::{amount::Amount, api::route::FundingMetadata},
    alloy::primitives::{
        utils::Unit, Address, Bytes, Uint, B256, U128, U256, U64,
    },
};

// TODO use https://mozilla.github.io/uniffi-rs/next/udl/remote_ext_types.html#remote-types when it's available

uniffi::custom_type!(Address, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

fn uint_to_hex<const BITS: usize, const LIMBS: usize>(
    obj: Uint<BITS, LIMBS>,
) -> String {
    format!("0x{obj:x}")
}

uniffi::custom_type!(U128, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

uniffi::custom_type!(U256, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| uint_to_hex(obj),
});

uniffi::custom_type!(U64, String, {
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

uniffi::custom_type!(Unit, u8, {
    try_lift: |val| Ok(Unit::new(val).expect("Unit must be less than 77")),
    lower: |obj| obj.get(),
});

#[uniffi::export]
fn funding_metadata_to_amount(value: FundingMetadata) -> Amount {
    value.to_amount()
}

#[uniffi::export]
fn funding_metadata_to_bridging_fee_amount(value: FundingMetadata) -> Amount {
    value.to_bridging_fee_amount()
}
