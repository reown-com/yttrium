use crate::sign::envelope_type0::{encode_envelope_type0, EnvelopeType0};
use crate::sign::protocol_types::{
    Controller, Metadata, Proposal, ProposalNamespaces, ProposalResponse,
    Relay, SessionSettle, SettleNamespace,
};
use crate::sign::relay_url::ConnectionOptions;
use alloy::rpc::json_rpc::{self, Id, ResponsePayload};
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{AeadCore, ChaCha20Poly1305, KeyInit, Nonce};
use data_encoding::BASE64;
use relay_rpc::auth::ed25519_dalek::{Signer, SigningKey};
use relay_rpc::domain::{DecodedClientId, Topic};
use relay_rpc::jwt::{JwtBasicClaims, JwtHeader};
use relay_rpc::rpc::{FetchMessages, FetchResponse};
use relay_rpc::{
    domain::{ClientId, MessageId, ProjectId},
    rpc::{ApproveSession, Payload, Request, Response},
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
#[cfg(target_arch = "wasm32")]
use web_sys::WebSocket;
use x25519_dalek::PublicKey;
#[cfg(not(target_arch = "wasm32"))]
use {
    futures::{SinkExt, StreamExt},
    tokio::net::TcpStream,
    tokio_tungstenite::{
        connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream,
    },
};

mod envelope_type0;
mod envelope_type1;
mod pairing_uri;
mod protocol_types;
mod relay_url;
#[cfg(test)]
mod tests;
mod utils;

#[derive(Debug, thiserror::Error)]
#[error("Sign request error: {0}")]
pub enum RequestError {
    #[error("Internal: {0}")]
    Internal(String),

    #[error("Should never happen: {0}")]
    ShouldNeverHappen(String),
}

#[derive(Debug, thiserror::Error)]
#[error("Sign pair error: {0}")]
pub enum PairError {
    #[error("Request error: {0}")]
    Request(RequestError),

    #[error("Internal: {0}")]
    Internal(String),

    #[error("Should never happen: {0}")]
    ShouldNeverHappen(String),
}

#[derive(Debug, thiserror::Error)]
#[error("Sign approve error: {0}")]
pub enum ApproveError {
    #[error("Request error: {0}")]
    Request(RequestError),

    #[error("Internal: {0}")]
    Internal(String),

    #[error("Should never happen: {0}")]
    ShouldNeverHappen(String),
}

struct WebSocketState {
    #[cfg(not(target_arch = "wasm32"))]
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    #[cfg(target_arch = "wasm32")]
    stream: (WebSocket, tokio::sync::mpsc::UnboundedReceiver<String>),
    message_id: u64,
}

pub struct Client {
    relay_url: String,
    #[allow(unused)]
    project_id: ProjectId,
    #[allow(unused)]
    client_id: ClientId,
    websocket: Option<WebSocketState>,
}

// TODO
// - separate app from wallet client
// - incoming message deduplication (RPC ID/hash)
// - random (?) request RPC ID generation
// - session expiry
// - WS reconnection & retries
// - respond to relay pings
//   - disconnect if no ping for 30s etc.
// - interpret relay disconnect reason
// - subscribe/fetch messages on startup - also solve that ordering problem?
// - handle connection error
// - handle project ID/JWT error

#[allow(unused)]
impl Client {
    pub fn new(
        relay_url: String,
        project_id: ProjectId,
        client_id: ClientId,
    ) -> Self {
        Self { relay_url, project_id, client_id, websocket: None }
    }

    pub async fn _connect(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L220
        unimplemented!()

        // TODO call `wc_proposeSession`
    }

    pub async fn pair(
        &mut self,
        uri: &str,
    ) -> Result<SessionProposal, PairError> {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L330

        // TODO parse URI
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/core/src/controllers/pairing.ts#L132
        let pairing_uri = pairing_uri::parse(uri)
            .map_err(|e| PairError::Internal(e.to_string()))?;

        // TODO immediately throw if expired - maybe not necessary if FetchMessages returns empty array?
        // note: no activatePairing
        // TODO save symkey, if necessary

        // TODO update relay method to not remove message

        let response = self
            .request::<FetchResponse>(relay_rpc::rpc::Params::FetchMessages(
                FetchMessages { topic: pairing_uri.topic.clone() },
            ))
            .await
            .map_err(|e| PairError::Internal(e.to_string()))?;

        for message in response.messages {
            if message.topic == pairing_uri.topic {
                let decoded =
                    BASE64.decode(message.message.as_bytes()).map_err(|e| {
                        PairError::Internal(format!(
                            "Failed to decode message: {e}"
                        ))
                    })?;
                let envelope =
                    envelope_type0::deserialize_envelope_type0(&decoded)
                        .map_err(|e| PairError::Internal(e.to_string()))?;
                let key = ChaCha20Poly1305::new(&pairing_uri.sym_key.into());
                let decrypted = key
                    .decrypt(&Nonce::from(envelope.iv), envelope.sb.as_slice())
                    .map_err(|e| PairError::Internal(e.to_string()))?;
                let request = serde_json::from_slice::<
                    json_rpc::Request<serde_json::Value>,
                >(&decrypted)
                .map_err(|e| {
                    PairError::Internal(format!(
                        "Failed to parse decrypted message: {e}"
                    ))
                })?;
                if request.meta.method != "wc_sessionPropose" {
                    return Err(PairError::Internal(format!(
                        "Expected wc_sessionPropose, got {}",
                        request.meta.method
                    )));
                }
                println!("rpc request: {}", request.meta.id);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&request.params).unwrap()
                );
                let proposal =
                    serde_json::from_value::<Proposal>(request.params)
                        .map_err(|e| PairError::Internal(e.to_string()))?;
                println!("{:?}", proposal);

                let proposer_public_key = hex::decode(proposal.proposer.public_key)
                    .map_err(|e| {
                        PairError::Internal(format!(
                            "Failed to decode proposer public key: {e}"
                        ))
                    })?
                    .try_into()
                    .map_err(|_| {
                        PairError::Internal(format!(
                            "Failed to convert proposer public key to fixed-size array"
                        ))
                    })?;
                println!("pairing topic: {}", proposal.pairing_topic);

                // TODO validate namespaces: https://specs.walletconnect.com/2.0/specs/clients/sign/namespaces#12-proposal-namespaces-must-not-have-chains-empty

                return Ok(SessionProposal {
                    session_proposal_rpc_id: request.meta.id,
                    pairing_topic: proposal.pairing_topic,
                    pairing_sym_key: pairing_uri.sym_key,
                    proposer_public_key,
                    requested_namespaces: proposal
                        .required_namespaces
                        .into_iter()
                        .chain(proposal.optional_namespaces.into_iter())
                        .collect(),
                });
            }
        }

        Err(PairError::Internal("No message found".to_string()))
    }

    pub async fn approve(
        &mut self,
        pairing: SessionProposal,
    ) -> Result<ApprovedSession, ApproveError> {
        // TODO params:
        // - approvedNamespaces, etc.

        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L341

        // TODO check is valid

        let mut namespaces = HashMap::new();
        for (namespace, namespace_proposal) in pairing.requested_namespaces {
            let accounts = namespace_proposal
                .chains
                .iter()
                .map(|chain| {
                    format!(
                        "{}:{}",
                        chain, "0x0000000000000000000000000000000000000000"
                    )
                })
                .collect();
            let namespace_settle = SettleNamespace {
                accounts,
                methods: namespace_proposal.methods,
                events: namespace_proposal.events,
            };
            namespaces.insert(namespace, namespace_settle);
        }
        println!("namespaces: {:?}", namespaces);

        let self_key = x25519_dalek::StaticSecret::random();
        let self_public_key = PublicKey::from(&self_key);
        pub fn diffie_hellman(
            public_key: &x25519_dalek::PublicKey,
            private_key: &x25519_dalek::StaticSecret,
        ) -> [u8; 32] {
            let shared_key = private_key.diffie_hellman(public_key);

            let derived_key =
                hkdf::Hkdf::<Sha256>::new(None, shared_key.as_bytes());

            let mut expanded_key = [0u8; 32];
            derived_key.expand(b"", &mut expanded_key).unwrap();
            expanded_key
        }
        let shared_secret =
            diffie_hellman(&pairing.proposer_public_key.into(), &self_key);
        let session_topic =
            hex::encode(sha2::Sha256::digest(shared_secret)).into();
        println!("session topic: {}", session_topic);

        let session_proposal_response = {
            let serialized =
                serde_json::to_string(&alloy::rpc::json_rpc::Response {
                    id: pairing.session_proposal_rpc_id,
                    payload: ResponsePayload::Success(ProposalResponse {
                        relay: Relay { protocol: "irn".to_string() },
                        responder_public_key: hex::encode(
                            self_public_key.to_bytes(),
                        ),
                    }) as ResponsePayload<_, ()>,
                })
                .map_err(|e| ApproveError::Internal(e.to_string()))?;

            let key = ChaCha20Poly1305::new(&pairing.pairing_sym_key.into());
            let nonce = ChaCha20Poly1305::generate_nonce()
                .map_err(|e| ApproveError::Internal(e.to_string()))?;
            let encrypted = key
                .encrypt(&nonce, serialized.as_bytes())
                .map_err(|e| ApproveError::Internal(e.to_string()))?;
            let encoded = encode_envelope_type0(&EnvelopeType0 {
                iv: nonce.into(),
                sb: encrypted,
            })
            .map_err(|e| ApproveError::Internal(e.to_string()))?;
            BASE64.encode(encoded.as_slice()).into()
        };

        let session_settlement_request = {
            let serialized =
                serde_json::to_string(&alloy::rpc::json_rpc::Request::new(
                    "wc_sessionSettle",
                    1000000010.into(),
                    SessionSettle {
                        relay: Relay { protocol: "irn".to_string() },
                        namespaces,
                        controller: Controller {
                            public_key: hex::encode(self_public_key.to_bytes()),
                            metadata: Metadata {
                                name: "Reown".to_string(),
                                description: "Reown".to_string(),
                                url: "https://reown.com".to_string(),
                                icons: vec![
                                    "https://reown.com/icon.png".to_string()
                                ],
                            },
                        },
                        expiry_timestamp: crate::time::SystemTime::now()
                            .duration_since(crate::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                            + 60 * 60 * 24 * 30,
                        session_properties: serde_json::Value::Null,
                        scoped_properties: serde_json::Value::Null,
                        session_config: serde_json::Value::Null,
                    },
                ))
                .map_err(|e| ApproveError::Internal(e.to_string()))?;

            let key = ChaCha20Poly1305::new(&shared_secret.into());
            let nonce = ChaCha20Poly1305::generate_nonce()
                .map_err(|e| ApproveError::Internal(e.to_string()))?;
            let encrypted = key
                .encrypt(&nonce, serialized.as_bytes())
                .map_err(|e| ApproveError::Internal(e.to_string()))?;
            let encoded = encode_envelope_type0(&EnvelopeType0 {
                iv: nonce.into(),
                sb: encrypted,
            })
            .map_err(|e| ApproveError::Internal(e.to_string()))?;
            BASE64.encode(encoded.as_slice()).into()
        };

        let approve_session = ApproveSession {
            pairing_topic: pairing.pairing_topic,
            session_topic,
            session_proposal_response,
            session_settlement_request,
            analytics: None,
        };

        // TODO insert session into storage

        match self
            .request::<bool>(relay_rpc::rpc::Params::ApproveSession(
                approve_session,
            ))
            .await
        {
            Ok(true) => {}
            Ok(false) => {
                // TODO remove from storage
                return Err(ApproveError::Internal(format!(
                    "Session rejected by relay"
                )));
            }
            Err(e) => {
                // TODO if error, remove from storage
                // https://github.com/reown-com/reown-kotlin/blob/1488873e0ac655bdc492ab12d8ea29b9985dd97c/protocol/sign/src/main/kotlin/com/reown/sign/engine/use_case/calls/ApproveSessionUseCase.kt#L115
                // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L476
                // consistency: ok if wallet thinks session is approved, but app never received approval
                // ideally one way we fix this via relay source of truth

                return Err(ApproveError::Request(e));
            }
        }

        Ok(ApprovedSession { session_sym_key: shared_secret })
    }

    pub async fn _reject(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L497
        unimplemented!()

        // TODO consider new relay method?
    }

    pub async fn _update(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L528
        unimplemented!()
    }

    pub async fn _extend(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L569
        unimplemented!()
    }

    pub async fn _request(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L601
        unimplemented!()
    }

    pub async fn _respond(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L701
        unimplemented!()
    }

    pub async fn _ping(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L727
        unimplemented!()
    }

    pub async fn _emit(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L764
        unimplemented!()
    }

    pub async fn _disconnect(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L781
        unimplemented!()
    }

    pub async fn _authenticate(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L817
        unimplemented!()
    }

    pub async fn _approve_session_authenticate(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L1123
        unimplemented!()
    }

    pub async fn _reject_session_authenticate(&self) {
        // TODO implement
        // https://github.com/WalletConnect/walletconnect-monorepo/blob/5bef698dcf0ae910548481959a6a5d87eaf7aaa5/packages/sign-client/src/controllers/engine.ts#L1298
        unimplemented!()
    }

    async fn request<T: DeserializeOwned>(
        &mut self,
        params: relay_rpc::rpc::Params,
    ) -> Result<T, RequestError> {
        let mut ws_state = if let Some(ws_state) = self.websocket.as_mut() {
            ws_state
        } else {
            let key = SigningKey::generate(&mut rand::thread_rng());

            let url = {
                let encoder = &data_encoding::BASE64URL_NOPAD;
                let claims = {
                    let data = JwtBasicClaims {
                        iss: DecodedClientId::from_key(&key.verifying_key())
                            .into(),
                        sub: "http://example.com".to_owned(),
                        aud: self.relay_url.clone(),
                        iat: crate::time::SystemTime::now()
                            .duration_since(crate::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64,
                        exp: Some(
                            crate::time::SystemTime::now()
                                .duration_since(crate::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs() as i64
                                + 60 * 60,
                        ),
                    };

                    encoder.encode(
                        serde_json::to_string(&data).unwrap().as_bytes(),
                    )
                };
                let header = encoder.encode(
                    serde_json::to_string(&JwtHeader::default())
                        .unwrap()
                        .as_bytes(),
                );
                let message = format!("{header}.{claims}");
                let signature = {
                    let data = key.sign(message.as_bytes());
                    encoder.encode(&data.to_bytes())
                };
                let auth = format!("{message}.{signature}");

                let conn_opts =
                    ConnectionOptions::new(self.project_id.clone(), auth)
                        .with_address(&self.relay_url);
                conn_opts.as_url().unwrap().to_string()
            };

            #[cfg(not(target_arch = "wasm32"))]
            let (ws_stream, _response) = connect_async(url)
                .await
                .map_err(|e| RequestError::Internal(e.to_string()))?;

            #[cfg(target_arch = "wasm32")]
            let ws_stream = {
                use wasm_bindgen::{prelude::Closure, JsCast};
                use web_sys::{Event, MessageEvent};

                let ws = web_sys::WebSocket::new(&url).map_err(|e| {
                    RequestError::Internal(format!(
                        "Failed to create WebSocket: {e:?}"
                    ))
                })?;
                let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
                let (tx2, mut rx2) = tokio::sync::mpsc::channel(1);
                let tx2 = std::sync::Arc::new(tokio::sync::Mutex::new(tx2));
                let tx2_clone = tx2.clone();
                let onopen_closure =
                    Closure::wrap(Box::new(move |_event: Event| {
                        let tx2 = tx2_clone.clone();
                        crate::spawn::spawn(async move {
                            let tx = tx2.lock().await;
                            tx.send(()).await.unwrap();
                        });
                    }) as Box<dyn Fn(Event)>);
                ws.set_onopen(Some(
                    Box::leak(Box::new(onopen_closure))
                        .as_ref()
                        .unchecked_ref(),
                ));
                let onmessage_closure =
                    Closure::wrap(Box::new(move |event: MessageEvent| {
                        // TODO may not be string messages
                        let message = event.data().as_string().unwrap();
                        tx.send(message).unwrap();
                    })
                        as Box<dyn Fn(MessageEvent)>);
                ws.set_onmessage(Some(
                    Box::leak(Box::new(onmessage_closure))
                        .as_ref()
                        .unchecked_ref(),
                ));
                web_sys::console::log_1(&"onopen".into());
                rx2.recv().await.unwrap(); // FIXME this blocks the async task
                web_sys::console::log_1(&"onopen received".into());
                (ws, rx)
            };

            const MIN: u64 = 1000000000; // MessageId::MIN is private
            self.websocket
                .insert(WebSocketState { stream: ws_stream, message_id: MIN })
        };

        let this_id = MessageId::new(ws_state.message_id);
        ws_state.message_id += 1;
        let request = Payload::Request(Request::new(this_id, params));
        let serialized = serde_json::to_string(&request).map_err(|e| {
            RequestError::ShouldNeverHappen(format!(
                "Failed to serialize request: {e}"
            ))
        })?;

        #[cfg(not(target_arch = "wasm32"))]
        ws_state.stream.send(Message::Text(serialized.into())).await.map_err(
            |e| RequestError::Internal(format!("Failed to send request: {e}")),
        )?;

        #[cfg(target_arch = "wasm32")]
        ws_state.stream.0.send_with_str(&serialized).map_err(|e| {
            RequestError::Internal(format!("Failed to send request: {e:?}"))
        })?;

        // TODO timeout
        #[cfg(not(target_arch = "wasm32"))]
        while let Some(n) = ws_state.stream.next().await {
            let n = n.map_err(|e| {
                RequestError::Internal(format!("WebSocket stream error: {e}"))
            })?;
            match n {
                Message::Text(text) => {
                    let response = serde_json::from_str::<Response>(&text)
                        .map_err(|e| {
                            RequestError::Internal(format!(
                                "Failed to parse response: {e}"
                            ))
                        })?;
                    if response.id() == this_id {
                        return match response {
                            Response::Success(response) => {
                                Ok(serde_json::from_value(response.result)
                                    .map_err(|e| {
                                        RequestError::Internal(format!(
                                            "Failed to parse response result: {e}"
                                        ))
                                    })?)
                            }
                            Response::Error(response) => Err(RequestError::Internal(
                                format!("RPC error: {:?}", response.error),
                            )),
                        };
                    }
                }
                _ => {}
            }
        }

        #[cfg(target_arch = "wasm32")]
        while let Some(message) = ws_state.stream.1.recv().await {
            let response =
                serde_json::from_str::<Response>(&message).map_err(|e| {
                    RequestError::Internal(format!(
                        "Failed to parse response: {e}"
                    ))
                })?;
            if response.id() == this_id {
                return match response {
                    Response::Success(response) => Ok(serde_json::from_value(
                        response.result,
                    )
                    .map_err(|e| {
                        RequestError::Internal(format!(
                            "Failed to parse response result: {e}"
                        ))
                    })?),
                    Response::Error(response) => Err(RequestError::Internal(
                        format!("RPC error: {:?}", response.error),
                    )),
                };
            }
        }

        Err(RequestError::Internal("No response".to_string()))
    }
}

pub struct SessionProposal {
    pub session_proposal_rpc_id: Id,
    pub pairing_topic: Topic,
    pub pairing_sym_key: [u8; 32],
    pub proposer_public_key: [u8; 32],
    pub requested_namespaces: ProposalNamespaces,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct ApprovedSession {
    pub session_sym_key: [u8; 32],
}
