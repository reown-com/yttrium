use {
    super::ProtocolError,
    chacha20poly1305::{
        XChaCha20Poly1305, XNonce,
        aead::{Aead, KeyInit, Payload},
    },
    curve25519_dalek::{
        edwards::CompressedEdwardsY, montgomery::MontgomeryPoint,
    },
    ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey},
    hkdf::Hkdf,
    sha2::{Digest, Sha256},
    zeroize::Zeroizing,
};

pub fn random_bytes<const N: usize>() -> [u8; N] {
    let mut b = [0u8; N];
    getrandom::getrandom(&mut b).expect("system randomness");
    b
}

pub fn new_sym_key() -> Zeroizing<[u8; 32]> {
    Zeroizing::new(random_bytes())
}

pub fn new_pairing_key() -> SigningKey {
    SigningKey::from_bytes(&random_bytes())
}

/// The bindingKey seed is HKDF-SHA256(ikm = symKey, salt = empty, info = "wc/binding").
pub fn derive_binding_key(sym: &[u8; 32]) -> SigningKey {
    let mut seed = Zeroizing::new([0u8; 32]);
    Hkdf::<Sha256>::new(Some(&[]), sym)
        .expand(b"wc/binding", seed.as_mut())
        .expect("32 bytes");
    SigningKey::from_bytes(&seed)
}

fn canonical_point(bytes: &[u8; 32]) -> Option<curve25519_dalek::EdwardsPoint> {
    let p = CompressedEdwardsY(*bytes).decompress()?;
    (p.compress().as_bytes() == bytes).then_some(p)
}

/// A pairing public key must be canonical, not small order, and in the prime-order subgroup.
pub fn is_valid_pairing_pub(bytes: &[u8]) -> bool {
    let Ok(arr) = <[u8; 32]>::try_from(bytes) else { return false };
    canonical_point(&arr)
        .is_some_and(|p| !p.is_small_order() && p.is_torsion_free())
}

/// Conventions: dalek `verify_strict` (cofactorless, refuses small-order A and R, non-canonical R
/// and S >= L) plus the canonical check on A.
pub fn verify_strict(public: &[u8], sig: &[u8], msg: &[u8]) -> bool {
    let (Ok(pk), Ok(sig)) =
        (<[u8; 32]>::try_from(public), <[u8; 64]>::try_from(sig))
    else {
        return false;
    };
    if canonical_point(&pk).is_none() {
        return false;
    }
    let Ok(vk) = VerifyingKey::from_bytes(&pk) else { return false };
    vk.verify_strict(msg, &Signature::from_bytes(&sig)).is_ok()
}

pub fn sign(key: &SigningKey, msg: &[u8]) -> [u8; 64] {
    key.sign(msg).to_bytes()
}

/// Every signature input starts `label ‖ 0x00 ‖ bindingPub`.
pub fn signed_prefix(label: &str, binding_pub: &[u8; 32]) -> Vec<u8> {
    let mut v = Vec::with_capacity(label.len() + 33);
    v.extend_from_slice(label.as_bytes());
    v.push(0);
    v.extend_from_slice(binding_pub);
    v
}

/// `label ‖ 0x00 ‖ bindingPub ‖ u32be(len(plain)) ‖ plain ‖ payload`.
pub fn message_signing_input(
    label: &str,
    binding_pub: &[u8; 32],
    plain: &[u8],
    payload: &[u8],
) -> Vec<u8> {
    let mut v = signed_prefix(label, binding_pub);
    v.extend_from_slice(&(plain.len() as u32).to_be_bytes());
    v.extend_from_slice(plain);
    v.extend_from_slice(payload);
    v
}

/// `ad = label ‖ 0x00 ‖ bindingPub ‖ plain`, without the length.
pub fn seal_ad(label: &str, binding_pub: &[u8; 32], plain: &[u8]) -> Vec<u8> {
    let mut v = signed_prefix(label, binding_pub);
    v.extend_from_slice(plain);
    v
}

pub struct SealingKeys {
    pub d2w: Zeroizing<[u8; 32]>,
    pub w2d: Zeroizing<[u8; 32]>,
}

/// The two directional sealing keys: X25519 over the Ed25519 pairing keys, then HKDF with the
/// symKey as salt and the bindingPub and both pairing keys in the info.
pub fn derive_sealing_keys(
    version: u32,
    own: &SigningKey,
    peer_pub: &[u8; 32],
    sym: &[u8; 32],
    binding_pub: &[u8; 32],
    dapp_pub: &[u8; 32],
    wallet_pub: &[u8; 32],
) -> Result<SealingKeys, ProtocolError> {
    let peer = VerifyingKey::from_bytes(peer_pub)
        .map_err(|_| ProtocolError::malformed("peer pairingPub"))?;
    let peer_mont: MontgomeryPoint = peer.to_montgomery();
    let own_scalar = Zeroizing::new(own.to_scalar_bytes());
    let shared = Zeroizing::new(peer_mont.mul_clamped(*own_scalar).to_bytes());
    let hk = Hkdf::<Sha256>::new(Some(sym), shared.as_ref());
    let derive = |dir: &str| {
        let mut info = format!("wc/v{version}/seal/{dir}").into_bytes();
        info.extend_from_slice(binding_pub);
        info.extend_from_slice(dapp_pub);
        info.extend_from_slice(wallet_pub);
        let mut k = Zeroizing::new([0u8; 32]);
        hk.expand(&info, k.as_mut()).expect("32 bytes");
        k
    };
    Ok(SealingKeys { d2w: derive("d2w"), w2d: derive("w2d") })
}

pub fn seal(
    key: &[u8; 32],
    nonce: &[u8; 24],
    plaintext: &[u8],
    ad: &[u8],
) -> Vec<u8> {
    XChaCha20Poly1305::new(key.into())
        .encrypt(XNonce::from_slice(nonce), Payload { msg: plaintext, aad: ad })
        .expect("encryption cannot fail")
}

pub fn open(
    key: &[u8; 32],
    nonce: &[u8; 24],
    ciphertext: &[u8],
    ad: &[u8],
) -> Result<Vec<u8>, ProtocolError> {
    XChaCha20Poly1305::new(key.into())
        .decrypt(
            XNonce::from_slice(nonce),
            Payload { msg: ciphertext, aad: ad },
        )
        .map_err(|_| ProtocolError::CannotDecrypt)
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
