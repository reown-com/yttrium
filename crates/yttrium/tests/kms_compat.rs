use {
    chacha20poly1305::{aead::Aead, ChaCha20Poly1305, KeyInit, Nonce},
    data_encoding::{BASE64, BASE64URL_NOPAD},
};

fn hex_to_array_32(s: &str) -> [u8; 32] {
    let v = hex::decode(s).expect("invalid hex");
    v.try_into().expect("expected 32 bytes key")
}

#[test]
fn chacha_poly_encode_decode_roundtrip_matches_swift() {
    let message = b"Test Message";
    let key_bytes = hex_to_array_32(
        "0653ca620c7b4990392e1c53c4a51c14a2840cd20f0f1524cf435b17b6fe988c",
    );
    let key = ChaCha20Poly1305::new(&key_bytes.into());

    // Deterministic nonce for the test (12 bytes)
    let nonce = Nonce::from(*b"0123456789ab");

    let ct_tag = key.encrypt(&nonce, message.as_slice()).expect("encrypt");

    // CryptoKit combined format: nonce || ciphertext || tag
    let mut combined = Vec::with_capacity(12 + ct_tag.len());
    combined.extend_from_slice(nonce.as_slice());
    combined.extend_from_slice(&ct_tag);

    // Decrypt
    let (n_part, ct_part) = combined.split_at(12);
    let decrypted = ChaCha20Poly1305::new(&key_bytes.into())
        .decrypt(&Nonce::from(*<&[u8; 12]>::try_from(n_part).unwrap()), ct_part)
        .expect("decrypt");
    assert_eq!(decrypted.as_slice(), message);
}

#[test]
fn chacha_poly_encode_cohesion_matches_swift_base64() {
    let plaintext = b"WalletConnect";
    let key_bytes = hex_to_array_32(
        "0653ca620c7b4990392e1c53c4a51c14a2840cd20f0f1524cf435b17b6fe988c",
    );
    let key = ChaCha20Poly1305::new(&key_bytes.into());
    let nonce = Nonce::from(*b"qwecfaasdads"); // 12 bytes

    let ct_tag = key.encrypt(&nonce, plaintext.as_slice()).expect("encrypt");

    let mut combined = Vec::with_capacity(12 + ct_tag.len());
    combined.extend_from_slice(nonce.as_slice());
    combined.extend_from_slice(&ct_tag);

    let b64 = BASE64.encode(&combined);
    assert_eq!(b64, "cXdlY2ZhYXNkYWRzVhkbjHqli8hN0rFbAtMPIsJho4zLvWskMTQKSGw=");
}

#[test]
fn chacha_poly_decode_cohesion_from_swift_base64() {
    let key_bytes = hex_to_array_32(
        "0653ca620c7b4990392e1c53c4a51c14a2840cd20f0f1524cf435b17b6fe988c",
    );
    let key = ChaCha20Poly1305::new(&key_bytes.into());
    let combined = BASE64
        .decode(b"cXdlY2ZhYXNkYWRzVhkbjHqli8hN0rFbAtMPIsJho4zLvWskMTQKSGw=")
        .expect("base64 decode");
    assert!(combined.len() > 12);
    let (n_part, ct_part) = combined.split_at(12);
    let decrypted = key
        .decrypt(&Nonce::from(*<&[u8; 12]>::try_from(n_part).unwrap()), ct_part)
        .expect("decrypt");
    assert_eq!(decrypted, b"WalletConnect");
}

#[test]
fn chacha_poly_malformed_sealbox_should_fail() {
    let key_bytes = hex_to_array_32(
        "0653ca620c7b4990392e1c53c4a51c14a2840cd20f0f1524cf435b17b6fe988c",
    );
    let key = ChaCha20Poly1305::new(&key_bytes.into());
    let nonce = Nonce::from(*b"qwecfaasdads");
    let ct_tag =
        key.encrypt(&nonce, b"Test Message".as_slice()).expect("encrypt");
    let mut combined = Vec::with_capacity(12 + ct_tag.len());
    combined.extend_from_slice(nonce.as_slice());
    combined.extend_from_slice(&ct_tag);
    combined.push(1); // corrupt

    let (n_part, ct_part) = combined.split_at(12);
    let result = ChaCha20Poly1305::new(&key_bytes.into()).decrypt(
        &Nonce::from(*<&[u8; 12]>::try_from(n_part).unwrap()),
        ct_part,
    );
    assert!(result.is_err());
}

#[test]
fn envelope_invalid_base64_should_fail() {
    assert!(BASE64.decode(b"invalid_base64").is_err());
    assert!(BASE64URL_NOPAD.decode(b"invalid_base64url").is_err());
}

#[test]
fn envelope_type2_indicator_from_base64url() {
    // Only check leading type byte equals 2 as in Swift test
    let serialised = b"AnsibWV0aG9kIjoid2Nfc2Vzc2lvbkF1dGhlbnRpY2F0ZSIsImlkIjoxNzEyMjIwNjg1NjM1MzAzLCJqc29ucnBjIjoiMi4wIiwicGFyYW1zIjp7ImV4cGlyeVRpbWVzdGFtcCI6MTcxMjIyNDI4NSwiYXV0aFBheWxvYWQiOnsidHlwZSI6ImVpcDQzNjEiLCJzdGF0ZW1lbnQiOiJJIGFjY2VwdCB0aGUgU2VydmljZU9yZyBUZXJtcyBvZiBTZXJ2aWNlOiBodHRwczpcL1wvYXBwLndlYjNpbmJveC5jb21cL3RvcyIsImNoYWlucyI6WyJlaXAxNTU6MSIsImVpcDE1NToxMzciXSwicmVzb3VyY2VzIjpbInVybjpyZWNhcDpleUJoZEhRaU9uc2laV2x3TVRVMUlqcDdJbkpsY1hWbGMzUXZjR1Z5YzI5dVlXeGZjMmxuYmlJNlczdDlYWDE5ZlE9PSJdLCJkb21haW4iOiJhcHAud2ViM2luYm94IiwidmVyc2lvbiI6IjEiLCJhdWQiOiJodHRwczpcL1wvYXBwLndlYjNpbmJveC5jb21cL2xvZ2luIiwibm9uY2UiOiIzMjg5MTc1NiIsImlhdCI6IjIwMjQtMDQtMDRUMDg6NTE6MjVaIn0sInJlcXVlc3RlciI6eyJwdWJsaWNLZXkiOiIxOWYzNmY1N2M1NjYxNDY4ODk0NmU3MzliNzY4NmE2ZmE1OGNiZWFmOGQ3MzZmM2EzZDI2NjVlM2NlYmE4ZDQ5IiwibWV0YWRhdGEiOnsicmVkaXJlY3QiOnsibmF0aXZlIjoid2NkYXBwOlwvXC8iLCJ1bml2ZXJzYWwiOiJ3d3cud2FsbGV0Y29ubmVjdC5jb21cL2RhcHAifSwiaWNvbnMiOlsiaHR0cHM6XC9cL2F2YXRhcnMuZ2l0aHVidXNlcmNvbnRlbnQuY29tXC91XC8zNzc4NDg4NiJdLCJkZXNjcmlwdGlvbiI6IldhbGxldENvbm5lY3QgREFwcCBzYW1wbGUiLCJ1cmwiOiJ3YWxsZXQuY29ubmVjdCIsIm5hbWUiOiJTd2lmdCBEYXBwIn19fX0";
    let bytes = BASE64URL_NOPAD.decode(serialised).expect("b64url");
    assert_eq!(bytes.first().copied(), Some(2));
}
