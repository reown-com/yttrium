use crate::user_operation::UserOperationV07;
use alloy::sol_types::SolValue;

pub mod combine;
pub mod hashed_call_data;
pub mod hashed_init_code;
pub mod hashed_paymaster_and_data;
pub mod max_priority_fee_per_gas_and_max_fee_per_gas;
pub mod verificaction_gas_limit_and_call_gas_limit;

pub fn pack_user_operation_v07(user_operation: &UserOperationV07) -> Vec<u8> {
    let hashed_init_code =
        hashed_init_code::get_hashed_init_code(user_operation);

    let hashed_call_data =
        hashed_call_data::get_hashed_call_data(user_operation);

    let hashed_paymaster_and_data =
        hashed_paymaster_and_data::get_hashed_paymaster_and_data(
            user_operation,
        );

    let verificaction_gas_limit_and_call_gas_limit_item =
        verificaction_gas_limit_and_call_gas_limit::get_verificaction_gas_limit_and_call_gas_limit(user_operation);

    let max_priority_fee_per_gas_and_max_fee_per_gas_item =
        max_priority_fee_per_gas_and_max_fee_per_gas::get_max_priority_fee_per_gas_and_max_fee_per_gas(user_operation);

    let items = (
        user_operation.sender.to_address(),
        user_operation.nonce,
        hashed_init_code,
        hashed_call_data,
        verificaction_gas_limit_and_call_gas_limit_item,
        user_operation.pre_verification_gas,
        max_priority_fee_per_gas_and_max_fee_per_gas_item,
        hashed_paymaster_and_data,
    );

    items.abi_encode()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_user_operation_v07() {
        let expected_packed_user_operation_hex = "0x000000000000000000000000a3abdc7f6334cd3ee466a115f30522377787c0240000000000000000000000000000000000000000000000000000000000000010c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a4700a8139e8d993db78f1d6b8682c7dcf9d4ef0b49b8bf883dc0a22a45b7aa7da2c00000000000000000000000000010b2500000000000000000000000000013880000000000000000000000000000000000000000000000000000000000000d9a900000000000000000000000043d4ca3500000000000000000000000417bbd4f1fc0dffa735c71f138a00eaaafa56834aebf784e3e446612810f3f325cfb8eda9";

        let user_operation = UserOperationV07::mock();
        let packed_user_operation = pack_user_operation_v07(&user_operation);

        let packed_user_operation_hex = hex::encode(&packed_user_operation);

        assert_eq!(
            format!("0x{}", packed_user_operation_hex),
            expected_packed_user_operation_hex,
        );
    }
}
