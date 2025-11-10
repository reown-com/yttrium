use {
    num_bigint::BigUint,
    serde_json::json,
    tiny_keccak::{Hasher, Keccak},
    yttrium::clear_signing::{
        format_typed_data, format_with_value, DisplayItem, EngineError,
        TypedData,
    },
};

const USDT_MAINNET: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
const UNISWAP_V3_ROUTER: &str = "0xE592427A0AEce92De3Edee1F18E0157C05861564";
const WETH_MAINNET: &str = "0xC02aaA39b223FE8D0A0E5C4F27eAD9083C756Cc2";
const TEST_ROUTER: &str = "0xF00D000000000000000000000000000000000123";
const AAVE_LPV2_MAINNET: &str = "0x7d2768de32b0b80b7a3454c06bdac94a69ddc7a9";
const USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
const DAI: &str = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
const ON_BEHALF_OF: &str = "0x1111111111111111111111111111111111111111";
const UNIVERSAL_ROUTER_OPTIMISM: &str =
    "0x851116d9223fAbEd8e56C0e6B8AD0c31d98B3507";
const STAKEWEIGHT_OPTIMISM: &str = "0x521B4C065Bbdbe3E20B3727340730936912DfA46";
const STAKEWEIGHT_INCREASE_UNLOCK_TIME_CALLDATA: &str =
    "0x7c616fe6000000000000000000000000000000000000000000000000000000006945563d";
const UNIVERSAL_ROUTER_CALLDATA_HEX: &str = concat!(
    "3593564c00000000000000000000000000000000000000000000000000000000",
    "0000006000000000000000000000000000000000000000000000000000000000",
    "000000a000000000000000000000000000000000000000000000000000000000",
    "69087dd700000000000000000000000000000000000000000000000000000000",
    "0000000310060400000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "0000000030000000000000000000000000000000000000000000000000000000",
    "0000000600000000000000000000000000000000000000000000000000000000",
    "0000004400000000000000000000000000000000000000000000000000000000",
    "0000004c00000000000000000000000000000000000000000000000000000000",
    "0000003c00000000000000000000000000000000000000000000000000000000",
    "0000000400000000000000000000000000000000000000000000000000000000",
    "0000000800000000000000000000000000000000000000000000000000000000",
    "000000003070b0e0000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "0000000030000000000000000000000000000000000000000000000000000000",
    "0000000600000000000000000000000000000000000000000000000000000000",
    "0000002200000000000000000000000000000000000000000000000000000000",
    "0000002a00000000000000000000000000000000000000000000000000000000",
    "0000001a00000000000000000000000000000000000000000000000000000000",
    "0000000200000000000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "0000000800000000000000000000000000000000000000000000000000001e12",
    "64f50cc870000000000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "0000000010000000000000000000000000000000000000000000000000000000",
    "0000000200000000000000000000000000b2c639c533813f4aa9d7837caf6265",
    "3d097ff850000000000000000000000000000000000000000000000000000000",
    "0000001f40000000000000000000000000000000000000000000000000000000",
    "00000000a0000000000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "0000000a00000000000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "0000000600000000000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "0000000010000000000000000000000000000000000000000000000000000000",
    "0000000600000000000000000000000000b2c639c533813f4aa9d7837caf6265",
    "3d097ff850000000000000000000000000000000000000000000000000000000",
    "0000000020000000000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "0000000600000000000000000000000000b2c639c533813f4aa9d7837caf6265",
    "3d097ff850000000000000000000000007ffc3dbf3b2b50ff3a1d5523bc24bb5",
    "043837b140000000000000000000000000000000000000000000000000000000",
    "0000000190000000000000000000000000000000000000000000000000000000",
    "0000000600000000000000000000000000b2c639c533813f4aa9d7837caf6265",
    "3d097ff85000000000000000000000000bf01daf454dce008d3e2bfd47d5e186",
    "f714772530000000000000000000000000000000000000000000000000000000",
    "000000000000000000000000000000000000000000000000000000001d2ec50c",
);

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
fn aave_deposit_formats_usdc_amount() {
    let calldata = build_calldata(
        selector("deposit(address,uint256,address,uint16)"),
        &[
            address_word(USDC),
            uint_word_u128(1_000_000_000),
            address_word(ON_BEHALF_OF),
            uint_word_u128(0),
        ],
    );

    let model = format_with_value(1, AAVE_LPV2_MAINNET, None, &calldata)
        .expect("format succeeds");

    assert_eq!(model.intent, "Supply");
    assert!(model.warnings.is_empty());
    assert_eq!(model.items.len(), 2);
    assert_eq!(
        model.items[0],
        DisplayItem {
            label: "Amount to supply".to_string(),
            value: "1,000 USDC".to_string(),
        }
    );
    assert_eq!(model.items[1].label, "Collateral recipient".to_string());
    assert_eq!(
        model.items[1].value.to_ascii_lowercase(),
        ON_BEHALF_OF.to_ascii_lowercase()
    );
}

#[test]
fn aave_repay_all_uses_message() {
    let max = BigUint::parse_bytes(
        b"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
        16,
    )
    .expect("max constant");
    let calldata = build_calldata(
        selector("repay(address,uint256,uint256,address)"),
        &[
            address_word(DAI),
            uint_word_biguint(max.clone()),
            uint_word_u128(2),
            address_word(ON_BEHALF_OF),
        ],
    );

    let model = format_with_value(1, AAVE_LPV2_MAINNET, None, &calldata)
        .expect("format succeeds");

    assert_eq!(model.intent, "Repay loan");
    assert_eq!(
        model.items.first(),
        Some(&DisplayItem {
            label: "Amount to repay".to_string(),
            value: "All".to_string(),
        })
    );
}

#[test]
fn walletconnect_increase_unlock_time_renders_date() {
    let calldata = hex::decode(
        STAKEWEIGHT_INCREASE_UNLOCK_TIME_CALLDATA.trim_start_matches("0x"),
    )
    .expect("call data hex");

    let model = format_with_value(10, STAKEWEIGHT_OPTIMISM, None, &calldata)
        .expect("format succeeds");

    assert_eq!(model.intent, "Increase Unlock Time");
    assert_eq!(
        model.items,
        vec![DisplayItem {
            label: "New Unlock Time".to_string(),
            value: "2025-12-19 13:42:21 UTC".to_string(),
        }]
    );
    assert!(model.warnings.is_empty());
    assert!(model.raw.is_none());
}

#[test]
fn aave_missing_token_errors() {
    let calldata = build_calldata(
        selector("deposit(address,uint256,address,uint16)"),
        &[
            address_word("0x000000000000000000000000000000000000dead"),
            uint_word_u128(1_000_000_000),
            address_word(ON_BEHALF_OF),
            uint_word_u128(0),
        ],
    );

    let err = format_with_value(1, AAVE_LPV2_MAINNET, None, &calldata)
        .expect_err("token lookup should fail");

    assert!(matches!(err, EngineError::TokenRegistry(_)));
}

#[test]
fn aave_borrow_variable_on_optimism() {
    let calldata = hex::decode("a415bcad00000000000000000000000094b008aa00579c1307b0ef2c499ad98a8ce58e5800000000000000000000000000000000000000000000000000000000000f424000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000bf01daf454dce008d3e2bfd47d5e186f71477253").expect("valid hex");

    let model = format_with_value(
        10,
        "0x794a61358D6845594F94dc1DB02A252b5b4814aD",
        None,
        &calldata,
    )
    .expect("format succeeds");

    assert_eq!(model.intent, "Borrow");
    assert!(model.warnings.is_empty());
    assert_eq!(model.items.len(), 3);
    assert_eq!(
        model.items[0],
        DisplayItem {
            label: "Amount to borrow".to_string(),
            value: "1 USDT".to_string()
        }
    );
    assert_eq!(
        model.items[1],
        DisplayItem {
            label: "Interest Rate mode".to_string(),
            value: "variable".to_string()
        }
    );
    assert_eq!(model.items[2].label, "Debtor");
    assert_eq!(
        model.items[2].value.to_ascii_lowercase(),
        "0xbf01daf454dce008d3e2bfd47d5e186f71477253"
    );
}

#[test]
fn uniswap_universal_router_missing_descriptor() {
    let calldata =
        hex::decode(UNIVERSAL_ROUTER_CALLDATA_HEX).expect("valid hex");
    let call_value = uint_word_u128(
        u128::from_str_radix("1e1264f50cc87", 16).expect("value"),
    );

    let err = format_with_value(
        10,
        UNIVERSAL_ROUTER_OPTIMISM,
        Some(&call_value),
        &calldata,
    )
    .expect_err("descriptor lookup should fail");

    match err {
        EngineError::Resolver(message) => {
            assert!(
                message.contains("descriptor not found"),
                "unexpected resolver error: {message}"
            );
        }
        other => panic!("expected resolver error, got {other:?}"),
    }
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

#[test]
fn usdt_approve_all_displays_all_message() {
    let calldata = build_calldata(
        selector("approve(address,uint256)"),
        &[
            address_word(AAVE_LPV2_MAINNET),
            uint_word_biguint(
                BigUint::parse_bytes(
                    b"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                    16,
                )
                .expect("max constant"),
            ),
        ],
    );

    let model = format_with_value(1, USDT_MAINNET, None, &calldata)
        .expect("format succeeds");

    assert_eq!(model.intent, "Approve USDT spending");
    assert!(model.warnings.is_empty());
    assert_eq!(model.items.len(), 2);
    assert_eq!(model.items[0].label, "Spender");
    assert_eq!(
        model.items[0].value.to_ascii_lowercase(),
        AAVE_LPV2_MAINNET.to_ascii_lowercase()
    );
    assert_eq!(
        model.items[1],
        DisplayItem { label: "Amount".to_string(), value: "All".to_string() }
    );
}

#[test]
fn usdt_approve_specific_amount_formats_decimals() {
    let calldata = build_calldata(
        selector("approve(address,uint256)"),
        &[address_word(AAVE_LPV2_MAINNET), uint_word_u128(85_031)],
    );

    let model = format_with_value(1, USDT_MAINNET, None, &calldata)
        .expect("format succeeds");

    assert_eq!(model.intent, "Approve USDT spending");
    assert!(model.warnings.is_empty());
    assert_eq!(model.items.len(), 2);
    assert_eq!(model.items[0].label, "Spender");
    assert_eq!(
        model.items[0].value.to_ascii_lowercase(),
        AAVE_LPV2_MAINNET.to_ascii_lowercase()
    );
    assert_eq!(
        model.items[1],
        DisplayItem {
            label: "Amount".to_string(),
            value: "0.085031 USDT".to_string(),
        }
    );
}

#[test]
fn usdt_approve_all_on_optimism() {
    let calldata = build_calldata(
        selector("approve(address,uint256)"),
        &[
            address_word(AAVE_LPV2_MAINNET),
            uint_word_biguint(
                BigUint::parse_bytes(
                    b"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                    16,
                )
                .expect("max constant"),
            ),
        ],
    );

    let model = format_with_value(
        10,
        "0x94b008aa00579c1307b0ef2c499ad98a8ce58e58",
        None,
        &calldata,
    )
    .expect("format succeeds");

    assert_eq!(model.intent, "Approve USDT spending");
    assert!(model.warnings.is_empty());
    assert_eq!(model.items.len(), 2);
    assert_eq!(model.items[0].label, "Spender");
    assert_eq!(
        model.items[0].value.to_ascii_lowercase(),
        AAVE_LPV2_MAINNET.to_ascii_lowercase()
    );
    assert_eq!(
        model.items[1],
        DisplayItem { label: "Amount".to_string(), value: "All".to_string() }
    );
}

#[test]
fn usdc_approve_all_on_optimism() {
    let calldata = build_calldata(
        selector("approve(address,uint256)"),
        &[
            address_word(AAVE_LPV2_MAINNET),
            uint_word_biguint(
                BigUint::parse_bytes(
                    b"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                    16,
                )
                .expect("max constant"),
            ),
        ],
    );

    let model = format_with_value(
        10,
        "0x7f5c764cBc14f9669B88837CA1490cCa17C31607",
        None,
        &calldata,
    )
    .expect("format succeeds");

    assert_eq!(model.intent, "Approve USDC spending");
    assert!(model.warnings.is_empty());
    assert_eq!(model.items.len(), 2);
    assert_eq!(model.items[0].label, "Spender");
    assert_eq!(
        model.items[0].value.to_ascii_lowercase(),
        AAVE_LPV2_MAINNET.to_ascii_lowercase()
    );
    assert_eq!(
        model.items[1],
        DisplayItem { label: "Amount".to_string(), value: "All".to_string() }
    );
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
