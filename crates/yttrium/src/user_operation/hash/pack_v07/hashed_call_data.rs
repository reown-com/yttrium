use crate::user_operation::UserOperationV07;
use alloy::primitives::{keccak256, B256};

pub fn get_hashed_call_data(
    user_operation: &UserOperationV07,
) -> eyre::Result<B256> {
    let hashed_call_data = {
        let call_data = user_operation.clone().call_data;
        keccak256(call_data)
    };
    let hashed_call_data_hex = hex::encode(hashed_call_data.clone());
    println!("hashed_call_data_hex: {:?}", hashed_call_data_hex);
    Ok(hashed_call_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_hashed_call_data() -> eyre::Result<()> {
        let expected_hashed_call_data_hex =
            "0x0a8139e8d993db78f1d6b8682c7dcf9d4ef0b49b8bf883dc0a22a45b7aa7da2c";
        let user_operation = UserOperationV07::mock();
        let hashed_call_data = get_hashed_call_data(&user_operation)?;
        println!("hashed_call_data: {:?}", hashed_call_data);
        eyre::ensure!(
            format!("{}", hashed_call_data) == expected_hashed_call_data_hex,
            "hashed_call_data should be {}",
            expected_hashed_call_data_hex
        );
        Ok(())
    }
}
