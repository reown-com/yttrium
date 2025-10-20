use hex::FromHex;
use yttrium::clear_signing::{format, DisplayItem};

const USDT_DESCRIPTOR: &str =
    include_str!("../../../vendor/registry/tether/calldata-usdt.json");

const USDT_OPTIMISM: &str = "0x94b008aa00579c1307b0ef2c499ad98a8ce58e58";

fn decode_calldata(hex: &str) -> Vec<u8> {
    Vec::from_hex(hex.trim_start_matches("0x")).expect("valid hex")
}

#[test]
fn approve_all_usdt_displays_all_message() {
    let calldata = decode_calldata(
        "0x095ea7b3000000000000000000000000794a61358d6845594f94dc1db02a252b5b4814adffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    );

    let model = format(USDT_DESCRIPTOR, 10, USDT_OPTIMISM, &calldata)
        .expect("format succeeds");

    assert_eq!(model.intent, "Approve");
    assert!(model.warnings.is_empty());
    assert_eq!(model.items.len(), 2);
    assert_eq!(
        model.items[0],
        DisplayItem {
            label: "Spender".to_owned(),
            value: "Aave DAO".to_owned(),
        }
    );
    assert_eq!(
        model.items[1],
        DisplayItem { label: "Amount".to_owned(), value: "All".to_owned() }
    );
}

#[test]
fn approve_specific_usdt_amount_formats_with_decimals() {
    // 0.085031 USDT => 85031 units with 6 decimals -> 0x14c27
    let calldata = decode_calldata(
        "0x095ea7b3000000000000000000000000794a61358d6845594f94dc1db02a252b5b4814ad0000000000000000000000000000000000000000000000000000000000014c27",
    );

    let model = format(USDT_DESCRIPTOR, 10, USDT_OPTIMISM, &calldata)
        .expect("format succeeds");

    assert_eq!(model.intent, "Approve");
    assert!(model.warnings.is_empty());
    assert_eq!(model.items.len(), 2);
    assert_eq!(
        model.items[0],
        DisplayItem {
            label: "Spender".to_owned(),
            value: "Aave DAO".to_owned(),
        }
    );
    assert_eq!(
        model.items[1],
        DisplayItem {
            label: "Amount".to_owned(),
            value: "0.085031 USDT".to_owned(),
        }
    );
}
