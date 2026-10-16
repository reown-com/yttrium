use {
    super::ProtocolError,
    base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD},
};

pub fn b64u(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn from_b64u(s: &str) -> Result<Vec<u8>, ProtocolError> {
    URL_SAFE_NO_PAD
        .decode(s)
        .map_err(|_| ProtocolError::malformed("not base64url"))
}
