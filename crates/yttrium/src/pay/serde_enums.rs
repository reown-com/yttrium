use {
    super::{CollectDataFieldType, PaymentStatus},
    serde::{de::Deserializer, ser::Serializer},
};

impl serde::Serialize for PaymentStatus {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(match self {
            Self::RequiresAction => "requires_action",
            Self::Processing => "processing",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Expired => "expired",
            Self::Unknown { value } => value,
        })
    }
}

impl<'de> serde::Deserialize<'de> for PaymentStatus {
    fn deserialize<D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        let s: String = serde::Deserialize::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "requires_action" => Self::RequiresAction,
            "processing" => Self::Processing,
            "succeeded" => Self::Succeeded,
            "failed" => Self::Failed,
            "expired" => Self::Expired,
            _ => Self::Unknown { value: s },
        })
    }
}

impl serde::Serialize for CollectDataFieldType {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(match self {
            Self::Text => "text",
            Self::Date => "date",
            Self::Checkbox => "checkbox",
            Self::Unknown { value } => value,
        })
    }
}

impl<'de> serde::Deserialize<'de> for CollectDataFieldType {
    fn deserialize<D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        let s: String = serde::Deserialize::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "text" => Self::Text,
            "date" => Self::Date,
            "checkbox" => Self::Checkbox,
            _ => Self::Unknown { value: s },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payment_status_serializes_known_variants() {
        assert_eq!(
            serde_json::to_string(&PaymentStatus::Succeeded).unwrap(),
            "\"succeeded\""
        );
        assert_eq!(
            serde_json::to_string(&PaymentStatus::RequiresAction).unwrap(),
            "\"requires_action\""
        );
    }

    #[test]
    fn payment_status_serializes_unknown_variant() {
        let status = PaymentStatus::Unknown { value: "new_status".to_string() };
        assert_eq!(serde_json::to_string(&status).unwrap(), "\"new_status\"");
    }

    #[test]
    fn payment_status_deserializes_known_variants() {
        assert_eq!(
            serde_json::from_str::<PaymentStatus>("\"succeeded\"").unwrap(),
            PaymentStatus::Succeeded
        );
        assert_eq!(
            serde_json::from_str::<PaymentStatus>("\"requires_action\"")
                .unwrap(),
            PaymentStatus::RequiresAction
        );
    }

    #[test]
    fn payment_status_deserializes_unknown_variant() {
        assert_eq!(
            serde_json::from_str::<PaymentStatus>("\"new_status\"").unwrap(),
            PaymentStatus::Unknown { value: "new_status".to_string() }
        );
    }

    #[test]
    fn payment_status_roundtrip() {
        for status in [
            PaymentStatus::RequiresAction,
            PaymentStatus::Processing,
            PaymentStatus::Succeeded,
            PaymentStatus::Failed,
            PaymentStatus::Expired,
            PaymentStatus::Unknown { value: "future_status".to_string() },
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let back: PaymentStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(back, status);
        }
    }

    #[test]
    fn collect_data_field_type_serializes_known_variants() {
        assert_eq!(
            serde_json::to_string(&CollectDataFieldType::Text).unwrap(),
            "\"text\""
        );
    }

    #[test]
    fn collect_data_field_type_serializes_unknown_variant() {
        let ft =
            CollectDataFieldType::Unknown { value: "dropdown".to_string() };
        assert_eq!(serde_json::to_string(&ft).unwrap(), "\"dropdown\"");
    }

    #[test]
    fn collect_data_field_type_roundtrip() {
        for ft in [
            CollectDataFieldType::Text,
            CollectDataFieldType::Date,
            CollectDataFieldType::Checkbox,
            CollectDataFieldType::Unknown { value: "dropdown".to_string() },
        ] {
            let json = serde_json::to_string(&ft).unwrap();
            let back: CollectDataFieldType =
                serde_json::from_str(&json).unwrap();
            assert_eq!(back, ft);
        }
    }
}
