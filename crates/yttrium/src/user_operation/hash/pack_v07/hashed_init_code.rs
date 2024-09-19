use crate::user_operation::UserOperationV07;
use alloy::primitives::{keccak256, B256};

pub fn get_hashed_init_code(
    user_operation: &UserOperationV07,
) -> eyre::Result<B256> {
    let uo = user_operation.clone();
    let value_vec = if let (Some(factory), Some(factory_data)) =
        (uo.factory, uo.factory_data.clone())
    {
        let factory_vec: Vec<u8> = factory.to_vec();
        let factory_data_vec: Vec<u8> = factory_data.into();
        let mut bytes_vec: Vec<u8> = vec![];
        bytes_vec.extend(factory_vec);
        bytes_vec.extend(factory_data_vec);
        bytes_vec
    } else {
        let bytes_vec: Vec<u8> = vec![];
        bytes_vec
    };

    let hashed_init_code = keccak256(value_vec);
    println!("hashed_init_code: {:?}", hashed_init_code);

    Ok(hashed_init_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_hashed_init_code() -> eyre::Result<()> {
        let expected_hashed_init_code_hex =
            "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470";
        let user_operation = UserOperationV07::mock();
        let hashed_init_code = get_hashed_init_code(&user_operation)?;
        println!("hashed_init_code: {:?}", hashed_init_code);
        eyre::ensure!(
            format!("{}", hashed_init_code) == expected_hashed_init_code_hex,
            "hashed_init_code should be {}",
            expected_hashed_init_code_hex
        );
        Ok(())
    }
}
