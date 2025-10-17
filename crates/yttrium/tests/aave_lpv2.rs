use num_bigint::BigUint;
use tiny_keccak::{Hasher, Keccak};
use yttrium::clear_signing::{format, DisplayItem, EngineError};
use yttrium::descriptors::aave::AAVE_LPV2;

const LENDING_POOL_MAINNET: &str = "0x7d2768dE32b0b80b7a3454c06BdAc94A69DDc7A9";
const USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
const DAI: &str = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
const ON_BEHALF_OF: &str = "0x1111111111111111111111111111111111111111";

#[test]
fn deposit_formats_usdc_amount() {
    let calldata = build_calldata(
        selector("deposit(address,uint256,address,uint16)"),
        &[
            address_word(USDC),
            uint_word_from_u128(1_000_000_000),
            address_word(ON_BEHALF_OF),
            uint_word_from_u128(0),
        ],
    );

    let model = format(AAVE_LPV2, 1, LENDING_POOL_MAINNET, &calldata).unwrap();

    assert_eq!(model.intent, "Supply");
    assert!(
        model.warnings.is_empty(),
        "expected no warnings, got {:?}",
        model.warnings
    );
    assert_eq!(model.items.len(), 2);
    assert_eq!(
        model.items[0],
        DisplayItem {
            label: "Amount to supply".to_string(),
            value: "1,000 USDC".to_string()
        }
    );
    assert_eq!(model.items[1].label, "Collateral recipient");
    assert_eq!(
        model.items[1].value.to_ascii_lowercase(),
        ON_BEHALF_OF.to_ascii_lowercase()
    );
}

#[test]
fn repay_all_uses_message() {
    let max = BigUint::parse_bytes(
        b"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        16,
    )
    .expect("max constant");
    let calldata = build_calldata(
        selector("repay(address,uint256,uint256,address)"),
        &[
            address_word(DAI),
            uint_word_from_biguint(&max),
            uint_word_from_u128(2),
            address_word(ON_BEHALF_OF),
        ],
    );

    let model = format(AAVE_LPV2, 1, LENDING_POOL_MAINNET, &calldata).unwrap();

    assert_eq!(model.intent, "Repay loan");
    assert_eq!(
        model.items.first(),
        Some(&DisplayItem {
            label: "Amount to repay".to_string(),
            value: "All".to_string()
        })
    );
}

#[test]
fn missing_token_errors() {
    let calldata = build_calldata(
        selector("deposit(address,uint256,address,uint16)"),
        &[
            address_word("0x000000000000000000000000000000000000dead"),
            uint_word_from_u128(1_000_000_000),
            address_word(ON_BEHALF_OF),
            uint_word_from_u128(0),
        ],
    );

    let err =
        format(AAVE_LPV2, 1, LENDING_POOL_MAINNET, &calldata).unwrap_err();

    assert!(matches!(err, EngineError::TokenRegistry(_)));
}

fn build_calldata(selector: [u8; 4], args: &[[u8; 32]]) -> Vec<u8> {
    let mut data = Vec::with_capacity(4 + args.len() * 32);
    data.extend_from_slice(&selector);
    for arg in args {
        data.extend_from_slice(arg);
    }
    data
}

fn selector(signature: &str) -> [u8; 4] {
    let mut hasher = Keccak::v256();
    hasher.update(signature.as_bytes());
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    [output[0], output[1], output[2], output[3]]
}

fn uint_word_from_u128(value: u128) -> [u8; 32] {
    uint_word_from_biguint(&BigUint::from(value))
}

fn uint_word_from_biguint(value: &BigUint) -> [u8; 32] {
    let mut word = [0u8; 32];
    let bytes = value.to_bytes_be();
    let start = 32 - bytes.len();
    word[start..].copy_from_slice(&bytes);
    word
}

fn address_word(address: &str) -> [u8; 32] {
    let mut word = [0u8; 32];
    let addr = address.trim_start_matches("0x");
    let bytes = hex::decode(addr).expect("valid address hex");
    word[12..].copy_from_slice(&bytes);
    word
}
