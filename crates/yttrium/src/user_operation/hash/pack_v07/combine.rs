use alloy::primitives::{Uint, B256};
use std::str::FromStr;

pub fn combine_and_trim_first_16_bytes(
    items: Vec<Uint<256, 4>>,
) -> eyre::Result<B256> {
    let items_bytes_hex = items
        .iter()
        .map(|item| item.to_be_bytes_vec()[16..].to_vec())
        .map(hex::encode)
        .collect::<Vec<String>>();
    println!("items_bytes_hex: {:?}", items_bytes_hex);

    let combined = items_bytes_hex.join("");
    println!("combined: {:?}", combined);

    let result = B256::from_str(&combined)?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combine_and_trim_first_16_bytes() -> eyre::Result<()> {
        let expected_result = B256::from_str(
            "0000000000000000000000000000000100000000000000000000000000000002",
        )?;
        let items = vec![Uint::<256, 4>::from(1), Uint::<256, 4>::from(2)];
        let result = combine_and_trim_first_16_bytes(items)?;
        println!("result: {:?}", result);
        eyre::ensure!(
            result == expected_result,
            "result should be {}",
            expected_result
        );
        Ok(())
    }
}
