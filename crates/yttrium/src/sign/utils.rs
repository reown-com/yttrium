// use relay_rpc::domain::MessageId;

// /// Generates unique message IDs for use in RPC requests. Uses 56 bits for the
// /// timestamp with millisecond precision, with the last 8 bits from a monotonic
// /// counter. Capable of producing up to `256000` unique values per second.
// #[derive(Debug, Clone)]
// pub struct MessageIdGenerator {
//     next: Arc<AtomicU8>,
// }

// impl MessageIdGenerator {
//     pub fn new() -> Self {
//         Self { next: Arc::new(AtomicU8::new(0)) }
//     }

//     /// Generates a [`MessageId`].
//     pub fn next(&self) -> MessageId {
//         let next = self.next.fetch_add(1, Ordering::Relaxed) as u64;
//         let timestamp = chrono::Utc::now().timestamp_millis() as u64;
//         let id = (timestamp << 8) | next;

//         MessageId::new(id)
//     }
// }

use {
    crate::sign::{
        client_types::Session,
        envelope_type0::{encode_envelope_type0, EnvelopeType0},
    },
    chacha20poly1305::{aead::Aead, AeadCore, ChaCha20Poly1305, KeyInit},
    data_encoding::BASE64,
    rand::Rng,
    relay_rpc::domain::Topic,
    serde::Serialize,
    sha2::{Digest, Sha256},
    std::sync::Arc,
};

pub fn topic_from_sym_key(sym_key: &[u8]) -> Topic {
    hex::encode(sha2::Sha256::digest(sym_key)).into()
}

pub fn diffie_hellman(
    public_key: &x25519_dalek::PublicKey,
    private_key: &x25519_dalek::StaticSecret,
) -> [u8; 32] {
    let shared_key = private_key.diffie_hellman(public_key);
    let derived_key = hkdf::Hkdf::<Sha256>::new(None, shared_key.as_bytes());
    let mut expanded_key = [0u8; 32];
    derived_key.expand(b"", &mut expanded_key).unwrap();
    expanded_key
}

pub fn generate_rpc_id() -> u64 {
    let time = crate::time::SystemTime::now()
        .duration_since(crate::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        * 1_000_000;
    let mut rng = rand::thread_rng();
    let random = rng.gen_range(0..=u16::MAX);
    time + random as u64
}

pub fn is_expired(expiry: u64) -> bool {
    let current_time = crate::time::SystemTime::now()
        .duration_since(crate::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    current_time >= expiry
}

/// Returns true if the peer side is the controller for this session.
/// Determined by comparing `controller_key` to `peer_public_key`.
pub fn is_peer_controller(session: &Session) -> bool {
    if let (Some(controller_key), Some(peer_key)) =
        (session.controller_key, session.peer_public_key)
    {
        controller_key == peer_key
    } else {
        false
    }
}

/// Compute the maximum allowed expiry as now + 7 days (in seconds).
pub fn compute_max_expiry(now_secs: u64) -> u64 {
    now_secs + 60 * 60 * 24 * 7
}

/// Validation errors for session extend requests.
#[derive(Debug, PartialEq, Eq)]
pub enum ExtendValidationError {
    Unauthorized,
    ExpiryTooHigh,
    ExpiryTooLow,
}

/// Validate an inbound wc_sessionExtend request.
/// - Ensures peer is controller
/// - Ensures requested_expiry > current session expiry
/// - Ensures requested_expiry <= now + 7 days
///   Returns accepted expiry on success.
pub fn validate_extend_request(
    session: &Session,
    requested_expiry: u64,
    now_secs: u64,
) -> Result<u64, ExtendValidationError> {
    if !is_peer_controller(session) {
        return Err(ExtendValidationError::Unauthorized);
    }
    let max_expiry = compute_max_expiry(now_secs);
    if requested_expiry <= session.expiry {
        return Err(ExtendValidationError::ExpiryTooLow);
    }
    if requested_expiry > max_expiry {
        return Err(ExtendValidationError::ExpiryTooHigh);
    }
    Ok(requested_expiry)
}

/// Should never fail, but will return a string error if it does
pub fn serialize_and_encrypt_message_type0_envelope<T: Serialize>(
    shared_secret: [u8; 32],
    message: &T,
) -> Result<Arc<str>, String> {
    let serialized = serde_json::to_vec(&message)
        .map_err(|e| format!("Failed to serialize message: {e}"))?;

    let key = ChaCha20Poly1305::new(&shared_secret.into());
    let nonce = ChaCha20Poly1305::generate_nonce()
        .map_err(|e| format!("Failed to generate nonce: {e}"))?;
    let encrypted = key
        .encrypt(&nonce, serialized.as_slice())
        .map_err(|e| format!("Failed to encrypt message: {e}"))?;
    let encoded = encode_envelope_type0(&EnvelopeType0 {
        iv: nonce.into(),
        sb: encrypted,
    });
    Ok(BASE64.encode(encoded.as_slice()).into())
}
