use yttrium::sign::client::Client;
use chacha20poly1305::{aead::Aead, ChaCha20Poly1305, KeyInit};
use chacha20poly1305::aead::AeadCore;
use data_encoding::BASE64;
use serde_json::json;

fn sym_key() -> [u8; 32] {
    hex::decode("0653ca620c7b4990392e1c53c4a51c14a2840cd20f0f1524cf435b17b6fe988c")
        .unwrap()
        .try_into()
        .unwrap()
}

#[test]
fn decrypt_type0_envelope_matches_swift_vectors() {
    // This test uses Swift ChaChaPoly vectors to ensure we can open the same envelope format.
    // Swift combined = nonce || ciphertext || tag; our envelope = [0][nonce][ciphertext+tag]
    let key_bytes: [u8; 32] = hex::decode(
        "0653ca620c7b4990392e1c53c4a51c14a2840cd20f0f1524cf435b17b6fe988c",
    )
    .unwrap()
    .try_into()
    .unwrap();

    // Vector from Swift tests for plaintext "WalletConnect"
    let combined_b64 = "cXdlY2ZhYXNkYWRzVhkbjHqli8hN0rFbAtMPIsJho4zLvWskMTQKSGw=";
    let combined = BASE64.decode(combined_b64.as_bytes()).unwrap();
    assert!(combined.len() > 12);
    let (nonce, ct_tag) = combined.split_at(12);

    // Build type0 envelope: [0][nonce][ct+tag]
    let mut envelope = Vec::with_capacity(1 + combined.len());
    envelope.push(0);
    envelope.extend_from_slice(nonce);
    envelope.extend_from_slice(ct_tag);
    let envelope_b64 = BASE64.encode(&envelope);

    let decrypted = Client::decrypt_type0_envelope(key_bytes, &envelope_b64)
        .expect("decrypt envelope");
    assert_eq!(decrypted, b"WalletConnect");
}


