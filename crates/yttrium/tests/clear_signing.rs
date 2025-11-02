use num_bigint::BigUint;
use serde_json::json;
use tiny_keccak::{Hasher, Keccak};
use yttrium::clear_signing::{
    format_typed_data, format_with_value, DisplayItem, TypedData,
};

const USDT_MAINNET: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
const UNISWAP_V3_ROUTER: &str = "0xE592427A0AEce92De3Edee1F18E0157C05861564";
const WETH_MAINNET: &str = "0xC02aaA39b223FE8D0A0E5C4F27eAD9083C756Cc2";
const TEST_ROUTER: &str = "0xF00D000000000000000000000000000000000123";

#[test]
fn approve_usdt_spender() {
    let selector = selector("approve(address,uint256)");
    let spender = address_word(UNISWAP_V3_ROUTER);
    let amount = uint_word_u128(1_000_000_000_000u128);
    let calldata = build_calldata(selector, &[spender, amount]);

    let model = format_with_value(1, USDT_MAINNET, None, &calldata)
        .expect("format succeeds");

    assert_eq!(model.intent, "Approve USDT spending");
    assert_eq!(
        model.items,
        vec![
            DisplayItem {
                label: "Spender".to_string(),
                value: "Uniswap V3 Router".to_string(),
            },
            DisplayItem {
                label: "Amount".to_string(),
                value: "1,000,000 USDT".to_string(),
            },
        ]
    );
    assert!(model.warnings.is_empty());
    assert!(model.raw.is_none());
}

#[test]
fn swap_usdc_to_weth_exact_input_single() {
    let selector =
        selector("exactInputSingle((address,address,uint24,address,uint256,uint256,uint160))");

    let params = [
        address_word("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
        address_word(WETH_MAINNET),
        uint_word_u32(3_000u32),
        address_word("0x1234567890abcdef1234567890abcdef12345678"),
        uint_word_u128(1_000_000_000_000u128),
        uint_word_u128(1_000_000_000_000_000_000u128),
        uint_word_u128(0u128),
    ];

    let calldata = build_calldata(selector, &params);

    let model = format_with_value(1, UNISWAP_V3_ROUTER, None, &calldata)
        .expect("format succeeds");

    assert_eq!(model.intent, "Swap tokens");
    assert_eq!(
        model.items,
        vec![
            DisplayItem {
                label: "Amount in".to_string(),
                value: "1,000,000 USDC".to_string(),
            },
            DisplayItem {
                label: "Minimum received".to_string(),
                value: "1 WETH".to_string(),
            },
            DisplayItem {
                label: "Recipient".to_string(),
                value: "0x1234567890AbcdEF1234567890aBcdef12345678".to_string(),
            },
        ]
    );
    assert!(model.warnings.is_empty());
    assert!(model.raw.is_none());
}

#[test]
fn deposit_weth_uses_call_value() {
    let selector = selector("deposit()");
    let calldata = build_calldata(selector, &[]);
    let value = uint_word_u128(500_000_000_000_000_000u128);

    let model = format_with_value(1, WETH_MAINNET, Some(&value), &calldata)
        .expect("format succeeds");

    assert_eq!(model.intent, "Wrap ETH into WETH");
    assert_eq!(
        model.items,
        vec![DisplayItem {
            label: "Amount".to_string(),
            value: "0.5 WETH".to_string(),
        }]
    );
    assert!(model.warnings.is_empty());
    assert!(model.raw.is_none());
}

#[test]
fn unknown_selector_returns_fallback() {
    let selector = [0u8, 1u8, 2u8, 3u8];
    let calldata = build_calldata(selector, &[uint_word_u128(42u128)]);

    let model = format_with_value(1, USDT_MAINNET, None, &calldata)
        .expect("format succeeds");

    assert_eq!(model.intent, "Unknown transaction");
    assert!(model.items.is_empty());
    assert!(model.warnings.iter().any(|w| w.contains("No ABI match")));
    assert!(model.raw.is_some());
}

#[test]
fn includes_descriptor_merges_display() {
    let selector = selector("setAmount(uint256)");
    let amount = uint_word_u128(42u128);
    let calldata = build_calldata(selector, &[amount]);

    let model = format_with_value(1, TEST_ROUTER, None, &calldata)
        .expect("format succeeds");

    assert_eq!(model.intent, "Set router amount");
    assert_eq!(
        model.items,
        vec![DisplayItem {
            label: "Amount".to_string(),
            value: "42".to_string()
        }]
    );
    assert!(model.warnings.is_empty());
    assert!(model.raw.is_none());
}

#[test]
fn eip712_limit_order_formats_tokens() {
    let typed_data_json = json!({
        "types": {
            "EIP712Domain": [
                {"name": "name", "type": "string"},
                {"name": "version", "type": "string"},
                {"name": "chainId", "type": "uint256"},
                {"name": "verifyingContract", "type": "address"}
            ],
            "OrderStructure": [
                {"name": "salt", "type": "uint256"},
                {"name": "maker", "type": "address"},
                {"name": "receiver", "type": "address"},
                {"name": "makerAsset", "type": "address"},
                {"name": "takerAsset", "type": "address"},
                {"name": "makingAmount", "type": "uint256"},
                {"name": "takingAmount", "type": "uint256"},
                {"name": "makerTraits", "type": "uint256"}
            ]
        },
        "primaryType": "OrderStructure",
        "domain": {
            "name": "1inch",
            "version": "1",
            "chainId": 1,
            "verifyingContract": "0x119c71d3BbAc22029622CBaEC24854d3D32D2828"
        },
        "message": {
            "salt": "1",
            "maker": "0xabc0000000000000000000000000000000000001",
            "receiver": "0xabc0000000000000000000000000000000000002",
            "makerAsset": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "takerAsset": "0xC02aaA39b223FE8D0A0E5C4F27eAD9083C756Cc2",
            "makingAmount": "1000000",
            "takingAmount": "1000000000000000000",
            "makerTraits": "0"
        }
    });

    let typed: TypedData = serde_json::from_value(typed_data_json)
        .expect("typed data should parse");

    let model = format_typed_data(&typed).expect("format succeeds");

    assert_eq!(model.intent, "1inch Order");
    assert_eq!(
        model.items,
        vec![
            DisplayItem {
                label: "From".to_string(),
                value: "0xabc0000000000000000000000000000000000001".to_string(),
            },
            DisplayItem {
                label: "Send".to_string(),
                value: "1 USDC".to_string(),
            },
            DisplayItem {
                label: "Receive minimum".to_string(),
                value: "1 WETH".to_string(),
            },
            DisplayItem {
                label: "To".to_string(),
                value: "0xabc0000000000000000000000000000000000002".to_string(),
            },
        ]
    );
    assert!(model.warnings.is_empty());
    assert!(model.raw.is_none());
}

fn build_calldata(selector: [u8; 4], words: &[[u8; 32]]) -> Vec<u8> {
    let mut data = Vec::with_capacity(4 + words.len() * 32);
    data.extend_from_slice(&selector);
    for word in words {
        data.extend_from_slice(word);
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

fn address_word(address: &str) -> [u8; 32] {
    let mut word = [0u8; 32];
    let cleaned = address.trim().trim_start_matches("0x");
    let bytes = hex::decode(cleaned).expect("valid address hex");
    assert_eq!(bytes.len(), 20, "address must be 20 bytes");
    word[12..].copy_from_slice(&bytes);
    word
}

fn uint_word_u32(value: u32) -> [u8; 32] {
    uint_word_biguint(BigUint::from(value))
}

fn uint_word_u128(value: u128) -> [u8; 32] {
    uint_word_biguint(BigUint::from(value))
}

fn uint_word_biguint(value: BigUint) -> [u8; 32] {
    let bytes = value.to_bytes_be();
    assert!(bytes.len() <= 32, "value must fit in 32 bytes");
    let mut word = [0u8; 32];
    word[32 - bytes.len()..].copy_from_slice(&bytes);
    word
}
