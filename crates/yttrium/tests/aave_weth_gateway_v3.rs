use num_bigint::BigUint;
use tiny_keccak::{Hasher, Keccak};
use yttrium::clear_signing::{format, DisplayItem};
use yttrium::descriptors::aave::AAVE_WETH_GATEWAY_V3;
use yttrium::resolver::local_resolver::{LocalResolver, ResolverClient};

const WETH_GATEWAY_MAINNET: &str = "0xd01607c3C5eCABa394D8be377a08590149325722";
const LENDING_POOL_V3_MAINNET: &str =
    "0x87870Bca3F3fD6335C3F4ce8392D69350B4fA4E2";
const RECIPIENT: &str = "0x2222222222222222222222222222222222222222";

#[test]
fn withdraw_eth_with_permit_formats_amount() {
    let calldata = build_calldata(
        selector("withdrawETHWithPermit(address,uint256,address,uint256,uint8,bytes32,bytes32)"),
        &[
            address_word(LENDING_POOL_V3_MAINNET),
            uint_word_from_u128(500_000_000_000_000_000),
            address_word(RECIPIENT),
            uint_word_from_u128(1_725_328_800), // arbitrary deadline
            uint_word_from_u128(27),
            bytes32_word(
                "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ),
            bytes32_word(
                "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            ),
        ],
    );

    let model =
        format(AAVE_WETH_GATEWAY_V3, 1, WETH_GATEWAY_MAINNET, &calldata)
            .unwrap();

    assert_eq!(model.intent, "Withdraw");
    assert!(
        model.warnings.is_empty(),
        "expected no warnings, got {:?}",
        model.warnings
    );
    assert_eq!(
        model.items,
        vec![
            DisplayItem {
                label: "Amount to withdraw".to_string(),
                value: "0.5 ETH".to_string()
            },
            DisplayItem {
                label: "To recipient".to_string(),
                value: RECIPIENT.to_string()
            }
        ]
    );
}

#[test]
fn deposit_eth_reports_missing_value() {
    let calldata = build_calldata(
        selector("depositETH(address,address,uint16)"),
        &[
            address_word(LENDING_POOL_V3_MAINNET),
            address_word(RECIPIENT),
            uint_word_from_u128(0),
        ],
    );

    let model =
        format(AAVE_WETH_GATEWAY_V3, 1, WETH_GATEWAY_MAINNET, &calldata)
            .unwrap();

    assert!(
        model.warnings.iter().any(|warning| warning.contains("@.value")),
        "expected warning about missing @.value, got {:?}",
        model.warnings
    );
    assert_eq!(model.items.len(), 1);
    assert_eq!(model.items[0].label, "Collateral recipient");
    assert_eq!(
        model.items[0].value.to_ascii_lowercase(),
        RECIPIENT.to_ascii_lowercase()
    );
}

#[test]
fn resolver_finds_descriptor_and_formats() {
    let resolver = LocalResolver::new();
    let caip10 = "eip155:1:0xd01607c3c5ecaba394d8be377a08590149325722";
    let descriptor =
        resolver.resolve_descriptor(caip10).expect("descriptor must resolve");
    assert_eq!(descriptor, AAVE_WETH_GATEWAY_V3);

    let calldata = build_calldata(
        selector("repayETH(address,uint256,address)"),
        &[
            address_word(LENDING_POOL_V3_MAINNET),
            uint_word_from_u128(1_000_000_000_000_000),
            address_word(RECIPIENT),
        ],
    );

    let model = format(descriptor, 1, WETH_GATEWAY_MAINNET, &calldata)
        .expect("format succeeds");
    assert_eq!(model.intent, "Repay loan");
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

fn bytes32_word(value: &str) -> [u8; 32] {
    let mut word = [0u8; 32];
    let stripped = value.trim_start_matches("0x");
    let bytes = hex::decode(stripped).expect("valid bytes32 hex");
    word.copy_from_slice(&bytes);
    word
}
