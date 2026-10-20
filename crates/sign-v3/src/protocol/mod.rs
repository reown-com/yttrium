//! The pure protocol layer: spec types, strict I-JSON, keys and labels, signed objects, messages,
//! sealing, intents, the `wc:` URI and the replay window. It imports nothing from the rest of the
//! crate, so it can become its own crate if ever needed.

pub mod constants;
pub mod crypto;
pub mod encoding;
pub mod ijson;
pub mod intent;
pub mod message;
pub mod replay;
pub mod types;
pub mod uri;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProtocolError {
    #[error("malformed: {0}")]
    Malformed(String),
    #[error("not I-JSON: {0}")]
    NotIJson(String),
    #[error("bad signature under {0}")]
    BadSignature(String),
    #[error("cannot decrypt")]
    CannotDecrypt,
}

impl ProtocolError {
    pub fn malformed(m: impl Into<String>) -> Self {
        Self::Malformed(m.into())
    }
}

#[cfg(test)]
mod spec_vectors {
    //! deep-dives/cryptography.md, "Test vectors" and "Negative vectors".
    use {
        super::{
            crypto::*,
            encoding::*,
            message::*,
            types::{DappIntent, Mode, PeerRequirement, tags},
        },
        ed25519_dalek::SigningKey,
        serde_json::json,
    };

    fn h(s: &str) -> Vec<u8> {
        hex::decode(s).unwrap()
    }
    fn h32(s: &str) -> [u8; 32] {
        h(s).try_into().unwrap()
    }

    const SYM: &str =
        "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
    const D_SEED: &str =
        "202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f";
    const W_SEED: &str =
        "404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f";

    struct Fx {
        sym: [u8; 32],
        binding: SigningKey,
        bpub: [u8; 32],
        dapp: SigningKey,
        wallet: SigningKey,
    }

    fn fx() -> Fx {
        let sym = h32(SYM);
        let binding = derive_binding_key(&sym);
        let bpub = binding.verifying_key().to_bytes();
        Fx {
            sym,
            binding,
            bpub,
            dapp: SigningKey::from_bytes(&h32(D_SEED)),
            wallet: SigningKey::from_bytes(&h32(W_SEED)),
        }
    }

    #[test]
    fn derives_keys_and_sealing_keys() {
        let f = fx();
        assert_eq!(
            hex::encode(f.binding.to_bytes()),
            "4862d66e8400fd42101d9d6408ef59ac91484b8694cefb8d2ead33e276253586"
        );
        assert_eq!(
            hex::encode(f.bpub),
            "6cfb054182b640109face0a80b308f0015b45643afe194ff3c416fb00278dd07"
        );
        let dp = f.dapp.verifying_key().to_bytes();
        let wp = f.wallet.verifying_key().to_bytes();
        assert_eq!(
            hex::encode(dp),
            "29acbae141bccaf0b22e1a94d34d0bc7361e526d0bfe12c89794bc9322966dd7"
        );
        assert_eq!(
            hex::encode(wp),
            "2543b92ff1095511476adc8369db6ddc933665a11978dda1404ee1066ca9559d"
        );
        let k = derive_sealing_keys(3, &f.dapp, &wp, &f.sym, &f.bpub, &dp, &wp)
            .unwrap();
        let kw =
            derive_sealing_keys(3, &f.wallet, &dp, &f.sym, &f.bpub, &dp, &wp)
                .unwrap();
        assert_eq!(
            hex::encode(*k.d2w),
            "7ca81f3549001ba8cac7480cd1ef9804f46ad0dedfd66081210a3212a1f29795"
        );
        assert_eq!(
            hex::encode(*k.w2d),
            "9f6972c4a5a3f816e137b42e790da4c2e5b2e4330dfdaf236f603a1170f3d936"
        );
        assert_eq!(*k.d2w, *kw.d2w);
        assert_eq!(*k.w2d, *kw.w2d);
    }

    #[test]
    fn reproduces_the_binding_proof() {
        let f = fx();
        let payload = super::types::BindingProofPayload {
            versions: vec![3],
            intent: DappIntent {
                own: Mode::Sealed,
                peer: PeerRequirement::None,
            },
            pairing_pub: b64u(&f.dapp.verifying_key().to_bytes()),
        };
        let proof =
            build_signed(&f.binding, labels::BINDING_PROOF, &f.bpub, &payload);
        assert_eq!(
            String::from_utf8(from_b64u(&proof.payload).unwrap()).unwrap(),
            r#"{"versions":[3],"intent":{"own":"sealed","peer":"none"},"pairingPub":"Kay64UG8yvCyLhqU000LxzYeUm0L_hLIl5S8kyKWbdc"}"#
        );
        assert_eq!(
            hex::encode(from_b64u(&proof.sig).unwrap()),
            "643e240c0e492c399714dbc1fd13217b4d3629a2eb4ebce360c21966eeb32e4c42fe6823afaf2dc3c448c3a55aeca687c4b60f741b63adcba1ac849b175d320f"
        );
        assert!(
            open_signed(&f.bpub, labels::BINDING_PROOF, &f.bpub, &proof)
                .is_ok()
        );
    }

    #[test]
    fn reproduces_the_sealed_request_and_decrypts_it() {
        let f = fx();
        let dp = f.dapp.verifying_key().to_bytes();
        let wp = f.wallet.verifying_key().to_bytes();
        let k = derive_sealing_keys(3, &f.dapp, &wp, &f.sym, &f.bpub, &dp, &wp)
            .unwrap();
        let rpc = json!({"id":1,"jsonrpc":"2.0","method":"wc_sessionRequest","params":{}});
        let w = build_message(BuildMessage {
            key: &f.dapp,
            version: 3,
            binding_pub: &f.bpub,
            tag: tags::REQUEST,
            seq: 1,
            deadline: Some(1759313400),
            rpc: &rpc,
            seal_key: Some(&k.d2w),
            nonce: Some(
                h("606162636465666768696a6b6c6d6e6f7071727374757677")
                    .try_into()
                    .unwrap(),
            ),
        });
        assert_eq!(
            String::from_utf8(from_b64u(&w.plain).unwrap()).unwrap(),
            r#"{"tag":1108,"seq":1,"sealed":true,"deadline":1759313400,"nonce":"YGFiY2RlZmdoaWprbG1ub3BxcnN0dXZ3"}"#
        );
        assert_eq!(
            hex::encode(from_b64u(&w.payload).unwrap()),
            "331052d5735bd337c45e259421628a8eb10f01fadfb495e6728c3a81d11ccce69b56b984088e4aa2a175c7958371b483ba59491fb8e4e0ed22296488d0f5893d09b90f0c171ce8300d63c839a7be7cd45e"
        );
        assert_eq!(
            hex::encode(from_b64u(&w.sig).unwrap()),
            "5a65bbf8589a9723f4a4f9f769a38a579b840ff039fd88cbf736984a96d23cd689a41f5f0472fcc1cfb59599adfd5dba2b009ed0a226b04b87e4f15a7c2e3f07"
        );
        let r = read_message(&w).unwrap();
        assert!(verify_message(&dp, 3, &f.bpub, &r));
        assert!(!verify_message(&wp, 3, &f.bpub, &r));
        assert_eq!(
            open_message_payload(&r, 3, &f.bpub, Some(&k.d2w)).unwrap().value,
            rpc
        );
        assert!(open_message_payload(&r, 3, &f.bpub, Some(&k.w2d)).is_err());
    }

    #[test]
    fn reproduces_the_fetch_and_close_signatures() {
        let f = fx();
        // The vector signs a topic; the SDK signs the connectionId the same way.
        let topic =
            "630dcd2966c4336691125448bbb25b4ff412a49c732db2c8abc1b8581bd710dd";
        assert_eq!(
            hex::encode(
                from_b64u(&sign_bytes(
                    &f.binding,
                    labels::FETCH,
                    &f.bpub,
                    topic.as_bytes()
                ))
                .unwrap()
            ),
            "3376f9c8ed422cf183f50752cf01fa77ea9567d64ef9fd90cc561a1b389f574bfe3c1217c567e3e5c8b39269e7474279ffdd47bc2e8c2bb5dce105f178fdd70c"
        );
        assert_eq!(
            hex::encode(
                from_b64u(&sign_bytes(&f.dapp, &close_label(3), &f.bpub, &[]))
                    .unwrap()
            ),
            "d4f713d7439f76f4f095d29e29964159bee5030a00891790d098352a394dacb6c3c8a89ab90db9093ac3c65fad0a82fe33c7f41645fa6ec2f8dc99f85f6d7b00"
        );
    }

    #[test]
    fn refuses_the_negative_vectors() {
        let f = fx();
        assert!(!is_valid_pairing_pub(&h(
            "0100000000000000000000000000000000000000000000000000000000000000"
        )));
        assert!(!is_valid_pairing_pub(&h(
            "2144432c47d6e1014b97775a774d49204d42afe004ebb2186bdeb23f5f117dfd"
        )));
        assert!(is_valid_pairing_pub(&f.dapp.verifying_key().to_bytes()));
        let bad = h(
            "c937e1e23b9f8547531c56a8be62fbc68d36d4518c63a4f831e8707c22e1c5f80c992667227ac0f1bd3a20d8354cdfd373f5ea83084461beb051281aeb813d06",
        );
        assert!(!verify_bytes(
            &f.dapp.verifying_key().to_bytes(),
            &b64u(&bad),
            &close_label(3),
            &[0u8; 32],
            &[]
        ));
    }

    #[test]
    fn rejects_malformed_plain_even_with_a_valid_signature() {
        let forge = |plain: &str| super::types::WireMessage {
            plain: b64u(plain.as_bytes()),
            payload: b64u(b"{}"),
            sig: "AA".into(),
        };
        assert!(
            read_message(&forge(
                r#"{"tag":1114,"seq":1.0,"sealed":false,"deadline":1}"#
            ))
            .is_err()
        );
        assert!(
            read_message(&forge(
                r#"{"tag":1114,"seq":1,"seq":2,"sealed":false,"deadline":1}"#
            ))
            .is_err()
        );
        assert!(
            read_message(&forge(r#"{"tag":1114,"seq":1,"sealed":false}"#))
                .is_err()
        );
        assert!(
            read_message(&forge(
                r#"{"tag":1131,"seq":0,"sealed":false,"deadline":1}"#
            ))
            .is_err()
        );
        assert!(
            read_message(&forge(
                r#"{"tag":1108,"seq":1,"sealed":true,"deadline":1}"#
            ))
            .is_err()
        );
    }

    #[test]
    fn uri_and_replay_window() {
        let sym = h32(SYM);
        let id = "09178356979748733185";
        let uri = super::uri::build_uri(id, &sym, 1759313100);
        assert_eq!(
            uri,
            format!(
                "wc:{id}@2?relay-protocol=irn&symKey={SYM}&expiryTimestamp=1759313100"
            )
        );
        let p = super::uri::parse_uri(&uri).unwrap();
        assert_eq!((p.connection_id.as_str(), p.legacy), (id, false));
        assert_eq!((*p.sym, p.expiry_timestamp), (sym, 1759313100));
        let v2 = uri.replace(id, &"Ab".repeat(32));
        assert!(super::uri::parse_uri(&v2).unwrap().legacy);
        for bad in
            ["39178356979748733185", "0", "0123456789012345678901", "0+1"]
        {
            assert!(super::uri::parse_uri(&uri.replace(id, bad)).is_err());
        }
        let mut w = super::replay::ReplayWindow::default();
        assert_eq!([2, 1, 3].map(|s| w.accept(s)), [true, true, true]);
        assert!(!w.accept(2));
        assert!(!w.accept(0));
    }
}
