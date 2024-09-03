use super::combine::combine_and_trim_first_16_bytes;
use crate::user_operation::UserOperationV07;
use alloy::primitives::B256;

pub fn get_verificaction_gas_limit_and_call_gas_limit(
    user_operation: &UserOperationV07,
) -> eyre::Result<B256> {
    let values = vec![
        user_operation.verification_gas_limit,
        user_operation.call_gas_limit,
    ];
    let combined = combine_and_trim_first_16_bytes(values)?;
    Ok(combined)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_verificaction_gas_limit_and_call_gas_limit() -> eyre::Result<()>
    {
        let expected_verification_gas_limit_and_call_gas_limit_hex =
            "0x00000000000000000000000000010b2500000000000000000000000000013880";
        let user_operation = UserOperationV07::mock();
        let verification_gas_limit_and_call_gas_limit =
            get_verificaction_gas_limit_and_call_gas_limit(&user_operation)?;
        println!(
            "verification_gas_limit_and_call_gas_limit: {:?}",
            verification_gas_limit_and_call_gas_limit
        );
        eyre::ensure!(
            format!("{}", verification_gas_limit_and_call_gas_limit)
                == expected_verification_gas_limit_and_call_gas_limit_hex,
            "verification_gas_limit_and_call_gas_limit should be {}",
            expected_verification_gas_limit_and_call_gas_limit_hex
        );
        Ok(())
    }
}
