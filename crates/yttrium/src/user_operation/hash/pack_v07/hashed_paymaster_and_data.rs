use crate::user_operation::UserOperationV07;
use alloy::primitives::{keccak256, Bytes, Uint, B256, U256};
use std::str::FromStr;

fn trim_first_16_bytes_or_default(item: Option<Uint<256, 4>>) -> String {
    let item = item.unwrap_or(U256::from(0));

    let item_bytes_vec: Vec<u8> = item.to_be_bytes_vec()[16..].to_vec();

    let item_hex = hex::encode(item_bytes_vec);

    item_hex
}

fn get_data(user_operation: &UserOperationV07) -> eyre::Result<Bytes> {
    let uo: UserOperationV07 = user_operation.clone();

    let data = if let Some(paymaster) = uo.paymaster.clone() {
        println!("paymaster: {:?}", paymaster);

        let paymaster_hex = format!("{}", paymaster);

        let paymaster_verification_gas_limit_hex = {
            let verification_limit_hex = trim_first_16_bytes_or_default(
                uo.paymaster_verification_gas_limit,
            );
            verification_limit_hex
        };

        let paymaster_post_op_gas_limit_hex = {
            let post_limit_hex =
                trim_first_16_bytes_or_default(uo.paymaster_post_op_gas_limit);
            post_limit_hex
        };

        let paymaster_data = {
            let paymaster_data = uo.paymaster_data.clone().unwrap_or_default();

            let paymaster_data_hex =
                format!("{:?}", paymaster_data)[2..].to_string();
            println!("paymaster_data_hex: {:?}", paymaster_data_hex);

            paymaster_data_hex
        };

        let combined = format!(
            "{}{}{}{}",
            paymaster_hex,
            paymaster_verification_gas_limit_hex,
            paymaster_post_op_gas_limit_hex,
            paymaster_data
        );
        println!("combined: {:?}", combined);

        combined
    } else {
        "".to_string()
    };

    let data = data.strip_prefix("0x").unwrap();

    let bytes = Bytes::from_str(data)?;

    Ok(bytes)
}

pub fn get_hashed_paymaster_and_data(
    user_operation: &UserOperationV07,
) -> eyre::Result<B256> {
    let data = get_data(user_operation)?;
    println!("data: {:?}", data);

    let hashed = keccak256(data);
    println!("hashed: {:?}", hashed);

    Ok(hashed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_hashed_paymaster_and_data() -> eyre::Result<()> {
        let expected_hashed_paymaster_and_data_hex = "0xfc0dffa735c71f138a00eaaafa56834aebf784e3e446612810f3f325cfb8eda9";
        let user_operation = UserOperationV07::mock();
        let hashed_paymaster_and_data =
            get_hashed_paymaster_and_data(&user_operation)?;
        println!("hashed_paymaster_and_data: {:?}", hashed_paymaster_and_data);
        eyre::ensure!(
            format!("{}", hashed_paymaster_and_data)
                == expected_hashed_paymaster_and_data_hex,
            "hashed_paymaster_and_data should be {}",
            expected_hashed_paymaster_and_data_hex
        );
        Ok(())
    }
}
