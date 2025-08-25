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
    rand::Rng,
    relay_rpc::domain::Topic,
    sha2::{Digest, Sha256},
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
