use {alloy::primitives::utils::Unit, serde::Deserialize};

pub fn deserialize_unit<'de, D>(deserializer: D) -> Result<Unit, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Unit::new(u8::deserialize(deserializer)?)
        .ok_or(serde::de::Error::custom("Unit must be less than 77"))
}

pub fn serialize_unit<S>(unit: &Unit, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_u8(unit.get())
}
