use {
    super::{ProtocolError, crypto::sha256_hex},
    zeroize::Zeroizing,
};

/// A valid v2 pairing URI, with the connectionId in the `v3` parameter. The topic is v2's pairing
/// topic for the symKey, so a v2 wallet can pair on the same URI.
pub fn build_uri(
    connection_id: &str,
    sym: &[u8; 32],
    expires_at_s: u64,
) -> String {
    format!(
        "wc:{}@2?relay-protocol=irn&symKey={}&expiryTimestamp={expires_at_s}&v3={connection_id}",
        sha256_hex(sym),
        hex::encode(sym)
    )
}

/// Region digit followed by a random u64, as minted by the Relay at create.
pub fn is_connection_id(s: &str) -> bool {
    let b = s.as_bytes();
    (2..=21).contains(&b.len())
        && matches!(b[0], b'0'..=b'2')
        && b.iter().all(u8::is_ascii_digit)
}

pub struct ParsedUri {
    pub connection_id: String,
    pub sym: Zeroizing<[u8; 32]>,
    pub relay_protocol: String,
    pub expiry_timestamp: u64,
    /// A v2 pairing URI: it has no `v3` parameter, and `connection_id` is its topic.
    pub legacy: bool,
}

pub fn parse_uri(uri: &str) -> Result<ParsedUri, ProtocolError> {
    let rest = uri
        .trim()
        .strip_prefix("wc:")
        .ok_or_else(|| ProtocolError::malformed("not a wc uri"))?;
    let (topic, query) = rest
        .split_once("@2?")
        .ok_or_else(|| ProtocolError::malformed("not a v2-shaped uri"))?;
    if topic.len() != 64 || !topic.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(ProtocolError::malformed("topic"));
    }
    let mut connection_id = None;
    let mut sym = None;
    let mut relay_protocol = "irn".to_string();
    let mut expiry = 0;
    for pair in query.split('&') {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        match k {
            "symKey" => {
                let b = hex::decode(v)
                    .map_err(|_| ProtocolError::malformed("symKey"))?;
                sym = Some(Zeroizing::new(
                    <[u8; 32]>::try_from(b.as_slice())
                        .map_err(|_| ProtocolError::malformed("symKey"))?,
                ));
            }
            "relay-protocol" => relay_protocol = v.to_string(),
            "expiryTimestamp" => expiry = v.parse().unwrap_or(0),
            "v3" => {
                if !is_connection_id(v) {
                    return Err(ProtocolError::malformed("connectionId"));
                }
                connection_id = Some(v.to_string());
            }
            _ => {}
        }
    }
    Ok(ParsedUri {
        legacy: connection_id.is_none(),
        connection_id: connection_id
            .unwrap_or_else(|| topic.to_ascii_lowercase()),
        sym: sym.ok_or_else(|| ProtocolError::malformed("symKey missing"))?,
        relay_protocol,
        expiry_timestamp: expiry,
    })
}
