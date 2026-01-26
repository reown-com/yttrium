//! TRON blockchain client implementation for WalletConnect.
//!
//! Supports `tron_signMessage` and `tron_signTransaction` per WalletConnect
//! specs. Uses secp256k1 ECDSA (same as Bitcoin/Ethereum) with Base58Check
//! addresses starting with 'T'.

use {
    bip32::{Language, Mnemonic, XPrv},
    rand::{
        SeedableRng,
        rngs::{OsRng, StdRng},
    },
    stacks_secp256k1::{
        Message, PublicKey, Secp256k1, SecretKey,
        hashes::{Hash, sha256},
    },
    tiny_keccak::{Hasher, Keccak},
};

/// TRON BIP44 coin type
const TRON_COIN_TYPE: u32 = 195;

/// TRON mainnet address prefix
const TRON_ADDRESS_PREFIX: u8 = 0x41;

/// TRON message signing prefix (TIP-191 compliant)
const TRON_MESSAGE_PREFIX: &str = "\x19TRON Signed Message:\n";

// Error types

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum TronError {
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),

    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    #[error("Invalid mnemonic: {0}")]
    InvalidMnemonic(String),

    #[error("Signing error: {0}")]
    SigningError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("Key derivation error: {0}")]
    KeyDerivationError(String),
}

// Record types

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct TronKeypair {
    /// Hex-encoded private key (32 bytes = 64 hex chars)
    pub sk: String,
    /// Hex-encoded compressed public key (33 bytes = 66 hex chars)
    pub pk: String,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct TronAddress {
    /// Base58Check encoded address starting with 'T'
    pub base58: String,
    /// Hex-encoded address (21 bytes = 42 hex chars, with 0x41 prefix)
    pub hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct TronSignedTransaction {
    /// Transaction ID (SHA256 of raw_data)
    pub tx_id: String,
    /// Array of hex-encoded RSV signatures
    pub signature: Vec<String>,
    /// Original raw_data_hex
    pub raw_data_hex: String,
}

// Helper functions

fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak::v256();
    let mut output = [0u8; 32];
    hasher.update(data);
    hasher.finalize(&mut output);
    output
}

fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let hash = sha256::Hash::hash(data);
    let mut output = [0u8; 32];
    output.copy_from_slice(hash.as_ref());
    output
}

fn double_sha256(data: &[u8]) -> [u8; 32] {
    sha256_hash(&sha256_hash(data))
}

/// Encode bytes to Base58Check (with double SHA256 checksum)
fn base58check_encode(data: &[u8]) -> String {
    let checksum = double_sha256(data);
    let mut with_checksum = data.to_vec();
    with_checksum.extend_from_slice(&checksum[0..4]);
    bs58::encode(with_checksum).into_string()
}

/// Decode Base58Check to bytes (verifying checksum)
fn base58check_decode(encoded: &str) -> Result<Vec<u8>, TronError> {
    let decoded = bs58::decode(encoded)
        .into_vec()
        .map_err(|e| TronError::InvalidAddress(e.to_string()))?;

    if decoded.len() < 4 {
        return Err(TronError::InvalidAddress("Address too short".to_string()));
    }

    let (data, checksum) = decoded.split_at(decoded.len() - 4);
    let computed_checksum = double_sha256(data);

    if checksum != &computed_checksum[0..4] {
        return Err(TronError::InvalidAddress("Invalid checksum".to_string()));
    }

    Ok(data.to_vec())
}

/// Derive address from uncompressed public key bytes (64 bytes, without 0x04
/// prefix)
fn derive_address_from_pubkey_bytes(
    pubkey_bytes: &[u8],
) -> Result<TronAddress, TronError> {
    if pubkey_bytes.len() != 64 {
        return Err(TronError::InvalidPublicKey(format!(
            "Expected 64 bytes, got {}",
            pubkey_bytes.len()
        )));
    }

    // Keccak256 hash of public key
    let hash = keccak256(pubkey_bytes);

    // Take last 20 bytes and prepend TRON prefix
    let mut address_bytes = [0u8; 21];
    address_bytes[0] = TRON_ADDRESS_PREFIX;
    address_bytes[1..].copy_from_slice(&hash[12..]);

    let base58 = base58check_encode(&address_bytes);
    let hex = hex::encode(address_bytes);

    Ok(TronAddress { base58, hex })
}

// Exported functions

/// Generate a new random TRON keypair
#[uniffi::export]
pub fn tron_generate_keypair() -> TronKeypair {
    let secp = Secp256k1::new();
    let (secret_key, public_key) =
        secp.generate_keypair(&mut StdRng::from_rng(OsRng).unwrap());

    TronKeypair {
        sk: hex::encode(secret_key.secret_bytes()),
        pk: hex::encode(public_key.serialize()),
    }
}

/// Generate a TRON keypair from a BIP39 mnemonic
///
/// Uses BIP44 derivation path m/44'/195'/0'/0/0 by default
#[uniffi::export]
pub fn tron_generate_keypair_from_mnemonic(
    mnemonic: &str,
    derivation_path: Option<String>,
) -> Result<TronKeypair, TronError> {
    let mnemonic = Mnemonic::new(mnemonic, Language::English)
        .map_err(|e| TronError::InvalidMnemonic(e.to_string()))?;

    let seed = mnemonic.to_seed("");

    // Default TRON BIP44 path: m/44'/195'/0'/0/0
    let path = derivation_path.unwrap_or_else(|| {
        format!("m/44'/{}'/{}'/{}/{}", TRON_COIN_TYPE, 0, 0, 0)
    });

    // Parse the derivation path and derive the key
    let xprv = XPrv::derive_from_path(
        &seed,
        &path.parse().map_err(|e| {
            TronError::KeyDerivationError(format!(
                "Invalid derivation path: {}",
                e
            ))
        })?,
    )
    .map_err(|e| TronError::KeyDerivationError(e.to_string()))?;

    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(&xprv.private_key().to_bytes())
        .map_err(|e| {
            TronError::KeyDerivationError(format!("Invalid derived key: {}", e))
        })?;
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);

    Ok(TronKeypair {
        sk: hex::encode(secret_key.secret_bytes()),
        pk: hex::encode(public_key.serialize()),
    })
}

/// Get the TRON address from a keypair
#[uniffi::export]
pub fn tron_get_address(
    keypair: &TronKeypair,
) -> Result<TronAddress, TronError> {
    let pk_bytes = hex::decode(&keypair.pk)
        .map_err(|e| TronError::InvalidPublicKey(e.to_string()))?;

    let public_key = PublicKey::from_slice(&pk_bytes)
        .map_err(|e| TronError::InvalidPublicKey(e.to_string()))?;

    // Get uncompressed public key (65 bytes), skip the 0x04 prefix
    let uncompressed = public_key.serialize_uncompressed();
    let pubkey_bytes = &uncompressed[1..]; // 64 bytes

    derive_address_from_pubkey_bytes(pubkey_bytes)
}

/// Convert a hex-encoded address to TronAddress
#[uniffi::export]
pub fn tron_address_from_hex(hex_addr: &str) -> Result<TronAddress, TronError> {
    let hex_clean = hex_addr.strip_prefix("0x").unwrap_or(hex_addr);
    let address_bytes = hex::decode(hex_clean)
        .map_err(|e| TronError::InvalidAddress(e.to_string()))?;

    if address_bytes.len() != 21 {
        return Err(TronError::InvalidAddress(format!(
            "Expected 21 bytes, got {}",
            address_bytes.len()
        )));
    }

    if address_bytes[0] != TRON_ADDRESS_PREFIX {
        return Err(TronError::InvalidAddress(
            "Invalid address prefix (expected 0x41)".to_string(),
        ));
    }

    let base58 = base58check_encode(&address_bytes);

    Ok(TronAddress { base58, hex: hex::encode(address_bytes) })
}

/// Convert a Base58Check address to TronAddress
#[uniffi::export]
pub fn tron_address_from_base58(
    base58_addr: &str,
) -> Result<TronAddress, TronError> {
    let address_bytes = base58check_decode(base58_addr)?;

    if address_bytes.len() != 21 {
        return Err(TronError::InvalidAddress(format!(
            "Expected 21 bytes, got {}",
            address_bytes.len()
        )));
    }

    if address_bytes[0] != TRON_ADDRESS_PREFIX {
        return Err(TronError::InvalidAddress(
            "Invalid address prefix (expected 0x41)".to_string(),
        ));
    }

    Ok(TronAddress {
        base58: base58_addr.to_string(),
        hex: hex::encode(address_bytes),
    })
}

/// Sign a message following TIP-191 (tron_signMessage)
///
/// Returns a hex-encoded RSV signature (65 bytes = 130 hex chars)
#[uniffi::export]
pub fn tron_sign_message(
    message: &str,
    keypair: &TronKeypair,
) -> Result<String, TronError> {
    let sk_bytes = hex::decode(&keypair.sk)
        .map_err(|e| TronError::InvalidPrivateKey(e.to_string()))?;

    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(&sk_bytes)
        .map_err(|e| TronError::InvalidPrivateKey(e.to_string()))?;

    // TIP-191: prefix with "\x19TRON Signed Message:\n" + message_length
    let prefix = format!("{}{}", TRON_MESSAGE_PREFIX, message.len());
    let mut prefixed_message = prefix.into_bytes();
    prefixed_message.extend_from_slice(message.as_bytes());

    // Keccak256 hash of prefixed message
    let hash = keccak256(&prefixed_message);

    // Sign with recoverable signature
    let msg = Message::from_digest(hash);
    let signature = secp.sign_ecdsa_recoverable(&msg, &secret_key);

    // Serialize to RSV format (r: 32 bytes + s: 32 bytes + v: 1 byte)
    let (recovery_id, sig_bytes) = signature.serialize_compact();
    let mut rsv = [0u8; 65];
    rsv[..64].copy_from_slice(&sig_bytes);
    rsv[64] = recovery_id.to_i32() as u8 + 27; // v = recovery_id + 27

    Ok(hex::encode(rsv))
}

/// Sign a transaction (tron_signTransaction)
///
/// Takes the hex-encoded raw_data and returns a signed transaction
#[uniffi::export]
pub fn tron_sign_transaction(
    raw_data_hex: &str,
    keypair: &TronKeypair,
) -> Result<TronSignedTransaction, TronError> {
    let raw_data_clean =
        raw_data_hex.strip_prefix("0x").unwrap_or(raw_data_hex);
    let raw_data_bytes = hex::decode(raw_data_clean)
        .map_err(|e| TronError::InvalidTransaction(e.to_string()))?;

    let sk_bytes = hex::decode(&keypair.sk)
        .map_err(|e| TronError::InvalidPrivateKey(e.to_string()))?;

    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(&sk_bytes)
        .map_err(|e| TronError::InvalidPrivateKey(e.to_string()))?;

    // Calculate txID: SHA256 of raw_data (NOT Keccak!)
    let tx_id = sha256_hash(&raw_data_bytes);

    // Sign the txID
    let msg = Message::from_digest(tx_id);
    let signature = secp.sign_ecdsa_recoverable(&msg, &secret_key);

    // Serialize to RSV format
    let (recovery_id, sig_bytes) = signature.serialize_compact();
    let mut rsv = [0u8; 65];
    rsv[..64].copy_from_slice(&sig_bytes);
    rsv[64] = recovery_id.to_i32() as u8;

    Ok(TronSignedTransaction {
        tx_id: hex::encode(tx_id),
        signature: vec![hex::encode(rsv)],
        raw_data_hex: raw_data_clean.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let keypair = tron_generate_keypair();
        // Private key: 32 bytes = 64 hex chars
        assert_eq!(keypair.sk.len(), 64);
        // Compressed public key: 33 bytes = 66 hex chars
        assert_eq!(keypair.pk.len(), 66);
    }

    #[test]
    fn test_generate_keypair_from_mnemonic() {
        // Generate a valid mnemonic first using bip32
        let mnemonic = Mnemonic::random(
            StdRng::from_rng(OsRng).unwrap(),
            Language::English,
        );
        let phrase = mnemonic.phrase();

        let keypair =
            tron_generate_keypair_from_mnemonic(phrase, None).unwrap();

        // Should be deterministic
        let keypair2 =
            tron_generate_keypair_from_mnemonic(phrase, None).unwrap();
        assert_eq!(keypair.sk, keypair2.sk);
        assert_eq!(keypair.pk, keypair2.pk);
    }

    #[test]
    fn test_address_derivation() {
        let keypair = tron_generate_keypair();
        let address = tron_get_address(&keypair).unwrap();

        // TRON addresses start with 'T'
        assert!(address.base58.starts_with('T'));
        // Base58 address is 34 characters
        assert_eq!(address.base58.len(), 34);
        // Hex address is 21 bytes = 42 hex chars
        assert_eq!(address.hex.len(), 42);
        // Hex starts with 41 (TRON prefix)
        assert!(address.hex.starts_with("41"));
    }

    #[test]
    fn test_address_roundtrip() {
        let keypair = tron_generate_keypair();
        let address = tron_get_address(&keypair).unwrap();

        // Convert to hex and back
        let from_hex = tron_address_from_hex(&address.hex).unwrap();
        assert_eq!(address.base58, from_hex.base58);

        // Convert to base58 and back
        let from_base58 = tron_address_from_base58(&address.base58).unwrap();
        assert_eq!(address.hex, from_base58.hex);
    }

    #[test]
    fn test_sign_message() {
        let keypair = tron_generate_keypair();
        let signature = tron_sign_message("Hello TRON", &keypair).unwrap();

        // RSV format: 65 bytes = 130 hex chars
        assert_eq!(signature.len(), 130);

        // Verify v value is 27 or 28
        let sig_bytes = hex::decode(&signature).unwrap();
        assert!(sig_bytes[64] == 27 || sig_bytes[64] == 28);
    }

    #[test]
    fn test_sign_transaction() {
        let keypair = tron_generate_keypair();
        // Example raw_data_hex (minimal test data)
        let raw_data_hex = "0a020a0c220838c3dc70fb5badc340\
                            e8ddf5f9c5325a67080112630a2d74\
                            797065";

        let signed = tron_sign_transaction(raw_data_hex, &keypair).unwrap();

        // txID is SHA256 hash: 32 bytes = 64 hex chars
        assert_eq!(signed.tx_id.len(), 64);
        // Should have one signature
        assert_eq!(signed.signature.len(), 1);
        // Signature is 65 bytes = 130 hex chars
        assert_eq!(signed.signature[0].len(), 130);
    }

    #[test]
    fn test_known_address() {
        // Test with a known TRON address for verification
        let base58 = "TJCnKsPa7y5okkXvQAidZBzqx3QyQ6sxMW";
        let address = tron_address_from_base58(base58).unwrap();
        assert_eq!(address.base58, base58);
        assert!(address.hex.starts_with("41"));
    }

    #[test]
    fn test_invalid_mnemonic() {
        let result =
            tron_generate_keypair_from_mnemonic("invalid mnemonic", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_address() {
        // Too short
        let result = tron_address_from_hex("41");
        assert!(result.is_err());

        // Invalid checksum in base58
        let result = tron_address_from_base58("TJCnKsPa7y5okkXvQAidZBzqx3Q");
        assert!(result.is_err());
    }
}
