use tiny_keccak::{Hasher, Keccak};
use yttrium::clear_signing::{format, DisplayItem, DisplayModel, RawPreview};

const DESCRIPTOR: &str = include_str!("fixtures/stake_weight_descriptor.json");

fn build_calldata(selector: [u8; 4], args: &[[u8; 32]]) -> Vec<u8> {
    let mut data = Vec::with_capacity(4 + args.len() * 32);
    data.extend_from_slice(&selector);
    for arg in args {
        data.extend_from_slice(arg);
    }
    data
}

fn uint_word(value: u64) -> [u8; 32] {
    let mut word = [0u8; 32];
    word[24..].copy_from_slice(&value.to_be_bytes());
    word
}

fn selector(signature: &str) -> [u8; 4] {
    let mut hasher = Keccak::v256();
    hasher.update(signature.as_bytes());
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    [output[0], output[1], output[2], output[3]]
}

#[test]
fn increase_unlock_time_ok() {
    // increaseUnlockTime(uint256)
    let selector = selector("increaseUnlockTime(uint256)");
    let unlock_time = 1_735_660_800u64; // 2024-12-31 16:00:00 UTC
    let calldata = build_calldata(selector, &[uint_word(unlock_time)]);

    let model = format(
        DESCRIPTOR,
        10,
        "0x521B4C065Bbdbe3E20B3727340730936912DfA46",
        &calldata,
    )
    .expect("format should succeed");

    let expected = DisplayModel {
        intent: "Increase Unlock Time".to_string(),
        items: vec![DisplayItem {
            label: "New Unlock Time".to_string(),
            value: "2024-12-31 16:00:00 UTC".to_string(),
        }],
        warnings: Vec::new(),
        raw: None,
    };

    assert_eq!(model, expected);
}

#[test]
fn unknown_selector_fallback() {
    let selector = [0xde, 0xad, 0xbe, 0xef];
    let calldata = build_calldata(selector, &[uint_word(42)]);

    let model = format(
        DESCRIPTOR,
        10,
        "0x521B4C065Bbdbe3E20B3727340730936912DfA46",
        &calldata,
    )
    .expect("format should succeed");

    assert_eq!(model.intent, "Unknown transaction");
    assert!(model.items.is_empty());
    assert!(model
        .warnings
        .iter()
        .any(|warning| warning.contains("No ABI match")));
    assert_eq!(
        model.raw,
        Some(RawPreview {
            selector: "0xdeadbeef".to_string(),
            args: vec![(String::from("0x000000000000000000000000000000000000000000000000000000000000002a"))],
        })
    );
}

#[test]
fn binding_mismatch_warn() {
    let selector = selector("increaseUnlockTime(uint256)");
    let unlock_time = 1_735_660_800u64;
    let calldata = build_calldata(selector, &[uint_word(unlock_time)]);

    let model = format(
        DESCRIPTOR,
        10,
        "0x0000000000000000000000000000000000000000",
        &calldata,
    )
    .expect("format should succeed");

    assert_eq!(model.intent, "Increase Unlock Time");
    assert_eq!(
        model.items,
        vec![DisplayItem {
            label: "New Unlock Time".to_string(),
            value: "2024-12-31 16:00:00 UTC".to_string(),
        }]
    );
    assert!(model
        .warnings
        .iter()
        .any(|warning| warning.contains("Descriptor deployment mismatch")));
    assert!(model.raw.is_none());
}
