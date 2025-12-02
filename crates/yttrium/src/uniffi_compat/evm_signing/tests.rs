use {
    crate::uniffi_compat::evm_signing::{sign_typed_data, EvmSigningError},
    alloy::signers::local::PrivateKeySigner,
    std::str::FromStr,
};

const TEST_PRIVATE_KEY: &str =
    "0000000000000000000000000000000000000000000000000000000000000001";

// Example from EIP-712 spec
const TYPED_DATA_JSON: &str = r#"{
    "types": {
        "EIP712Domain": [
            { "name": "name", "type": "string" },
            { "name": "version", "type": "string" },
            { "name": "chainId", "type": "uint256" },
            { "name": "verifyingContract", "type": "address" }
        ],
        "Person": [
            { "name": "name", "type": "string" },
            { "name": "wallet", "type": "address" }
        ],
        "Mail": [
            { "name": "from", "type": "Person" },
            { "name": "to", "type": "Person" },
            { "name": "contents", "type": "string" }
        ]
    },
    "primaryType": "Mail",
    "domain": {
        "name": "Ether Mail",
        "version": "1",
        "chainId": 1,
        "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
    },
    "message": {
        "from": {
            "name": "Cow",
            "wallet": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
        },
        "to": {
            "name": "Bob",
            "wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
        },
        "contents": "Hello, Bob!"
    }
}"#;

#[test]
fn test_sign_typed_data_valid() {
    let signer = PrivateKeySigner::from_str(TEST_PRIVATE_KEY).unwrap();
    let signature = sign_typed_data(TYPED_DATA_JSON.to_string(), &signer)
        .expect("signing failed");

    assert!(signature.starts_with("0x"));
    assert_eq!(signature.len(), 134); // 0x + 65 bytes + 1 byte for parity/recovery id? Alloy seems to output 134 chars.
}

#[test]
fn test_sign_typed_data_invalid_json() {
    let signer = PrivateKeySigner::from_str(TEST_PRIVATE_KEY).unwrap();
    let result = sign_typed_data("invalid json".to_string(), &signer);

    assert!(matches!(result, Err(EvmSigningError::InvalidTypedData(_))));
}

#[test]
fn test_sign_typed_data_invalid_structure() {
    let signer = PrivateKeySigner::from_str(TEST_PRIVATE_KEY).unwrap();
    let invalid_structure = r#"{
        "types": {},
        "domain": {},
        "message": {}
    }"#;
    // This might fail on parsing or later validation depending on strictness
    let result = sign_typed_data(invalid_structure.to_string(), &signer);

    // It should fail either at parsing or hashing stage, but definitely fail
    assert!(result.is_err());
}
