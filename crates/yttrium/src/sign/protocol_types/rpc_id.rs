use {
    rand::Rng,
    serde::{Deserialize, Serialize},
    std::fmt::Display,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProtocolRpcId(pub(self) u64);

impl ProtocolRpcId {
    pub fn generate() -> ProtocolRpcId {
        // overflows u64 at year 10889
        // overflows Number.MAX_SAFE_INTEGER at year 3084
        let time = crate::time::SystemTime::now()
            .duration_since(crate::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let mut rng = rand::thread_rng();
        let random = rng.gen_range(0..=u8::MAX) as u64;
        let id = (time << 8) | random;
        Self(id)
    }

    pub fn into_value(self) -> u64 {
        self.0
    }
}

impl Display for ProtocolRpcId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "uniffi")]
uniffi::custom_type!(ProtocolRpcId, u64, {
    remote,
    try_lift: |val| Ok(ProtocolRpcId(val)),
    lower: |obj| obj.into_value(),
});
