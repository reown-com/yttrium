use {
    crate::user_operation::UserOperationV07,
    alloy::primitives::{B256, keccak256},
};

pub fn get_hashed_call_data(user_operation: &UserOperationV07) -> B256 {
    keccak256(&user_operation.call_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_hashed_call_data() {
        let expected_hashed_call_data_hex = "0x0a8139e8d993db78f1d6b8682c7dcf9d4ef0b49b8bf883dc0a22a45b7aa7da2c";
        let user_operation = UserOperationV07::mock();
        let hashed_call_data = get_hashed_call_data(&user_operation);
        assert_eq!(hashed_call_data.to_string(), expected_hashed_call_data_hex,);
    }
}
