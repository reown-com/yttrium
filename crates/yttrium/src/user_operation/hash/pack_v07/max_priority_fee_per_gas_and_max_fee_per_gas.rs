use crate::user_operation::UserOperationV07;
use alloy::primitives::B256;

pub fn get_max_priority_fee_per_gas_and_max_fee_per_gas(
    user_operation: &UserOperationV07,
) -> eyre::Result<B256> {
    let values = vec![
        user_operation.max_priority_fee_per_gas,
        user_operation.max_fee_per_gas,
    ];
    let combined = super::combine::combine_and_trim_first_16_bytes(values)?;
    Ok(combined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_max_priority_fee_per_gas_and_max_fee_per_gas(
    ) -> eyre::Result<()> {
        let expected_max_priority_fee_per_gas_and_max_fee_per_gas_hex = "0x00000000000000000000000043d4ca3500000000000000000000000417bbd4f1";
        let user_operation = UserOperationV07::mock();
        let max_priority_fee_per_gas_and_max_fee_per_gas =
            get_max_priority_fee_per_gas_and_max_fee_per_gas(&user_operation)?;
        println!(
            "max_priority_fee_per_gas_and_max_fee_per_gas: {:?}",
            max_priority_fee_per_gas_and_max_fee_per_gas
        );
        eyre::ensure!(
            format!("{}", max_priority_fee_per_gas_and_max_fee_per_gas)
                == expected_max_priority_fee_per_gas_and_max_fee_per_gas_hex,
            "max_priority_fee_per_gas_and_max_fee_per_gas should be {}",
            expected_max_priority_fee_per_gas_and_max_fee_per_gas_hex
        );
        Ok(())
    }
}
