use crate::sign::{
    client_types::Session,
    utils::{
        compute_max_expiry, validate_extend_request, ExtendValidationError,
    },
};

fn make_session(
    expiry: u64,
    controller_key: Option<[u8; 32]>,
    peer_public_key: Option<[u8; 32]>,
) -> Session {
    yttrium::sign::client_types::Session {
        request_id: 0,
        topic: "deadbeef".into(),
        expiry,
        relay_protocol: "irn".into(),
        relay_data: None,
        controller_key,
        session_sym_key: [0u8; 32],
        self_public_key: [1u8; 32],
        self_meta_data: yttrium::sign::protocol_types::Metadata {
            name: "".into(),
            description: "".into(),
            url: "".into(),
            icons: vec![],
            verify_url: None,
            redirect: None,
        },
        peer_public_key,
        peer_meta_data: None,
        session_namespaces: std::collections::HashMap::new(),
        required_namespaces: std::collections::HashMap::new(),
        optional_namespaces: None,
        session_properties: None,
        scoped_properties: None,
        is_acknowledged: false,
        pairing_topic: "deadbeef".into(),
        transport_type: None,
    }
}

#[test]
fn test_extend_peer_unauthorized_rejected() {
    let now = 1_700_000_000u64;
    let session =
        make_session(now + 24 * 3600, Some([9u8; 32]), Some([8u8; 32]));
    let requested = now + 2 * 24 * 3600;
    let res = validate_extend_request(&session, requested, now);
    assert_eq!(res, Err(ExtendValidationError::Unauthorized));
}

#[test]
fn test_extend_ttl_too_high_rejected() {
    let now = 1_700_000_000u64;
    let controller = [9u8; 32];
    let session =
        make_session(now + 24 * 3600, Some(controller), Some(controller));
    let requested = now + 10 * 24 * 3600;
    let res = validate_extend_request(&session, requested, now);
    assert_eq!(res, Err(ExtendValidationError::ExpiryTooHigh));
}

#[test]
fn test_extend_ttl_too_low_rejected() {
    let now = 1_700_000_000u64;
    let controller = [9u8; 32];
    let session =
        make_session(now + 2 * 24 * 3600, Some(controller), Some(controller));
    let requested = now + 24 * 3600;
    let res = validate_extend_request(&session, requested, now);
    assert_eq!(res, Err(ExtendValidationError::ExpiryTooLow));
}

#[test]
fn test_extend_valid_updates() {
    let now = 1_700_000_000u64;
    let controller = [9u8; 32];
    let session =
        make_session(now + 24 * 3600, Some(controller), Some(controller));
    let requested = now + 6 * 24 * 3600;
    let res = validate_extend_request(&session, requested, now).unwrap();
    assert_eq!(res, requested);
    assert!(res <= compute_max_expiry(now));
}
