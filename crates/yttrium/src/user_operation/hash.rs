use {
    crate::user_operation::{
        user_operation_hash::UserOperationHash, UserOperationV07,
    },
    alloy::{
        primitives::{keccak256, Address, Bytes, B256, U256},
        sol_types::SolValue,
    },
};

pub mod pack_v07;

pub fn get_user_operation_hash_v07(
    user_operation: &UserOperationV07,
    entry_point: &Address,
    chain_id: u64,
) -> UserOperationHash {
    let packed_user_operation = {
        let packed = pack_v07::pack_user_operation_v07(user_operation);
        keccak256(packed)
    };

    let chain_id = U256::from(chain_id);

    let values = (packed_user_operation, entry_point, chain_id);
    let abi_encoded = values.abi_encode();
    assert_eq!(values.sol_name(), "(bytes32,address,uint256)");

    let encoded: Bytes = abi_encoded.into();
    let hash_bytes = keccak256(encoded);
    let hash = B256::from_slice(hash_bytes.as_slice());
    UserOperationHash(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_user_operation_hash_v07() {
        let expected_hash = "0xa1ea19d934f05fc2d725f2be8452ad7e2f29d9747674045ea366a320b782411d";
        let user_operation = UserOperationV07::mock();
        let entry_point = "0x0000000071727De22E5E9d8BAf0edAc6f37da032"
            .parse::<Address>()
            .unwrap();
        let chain_id = 11155111;
        let hash = get_user_operation_hash_v07(
            &user_operation,
            &entry_point,
            chain_id,
        );
        assert_eq!(hash.0.to_string(), expected_hash);
    }
}
