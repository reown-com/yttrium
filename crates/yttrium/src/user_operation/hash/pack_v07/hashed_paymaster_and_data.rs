use crate::user_operation::UserOperationV07;
use alloy::primitives::{keccak256, Bytes, B256};

fn get_data(user_operation: &UserOperationV07) -> Bytes {
    if let Some(paymaster) = user_operation.paymaster {
        let address = paymaster.into_iter();

        let paymaster_verification_gas_limit = user_operation
            .paymaster_verification_gas_limit
            .unwrap_or_default()
            .to_be_bytes_vec()
            .into_iter()
            .skip(16);

        let paymaster_post_op_gas_limit = user_operation
            .paymaster_post_op_gas_limit
            .unwrap_or_default()
            .to_be_bytes_vec()
            .into_iter()
            .skip(16);

        let paymaster_data = user_operation
            .paymaster_data
            .as_ref()
            .unwrap_or_default()
            .iter()
            .copied();

        address
            .chain(paymaster_verification_gas_limit)
            .chain(paymaster_post_op_gas_limit)
            .chain(paymaster_data)
            .collect()
    } else {
        Bytes::new()
    }
}

pub fn get_hashed_paymaster_and_data(
    user_operation: &UserOperationV07,
) -> B256 {
    let data = get_data(user_operation);
    keccak256(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_hashed_paymaster_and_data() {
        let expected_hashed_paymaster_and_data_hex = "0xfc0dffa735c71f138a00eaaafa56834aebf784e3e446612810f3f325cfb8eda9";
        let user_operation = UserOperationV07::mock();
        let hashed_paymaster_and_data =
            get_hashed_paymaster_and_data(&user_operation);
        assert_eq!(
            hashed_paymaster_and_data.to_string(),
            expected_hashed_paymaster_and_data_hex,
        );
    }
}
