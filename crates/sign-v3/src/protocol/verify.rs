//! Verify v4 App Signatures.
//!
//! The dApp's backend signs each outgoing message as a JWT, the
//! `appSignature`, which the dApp sends next to the message. The Relay
//! verifies it against the dApp's domain and hands the wallet `appMetadata`,
//! with the JWT verbatim as `appMetadata.appSignature`. The wallet doesn't verify the JWT's
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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppMetadata {
    /// The app's JWT, verbatim. Kept out of `AppVerification`: the wallet
    /// only needs what it proves.
    pub app_signature: Option<String>,
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
            app_signature: s("appSignature"),
            origin: s("origin"),
            name: s("name"),
            icon: s("icon"),
            is_verified: b("isVerified"),
            is_scam: b("isScam"),
            is_similar_to: s("isSimilarTo"),
        })
    }
}

/// What the wallet shows about the app: every field comes from the Relay's
/// `appMetadata`, after the SDK checked that the app's signature is for this
/// message. Never from the app's own metadata. The wallet shows what's present.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[serde(rename_all = "camelCase")]
pub struct AppVerification {
    /// The domain the Relay verified the signature for, e.g.
    /// `app.uniswap.org`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// Only when WalletConnect reviewed the app.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Only when WalletConnect reviewed the app; hosted by WalletConnect.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default)]
    pub is_verified: bool,
    #[serde(default)]
    pub is_scam: bool,
    /// A domain this app looks like.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_similar_to: Option<String>,
}

/// The JSON-RPC error a wallet answers a mismatched request with:
/// `{ code: -32600, message: "app_signature_mismatch" }`.
pub const APP_SIGNATURE_MISMATCH_CODE: i64 = -32600;

/// The app's signature is not for this message: `sub` or `cth` differ, or
/// the JWT doesn't decode. The message is dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("app_signature_mismatch")]
pub struct AppSignatureMismatch;

/// Is the JWT bound to these bytes: `sub` to the plaintext and, when present,
/// `cth` to the payload as received?
pub fn is_bound(jwt: &str, plaintext: &[u8], payload: &[u8]) -> bool {
    decode_claims(jwt).is_some_and(|c| {
        c.sub == hash_b64u(plaintext)
            && c.cth.is_none_or(|cth| cth == hash_b64u(payload))
    })
}

/// `https://app.example.com` → `app.example.com`.
fn domain_of(origin: &str) -> String {
    let rest = origin.split_once("://").map_or(origin, |(_, r)| r);
    rest.split('/').next().unwrap_or_default().to_ascii_lowercase()
}

/// The verification of one received message or proposal. `plaintext` is the
/// JSON-RPC bytes after decoding or decrypting; `payload` the base64url
/// decoded `payload` as received (the same bytes when monitored).
pub fn app_verification(
    app_metadata: Option<&Value>,
    plaintext: &[u8],
    payload: &[u8],
) -> Result<AppVerification, AppSignatureMismatch> {
    let md = app_metadata.and_then(AppMetadata::from_value).unwrap_or_default();
    let screening = AppVerification {
        is_scam: md.is_scam.unwrap_or(false),
        is_similar_to: md.is_similar_to.clone(),
        ..Default::default()
    };
    let Some(jwt) = md.app_signature.as_deref() else {
        // Unsigned: no identity. An origin, name or icon here is ignored.
        return Ok(screening);
    };
    if !is_bound(jwt, plaintext, payload) {
        return Err(AppSignatureMismatch);
    }
    Ok(AppVerification {
        domain: md.origin.as_deref().map(domain_of),
        name: md.name,
        icon: md.icon,
        is_verified: md.is_verified == Some(true),
        ..screening
    })
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

    /// `appMetadata` as the Relay delivers it, with the JWT inside.
    fn signed(sig: &str, mut md: Value) -> Value {
        md["appSignature"] = sig.into();
        md
    }

    #[test]
    fn takes_every_field_from_the_relay_once_bound() {
        let sig = jwt(json!({ "sub": hash_b64u(RPC) }));
        let ver = |md: Value| {
            app_verification(Some(&signed(&sig, md)), RPC, RPC).unwrap()
        };
        assert_eq!(
            ver(json!({ "origin": "https://App.Example.com" })),
            AppVerification {
                domain: Some("app.example.com".into()),
                ..Default::default()
            }
        );
        assert_eq!(
            ver(
                json!({ "origin": "https://app.example.com", "name": "App", "icon": "https://i", "isVerified": true, "isScam": true, "isSimilarTo": "app.exarnple.com" })
            ),
            AppVerification {
                domain: Some("app.example.com".into()),
                name: Some("App".into()),
                icon: Some("https://i".into()),
                is_verified: true,
                is_scam: true,
                is_similar_to: Some("app.exarnple.com".into()),
            }
        );
        // Signed, but the Relay knew nothing about the app.
        assert_eq!(
            app_verification(Some(&signed(&sig, json!({}))), RPC, RPC).unwrap(),
            AppVerification::default()
        );
    }

    #[test]
    fn unsigned_requests_keep_only_screening() {
        let md = json!({ "origin": "https://evil.example", "name": "Bank", "icon": "i", "isVerified": true, "isScam": true, "isSimilarTo": "bank.example" });
        assert_eq!(
            app_verification(Some(&md), RPC, RPC).unwrap(),
            AppVerification {
                is_scam: true,
                is_similar_to: Some("bank.example".into()),
                ..Default::default()
            }
        );
        assert_eq!(
            app_verification(None, RPC, RPC).unwrap(),
            AppVerification::default()
        );
    }

    #[test]
    fn a_mismatched_sub_is_an_error() {
        let md = json!({ "origin": "https://app.example", "name": "App", "isVerified": true });
        let other = jwt(json!({ "sub": hash_b64u(b"another request") }));
        assert_eq!(
            app_verification(Some(&signed(&other, md.clone())), RPC, RPC),
            Err(AppSignatureMismatch)
        );
        // One byte of difference: a re-serialized request doesn't match.
        let respaced =
            String::from_utf8(RPC.to_vec()).unwrap().replacen(":", ": ", 1);
        let sig = jwt(json!({ "sub": hash_b64u(respaced.as_bytes()) }));
        assert_eq!(
            app_verification(Some(&signed(&sig, md.clone())), RPC, RPC),
            Err(AppSignatureMismatch)
        );
        for garbage in ["", "x", "a.b.c", "a.!!!.c"] {
            assert_eq!(
                app_verification(Some(&signed(garbage, md.clone())), RPC, RPC),
                Err(AppSignatureMismatch)
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
            app_verification(
                Some(&signed(&sig(ciphertext), md.clone())),
                RPC,
                ciphertext
            )
            .unwrap()
            .domain
            .as_deref(),
            Some("app.example")
        );
        assert_eq!(
            app_verification(
                Some(&signed(&sig(b"other"), md.clone())),
                RPC,
                ciphertext
            ),
            Err(AppSignatureMismatch)
        );
    }

    #[test]
    fn serializes_as_the_wallet_ui_shape() {
        let v = AppVerification {
            domain: Some("app.example".into()),
            name: Some("App".into()),
            is_verified: true,
            ..Default::default()
        };
        assert_eq!(
            serde_json::to_value(&v).unwrap(),
            json!({ "domain": "app.example", "name": "App", "isVerified": true, "isScam": false })
        );
        assert_eq!(
            serde_json::to_value(AppVerification::default()).unwrap(),
            json!({ "isVerified": false, "isScam": false })
        );
    }
}
