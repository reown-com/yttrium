use {
    relay_rpc::rpc::JSON_RPC_VERSION,
    serde::{Deserialize, Serialize},
    std::{fmt::Display, sync::Arc},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JsonRpcVersion(Arc<str>);

impl JsonRpcVersion {
    pub fn version_2() -> Self {
        Self(JSON_RPC_VERSION.clone())
    }
}

impl Display for JsonRpcVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Can't just use an Arc<str>. See https://github.com/mozilla/uniffi-rs/issues/2727
#[cfg(feature = "uniffi")]
uniffi::custom_type!(JsonRpcVersion, String, {
    remote,
    try_lift: |val| Ok(JsonRpcVersion(val.into())),
    lower: |obj| obj.to_string(),
});
