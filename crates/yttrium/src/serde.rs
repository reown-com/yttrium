pub mod duration_millis {
    use {
        serde::{de, ser, Deserialize},
        std::time::Duration,
    };

    pub fn serialize<S>(dt: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_u128(dt.as_millis())
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Duration, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        u64::deserialize(d).map(Duration::from_millis)
    }
}

pub mod systemtime_millis {
    use {
        super::duration_millis,
        serde::{de, ser},
        std::time::{SystemTime, UNIX_EPOCH},
    };

    pub fn serialize<S>(
        dt: &SystemTime,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let dt = dt.duration_since(UNIX_EPOCH).unwrap_or_default();
        duration_millis::serialize(&dt, serializer)
    }

    pub fn deserialize<'de, D>(d: D) -> Result<SystemTime, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        duration_millis::deserialize(d).map(|dt| UNIX_EPOCH + dt)
    }
}

pub mod option_duration_millis {
    use {
        serde::{de, ser, Deserialize},
        std::time::Duration,
    };

    pub fn serialize<S>(
        dt: &Option<Duration>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        if let Some(dt) = dt {
            serializer.serialize_some(&dt.as_millis())
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Option<Duration>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Option::<u64>::deserialize(d).map(|o| o.map(Duration::from_millis))
    }
}
