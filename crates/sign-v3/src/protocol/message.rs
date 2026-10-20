use {
    super::{
        ProtocolError,
        crypto::{
            message_signing_input, open, random_bytes, seal, seal_ad, sign,
            signed_prefix, verify_strict,
        },
        encoding::{b64u, from_b64u},
        ijson::{Parsed, parse_bytes},
        types::{Plain, SignedObject, WireMessage, tags},
    },
    ed25519_dalek::SigningKey,
    serde::Serialize,
    serde_json::Value,
};

pub mod labels {
    pub const BINDING_PROOF: &str = "wc/bindingProof";
    pub const FETCH: &str = "wc/fetch";
    pub const BIND: &str = "wc/bind";
    pub const REJECT: &str = "wc/reject";
    pub const PROPOSE: &str = "wc/propose";
    pub const RESPOND: &str = "wc/respond";
}

pub fn close_label(version: u32) -> String {
    format!("wc/v{version}/close")
}

/// Labels table: proposal and approval are versionless, everything else carries the version.
pub fn label_for(version: u32, tag: u32) -> String {
    match tag {
        tags::PROPOSAL => labels::PROPOSE.into(),
        tags::APPROVAL => labels::RESPOND.into(),
        _ => format!("wc/v{version}/{tag}"),
    }
}

fn strict(bytes: &[u8], what: &str) -> Result<Parsed, ProtocolError> {
    parse_bytes(bytes)
        .map_err(|e| ProtocolError::NotIJson(format!("{what}: {}", e.0)))
}

fn binding_arr(b: &[u8]) -> [u8; 32] {
    b.try_into().expect("bindingPub is 32 bytes")
}

pub fn build_signed<T: Serialize>(
    key: &SigningKey,
    label: &str,
    binding_pub: &[u8; 32],
    value: &T,
) -> SignedObject {
    let payload = serde_json::to_vec(value).expect("serializable");
    let mut input = signed_prefix(label, binding_pub);
    input.extend_from_slice(&payload);
    SignedObject { payload: b64u(&payload), sig: b64u(&sign(key, &input)) }
}

/// Verify first, then parse strictly.
pub fn open_signed(
    public: &[u8],
    label: &str,
    binding_pub: &[u8; 32],
    o: &SignedObject,
) -> Result<Parsed, ProtocolError> {
    let payload = from_b64u(&o.payload)?;
    let sig = from_b64u(&o.sig)?;
    let mut input = signed_prefix(label, binding_pub);
    input.extend_from_slice(&payload);
    if !verify_strict(public, &sig, &input) {
        return Err(ProtocolError::BadSignature(label.into()));
    }
    strict(&payload, label)
}

/// Fetch signs the connectionId's ASCII bytes; close signs nothing beyond the bindingPub.
pub fn sign_bytes(
    key: &SigningKey,
    label: &str,
    binding_pub: &[u8; 32],
    rest: &[u8],
) -> String {
    let mut input = signed_prefix(label, binding_pub);
    input.extend_from_slice(rest);
    b64u(&sign(key, &input))
}

pub fn verify_bytes(
    public: &[u8],
    sig: &str,
    label: &str,
    binding_pub: &[u8; 32],
    rest: &[u8],
) -> bool {
    let Ok(sig) = from_b64u(sig) else { return false };
    let mut input = signed_prefix(label, binding_pub);
    input.extend_from_slice(rest);
    verify_strict(public, &sig, &input)
}

pub struct BuildMessage<'a> {
    pub key: &'a SigningKey,
    pub version: u32,
    pub binding_pub: &'a [u8; 32],
    pub tag: u32,
    pub seq: u64,
    pub deadline: Option<u64>,
    pub rpc: &'a Value,
    pub seal_key: Option<&'a [u8; 32]>,
    pub nonce: Option<[u8; 24]>,
}

pub fn build_message(m: BuildMessage<'_>) -> WireMessage {
    let label = label_for(m.version, m.tag);
    let nonce = m.seal_key.map(|_| m.nonce.unwrap_or_else(random_bytes));
    let plain = Plain {
        tag: m.tag,
        seq: m.seq,
        sealed: nonce.is_some(),
        deadline: m.deadline,
        nonce: nonce.map(|n| b64u(&n)),
    };
    let plain_bytes = serde_json::to_vec(&plain).expect("serializable");
    let rpc_bytes = serde_json::to_vec(m.rpc).expect("serializable");
    let payload = match (m.seal_key, nonce) {
        (Some(k), Some(n)) => seal(
            k,
            &n,
            &rpc_bytes,
            &seal_ad(&label, m.binding_pub, &plain_bytes),
        ),
        _ => rpc_bytes,
    };
    let sig = sign(
        m.key,
        &message_signing_input(&label, m.binding_pub, &plain_bytes, &payload),
    );
    WireMessage {
        plain: b64u(&plain_bytes),
        payload: b64u(&payload),
        sig: b64u(&sig),
    }
}

#[derive(Debug, Clone)]
pub struct ReadMessage {
    pub plain: Plain,
    pub plain_bytes: Vec<u8>,
    pub payload_bytes: Vec<u8>,
    pub sig: Vec<u8>,
}

/// Parses `plain` strictly; `deadline` absent only on 1130 and 1131;
/// `nonce` present exactly when sealed, 24 bytes.
pub fn read_message(w: &WireMessage) -> Result<ReadMessage, ProtocolError> {
    let plain_bytes = from_b64u(&w.plain)?;
    let p = strict(&plain_bytes, "plain")?;
    if !p.value.is_object() {
        return Err(ProtocolError::malformed("plain is not an object"));
    }
    let not_ijson = |e: super::ijson::IJsonError| ProtocolError::NotIJson(e.0);
    let tag = p.int("/tag").map_err(not_ijson)?;
    let seq = p.int("/seq").map_err(not_ijson)?;
    let deadline = p.opt_int("/deadline").map_err(not_ijson)?;
    let sealed = p.value["sealed"]
        .as_bool()
        .ok_or_else(|| ProtocolError::malformed("sealed"))?;
    if tag < 0
        || tag > u32::MAX as i64
        || seq < 0
        || deadline.is_some_and(|d| d < 0)
    {
        return Err(ProtocolError::malformed("negative field"));
    }
    let tag = tag as u32;
    let handshake = tag == tags::PROPOSAL || tag == tags::APPROVAL;
    if handshake == deadline.is_some() {
        return Err(ProtocolError::malformed("deadline presence"));
    }
    let nonce = match &p.value["nonce"] {
        Value::Null => None,
        Value::String(s) => Some(s.clone()),
        _ => return Err(ProtocolError::malformed("nonce")),
    };
    if sealed != nonce.is_some() {
        return Err(ProtocolError::malformed("nonce presence"));
    }
    if let Some(n) = &nonce
        && from_b64u(n)?.len() != 24
    {
        return Err(ProtocolError::malformed("nonce length"));
    }
    Ok(ReadMessage {
        plain: Plain {
            tag,
            seq: seq as u64,
            sealed,
            deadline: deadline.map(|d| d as u64),
            nonce,
        },
        plain_bytes,
        payload_bytes: from_b64u(&w.payload)?,
        sig: from_b64u(&w.sig)?,
    })
}

pub fn verify_message(
    public: &[u8],
    version: u32,
    binding_pub: &[u8],
    m: &ReadMessage,
) -> bool {
    let label = label_for(version, m.plain.tag);
    verify_strict(
        public,
        &m.sig,
        &message_signing_input(
            &label,
            &binding_arr(binding_pub),
            &m.plain_bytes,
            &m.payload_bytes,
        ),
    )
}

pub fn read_rpc(bytes: &[u8]) -> Result<Parsed, ProtocolError> {
    let p = strict(bytes, "payload")?;
    if p.value.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Err(ProtocolError::malformed("not JSON-RPC 2.0"));
    }
    p.int("/id").map_err(|e| ProtocolError::NotIJson(e.0))?;
    if p.value.get("method").and_then(Value::as_str) == Some("wc_sessionExtend")
    {
        p.int("/params/expiry").map_err(|e| ProtocolError::NotIJson(e.0))?;
    }
    let is_request = p.value.get("method").is_some_and(Value::is_string);
    if !is_request
        && p.value.get("result").is_none()
        && p.value.get("error").is_none()
    {
        return Err(ProtocolError::malformed("rpc shape"));
    }
    Ok(p)
}

/// Verify first (the caller), then decrypt; any failure drops the message.
pub fn open_message_payload(
    m: &ReadMessage,
    version: u32,
    binding_pub: &[u8],
    key: Option<&[u8; 32]>,
) -> Result<Parsed, ProtocolError> {
    if !m.plain.sealed {
        return read_rpc(&m.payload_bytes);
    }
    let key = key.ok_or(ProtocolError::CannotDecrypt)?;
    let nonce: [u8; 24] =
        from_b64u(m.plain.nonce.as_deref().unwrap_or_default())?
            .try_into()
            .map_err(|_| ProtocolError::CannotDecrypt)?;
    let ad = seal_ad(
        &label_for(version, m.plain.tag),
        &binding_arr(binding_pub),
        &m.plain_bytes,
    );
    read_rpc(&open(key, &nonce, &m.payload_bytes, &ad)?)
}

pub fn is_request(rpc: &Value) -> bool {
    rpc.get("method").is_some_and(Value::is_string)
}
