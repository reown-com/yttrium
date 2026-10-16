use {super::ProtocolError, zeroize::Zeroizing};

/// The v2 URI with the connectionId in place of the topic.
pub fn build_uri(
    connection_id: &str,
    sym: &[u8; 32],
    expires_at_s: u64,
) -> String {
    format!(
        "wc:{connection_id}@2?relay-protocol=irn&symKey={}&expiryTimestamp={expires_at_s}",
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
    /// A v2 pairing URI: its 64 hex topic is in place of the connectionId.
    pub legacy: bool,
}

pub fn parse_uri(uri: &str) -> Result<ParsedUri, ProtocolError> {
    let rest = uri
        .trim()
        .strip_prefix("wc:")
        .ok_or_else(|| ProtocolError::malformed("not a wc uri"))?;
    let (id, query) = rest
        .split_once("@2?")
        .ok_or_else(|| ProtocolError::malformed("not a v2-shaped uri"))?;
    let legacy = id.len() == 64 && id.bytes().all(|b| b.is_ascii_hexdigit());
    if !legacy && !is_connection_id(id) {
        return Err(ProtocolError::malformed("connectionId"));
    }
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
            _ => {}
        }
    }
    Ok(ParsedUri {
        connection_id: if legacy { id.to_ascii_lowercase() } else { id.into() },
        sym: sym.ok_or_else(|| ProtocolError::malformed("symKey missing"))?,
        relay_protocol,
        expiry_timestamp: expiry,
        legacy,
    })
}
