use alloy::primitives::{Address, Bytes, B256, U256, U64};

// TODO use https://mozilla.github.io/uniffi-rs/next/udl/remote_ext_types.html#remote-types when it's available

uniffi::custom_type!(Address, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

uniffi::custom_type!(U256, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

uniffi::custom_type!(U64, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

uniffi::custom_type!(Bytes, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

uniffi::custom_type!(B256, String, {
    try_lift: |val| Ok(val.parse()?),
    lower: |obj| obj.to_string(),
});

use erc6492::RpcError;
uniffi::custom_type!(RpcError, String, {
    try_lift: |_val| unimplemented!(),
    lower: |obj| obj.to_string(),
});
