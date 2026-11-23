//! Verify v4 App Signatures.
//!
//! The dApp's backend signs each outgoing message as a JWT, the
//! `appSignature`, which the dApp sends next to the message. The Relay
//! verifies it against the dApp's domain and hands the wallet the JWT
//! verbatim plus `appMetadata`. The wallet doesn't verify the JWT's
//! signature: it only checks that the JWT is bound to the message it got.
//!
//! What the claims cover:
//! - `sub`: base64url, no padding, of SHA-256 over the plaintext JSON-RPC
//!   bytes. They are the bytes the SDK serializes the JSON-RPC object to,
//!   before any sealing: for a monitored message, exactly the base64url
//!   decoded `payload`; for a sealed one, the bytes the payload decrypts to.
//! - `cth`, sealed messages only: base64url, no padding, of SHA-256 over the
//!   base64url decoded `payload` as sent, that is, the ciphertext.

use {
    super::{
        encoding::{b64u, from_b64u},
        ijson,
    },
    serde::{Deserialize, Serialize},
    serde_json::Value,
    sha2::{Digest, Sha256},
};

/// base64url (no padding) SHA-256, the encoding of `sub` and `cth`.
pub fn hash_b64u(bytes: &[u8]) -> String {
    b64u(&Sha256::digest(bytes))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSignatureClaims {
    pub sub: String,
    pub cth: Option<String>,
}

/// Decodes the JWT's claims without checking its signature: the Relay did.
pub fn decode_claims(jwt: &str) -> Option<AppSignatureClaims> {
    let mut parts = jwt.split('.');
    let (Some(_), Some(claims), Some(_), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return None;
    };
    let v = ijson::value(&from_b64u(claims.trim_end_matches('=')).ok()?)?;
    let cth = match v.get("cth") {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) => Some(s.clone()),
        Some(_) => return None,
    };
    Some(AppSignatureClaims { sub: v.get("sub")?.as_str()?.into(), cth })
}

/// What the Relay says about the app, from its registry and screening.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppMetadata {
    pub origin: Option<String>,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub is_verified: Option<bool>,
    pub is_scam: Option<bool>,
    pub is_similar_to: Option<String>,
}

impl AppMetadata {
    /// Field by field, so one odd field doesn't lose the others.
    pub fn from_value(v: &Value) -> Option<Self> {
        if !v.is_object() {
            return None;
        }
        let s = |k: &str| v.get(k).and_then(Value::as_str).map(str::to_string);
        let b = |k: &str| v.get(k).and_then(Value::as_bool);
        Some(Self {
            origin: s("origin"),
            name: s("name"),
            icon: s("icon"),
            is_verified: b("isVerified"),
            is_scam: b("isScam"),
            is_similar_to: s("isSimilarTo"),
        })
    }
}

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize,
)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[serde(rename_all = "snake_case")]
pub enum VerifyState {
    /// Unsigned: nothing is known about the app.
    #[default]
    Default,
    /// Signed by a key the domain published.
    DomainValidated,
    /// As above, and the Relay knows the business's name.
    BusinessName,
    /// As above, and the business is verified.
    Verified,
    /// Signed, but the signature is not for this message.
    Invalid,
}

/// What the wallet shows about the app. Identity comes only from the Relay's
/// `appMetadata`, never from the app's own metadata.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct VerifyContext {
    pub state: VerifyState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_scam: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_similar_to: Option<String>,
}

/// Is the JWT bound to these bytes: `sub` to the plaintext and, when present,
/// `cth` to the payload as received?
pub fn is_bound(jwt: &str, plaintext: &[u8], payload: &[u8]) -> bool {
    decode_claims(jwt).is_some_and(|c| {
        c.sub == hash_b64u(plaintext)
            && c.cth.is_none_or(|cth| cth == hash_b64u(payload))
    })
}

/// The verify context of one received message or proposal. `plaintext` is
/// the JSON-RPC bytes after decoding or decrypting; `payload` the base64url
/// decoded `payload` as received (the same bytes when monitored).
pub fn verify_context(
    app_signature: Option<&str>,
    app_metadata: Option<&Value>,
    plaintext: &[u8],
    payload: &[u8],
) -> VerifyContext {
    let md = app_metadata.and_then(AppMetadata::from_value).unwrap_or_default();
    let screening = VerifyContext {
        is_scam: md.is_scam,
        is_similar_to: md.is_similar_to.clone(),
        ..Default::default()
    };
    let Some(jwt) = app_signature else {
        // An unsigned request has no identity: an origin here is ignored.
        return screening;
    };
    if !is_bound(jwt, plaintext, payload) {
        // The metadata described another message: drop all of it.
        return VerifyContext {
            state: VerifyState::Invalid,
            ..Default::default()
        };
    }
    let Some(origin) = md.origin else { return screening };
    let state = if md.is_verified == Some(true) {
        VerifyState::Verified
    } else if md.name.is_some() {
        VerifyState::BusinessName
    } else {
        VerifyState::DomainValidated
    };
    VerifyContext {
        state,
        origin: Some(origin),
        name: md.name,
        icon: md.icon,
        ..screening
    }
}

#[cfg(test)]
mod tests {
    use {super::*, serde_json::json};

    fn jwt(claims: Value) -> String {
        format!(
            "{}.{}.{}",
            b64u(br#"{"alg":"ES256","typ":"JWT"}"#),
            b64u(claims.to_string().as_bytes()),
            b64u(b"not checked")
        )
    }

    const RPC: &[u8] = br#"{"id":1,"jsonrpc":"2.0","method":"wc_sessionRequest","params":{"chainId":"eip155:1","request":{"method":"personal_sign","params":["0x68656c6c6f","0xabc"]}}}"#;

    #[test]
    fn hashes_are_base64url_sha256_without_padding() {
        // SHA-256("abc"), RFC 6234.
        assert_eq!(
            hash_b64u(b"abc"),
            b64u(
                &hex::decode(
                    "ba7816bf8f01cfea414140de5dae2223\
                     b00361a396177a9cb410ff61f20015ad"
                )
                .unwrap()
            )
        );
        assert_eq!(
            hash_b64u(b"abc"),
            "ungWv48Bz-pBQUDeXa4iI7ADYaOWF3qctBD_YfIAFa0"
        );
        assert!(!hash_b64u(RPC).contains('='));
    }

    #[test]
    fn decodes_claims_without_checking_the_signature() {
        let c = decode_claims(&jwt(
            json!({ "iss": "app.example", "sub": "s", "cth": "c", "exp": 1 }),
        ))
        .unwrap();
        assert_eq!(c.sub, "s");
        assert_eq!(c.cth.as_deref(), Some("c"));
        assert!(decode_claims("a.b").is_none());
        assert!(decode_claims("a.b.c.d").is_none());
        assert!(decode_claims(&jwt(json!({ "iss": "x" }))).is_none());
        assert!(decode_claims(&jwt(json!({ "sub": "s", "cth": 1 }))).is_none());
    }

    #[test]
    fn derives_the_state_from_the_metadata_present() {
        let sig = jwt(json!({ "sub": hash_b64u(RPC) }));
        let ctx = |md: Value| verify_context(Some(&sig), Some(&md), RPC, RPC);
        let o = "https://app.example";
        assert_eq!(
            ctx(json!({ "origin": o })),
            VerifyContext {
                state: VerifyState::DomainValidated,
                origin: Some(o.into()),
                ..Default::default()
            }
        );
        let named = ctx(json!({ "origin": o, "name": "App", "icon": "i" }));
        assert_eq!(named.state, VerifyState::BusinessName);
        assert_eq!(named.name.as_deref(), Some("App"));
        assert_eq!(
            ctx(json!({ "origin": o, "name": "App", "isVerified": true }))
                .state,
            VerifyState::Verified
        );
        let scam = ctx(json!({ "origin": o, "isScam": true }));
        assert_eq!(scam.state, VerifyState::DomainValidated);
        assert_eq!(scam.is_scam, Some(true));
        // Signed, but the Relay knew nothing about the app.
        assert_eq!(
            verify_context(Some(&sig), None, RPC, RPC).state,
            VerifyState::Default
        );
    }

    #[test]
    fn unsigned_requests_keep_only_screening() {
        let md = json!({ "origin": "https://evil.example", "name": "Bank", "isScam": true, "isSimilarTo": "bank.example" });
        assert_eq!(
            verify_context(None, Some(&md), RPC, RPC),
            VerifyContext {
                state: VerifyState::Default,
                is_scam: Some(true),
                is_similar_to: Some("bank.example".into()),
                ..Default::default()
            }
        );
        assert_eq!(verify_context(None, None, RPC, RPC), Default::default());
    }

    #[test]
    fn a_mismatched_sub_is_invalid_and_drops_the_metadata() {
        let md = json!({ "origin": "https://app.example", "name": "App", "isVerified": true, "isScam": false });
        let other = jwt(json!({ "sub": hash_b64u(b"another request") }));
        assert_eq!(
            verify_context(Some(&other), Some(&md), RPC, RPC),
            VerifyContext { state: VerifyState::Invalid, ..Default::default() }
        );
        // One byte of difference: a re-serialized request doesn't match.
        let respaced =
            String::from_utf8(RPC.to_vec()).unwrap().replacen(":", ": ", 1);
        let sig = jwt(json!({ "sub": hash_b64u(respaced.as_bytes()) }));
        assert_eq!(
            verify_context(Some(&sig), Some(&md), RPC, RPC).state,
            VerifyState::Invalid
        );
        for garbage in ["", "x", "a.b.c", "a.!!!.c"] {
            assert_eq!(
                verify_context(Some(garbage), Some(&md), RPC, RPC).state,
                VerifyState::Invalid
            );
        }
    }

    #[test]
    fn a_sealed_message_checks_cth_against_the_payload() {
        let ciphertext = b"opaque ciphertext";
        let md = json!({ "origin": "https://app.example" });
        let sig = |cth: &[u8]| {
            jwt(json!({ "sub": hash_b64u(RPC), "cth": hash_b64u(cth) }))
        };
        assert_eq!(
            verify_context(Some(&sig(ciphertext)), Some(&md), RPC, ciphertext)
                .state,
            VerifyState::DomainValidated
        );
        assert_eq!(
            verify_context(Some(&sig(b"other")), Some(&md), RPC, ciphertext)
                .state,
            VerifyState::Invalid
        );
    }

    #[test]
    fn serializes_as_the_wallet_ui_shape() {
        let c = VerifyContext {
            state: VerifyState::BusinessName,
            origin: Some("https://app.example".into()),
            name: Some("App".into()),
            is_similar_to: None,
            ..Default::default()
        };
        assert_eq!(
            serde_json::to_value(&c).unwrap(),
            json!({ "state": "business_name", "origin": "https://app.example", "name": "App" })
        );
        assert_eq!(
            serde_json::to_value(VerifyContext::default()).unwrap(),
            json!({ "state": "default" })
        );
    }
}
