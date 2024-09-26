use crate::user_operation::UserOperationV07;
use alloy::primitives::B256;

pub fn get_max_priority_fee_per_gas_and_max_fee_per_gas(
    user_operation: &UserOperationV07,
) -> B256 {
    super::combine::combine_and_trim_first_16_bytes(
        user_operation.max_priority_fee_per_gas,
        user_operation.max_fee_per_gas,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::fixed_bytes;

    #[test]
    fn test_get_max_priority_fee_per_gas_and_max_fee_per_gas() {
        let expected_max_priority_fee_per_gas_and_max_fee_per_gas = fixed_bytes!(
            "00000000000000000000000043d4ca3500000000000000000000000417bbd4f1"
        );
        let user_operation = UserOperationV07::mock();
        let max_priority_fee_per_gas_and_max_fee_per_gas =
            get_max_priority_fee_per_gas_and_max_fee_per_gas(&user_operation);
        assert_eq!(
            max_priority_fee_per_gas_and_max_fee_per_gas,
            expected_max_priority_fee_per_gas_and_max_fee_per_gas,
        );
    }
}
