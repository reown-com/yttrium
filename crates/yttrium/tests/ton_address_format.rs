use ton_lib::ton_lib_core::{cell::TonHash, types::TonAddress};

#[test]
fn unbounceable_address_format_for_known_pubkey() {
    // Public key from user example
    let pk_hex = "a323642d9cd5e4631368be4f3b15017427e4d1d15d97723a103f1c29609b7c14";
    let pk_bytes = hex::decode(pk_hex).expect("valid hex");
    let mut pk_array = [0u8; 32];
    pk_array.copy_from_slice(&pk_bytes);

    let address = TonAddress::new(0, TonHash::from(pk_array));

    // Unbounceable, URL-safe, mainnet should start with "UQ" and match expected
    let friendly_unbounceable = address.to_base64(true, false, true);
    println!("Unbounceable (UQ): {}", friendly_unbounceable);
    assert_eq!(
        friendly_unbounceable,
        "UQCjI2QtnNXkYxNovk87FQF0J-TR0V2XcjoQPxwpYJt8FOUF"
    );

    // Bounceable, URL-safe, mainnet should start with "EQ" and match expected
    let friendly_bounceable = address.to_base64(true, true, true);
    println!("Bounceable (EQ): {}", friendly_bounceable);
    assert_eq!(
        friendly_bounceable,
        "EQCjI2QtnNXkYxNovk87FQF0J-TR0V2XcjoQPxwpYJt8FLjA"
    );
}


