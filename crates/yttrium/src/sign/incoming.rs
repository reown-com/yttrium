use {
    crate::sign::{
        client_errors::RequestError,
        client_types::Session,
        envelope_type0,
        protocol_types::{
            Metadata, SessionDeleteJsonRpc, SessionProposalJsonRpcResponse,
            SessionRequestJsonRpc, SessionRequestJsonRpcErrorResponse,
            SessionRequestJsonRpcResponse, SessionRequestJsonRpcResultResponse,
            SessionSettle,
        },
        relay::IncomingSessionMessage,
        storage::{Storage, StoragePairing},
        utils::{diffie_hellman, topic_from_sym_key},
    },
    chacha20poly1305::{aead::Aead, ChaCha20Poly1305, KeyInit, Nonce},
    data_encoding::BASE64,
    relay_rpc::{
        domain::Topic,
        rpc::{BatchSubscribe, Params, Response, Subscription},
    },
    serde_json::Value,
    std::{collections::HashMap, sync::Arc},
};

#[derive(Debug, thiserror::Error)]
pub enum HandleError {
    // Logic or runtime errors e.g. storage
    #[error("Internal: {0}")]
    Internal(String),

    // Message ignored because of not being recognized and a JSON-RPC error response cannot be sent back
    #[error("Dropped: {0}")]
    Dropped(String),

    // Client errors that, in theory, we could send a JSON-RPC error response back to the sender
    #[error("Client: {0}")]
    Client(String),
}

pub fn handle(
    session_store: Arc<dyn Storage>,
    sub_msg: Subscription,
    session_request_tx: tokio::sync::mpsc::UnboundedSender<(
        Topic,
        IncomingSessionMessage,
    )>,
    priority_request_tx: tokio::sync::mpsc::UnboundedSender<(
        Params,
        tokio::sync::oneshot::Sender<Result<Response, RequestError>>,
    )>,
) -> Result<(), HandleError> {
    let key = session_store
        .get_decryption_key_for_topic(sub_msg.data.topic.clone())
        .map_err(|e| {
            HandleError::Internal(format!("get decryption key for topic: {e}"))
        })?;
    let session_sym_key = if let Some(session_sym_key) = key {
        session_sym_key
    } else {
        return Err(HandleError::Dropped(
            "No decryption key found to decrypt message, ACKing and ignoring"
                .to_string(),
        ));
    };

    let decoded = BASE64
        .decode(sub_msg.data.message.as_bytes())
        .map_err(|e| HandleError::Client(format!("decode message: {e}")))?;

    let envelope = envelope_type0::deserialize_envelope_type0(&decoded)
        .map_err(|e| {
            HandleError::Client(format!("deserialize envelope: {e}"))
        })?;
    let key = ChaCha20Poly1305::new(&session_sym_key.into());
    let decrypted = key
        .decrypt(&Nonce::from(envelope.iv), envelope.sb.as_slice())
        .map_err(|e| HandleError::Client(format!("decrypt message: {e}")))?;
    let value = serde_json::from_slice::<serde_json::Value>(&decrypted)
        .map_err(|e| HandleError::Client(format!("parse message: {e}")))?;

    if let Some(method) = value.get("method") {
        if method.as_str() == Some("wc_sessionSettle") {
            let request = serde_json::from_value::<SessionSettle>(
                value
                    .get("params")
                    .ok_or(HandleError::Client("params not found".to_string()))?
                    .clone(),
            )
            .map_err(|e| {
                HandleError::Client(format!("parse settle message: {e}"))
            })?;

            let controller_key = Some(
                hex::decode(request.controller.public_key.clone())
                    .map_err(|e| HandleError::Client(format!("decode controller public key: {e}")))?
                    .try_into()
                    .map_err(|e| HandleError::Client(format!("convert controller public key to fixed-size array: {e:?}")))?,
            );

            let session = Session {
                request_id: 0,
                topic: sub_msg.data.topic.clone(),
                expiry: request.expiry,
                relay_protocol: "irn".to_string(),
                relay_data: None,
                self_public_key: session_sym_key, // TODO this is wrong
                controller_key,
                session_sym_key,
                self_meta_data: Metadata {
                    name: "".to_string(),
                    description: "".to_string(),
                    url: "".to_string(),
                    icons: vec![],
                    verify_url: None,
                    redirect: None,
                },
                peer_public_key: Some(
                    hex::decode(request.controller.public_key.clone())
                        .unwrap()
                        .try_into()
                        .unwrap(),
                ),
                peer_meta_data: Some(request.controller.metadata.clone()),
                session_namespaces: request.namespaces.clone(),
                required_namespaces: HashMap::new(),
                optional_namespaces: None,
                session_properties: request.session_properties.clone(),
                scoped_properties: request.scoped_properties.clone(),
                is_acknowledged: false,
                pairing_topic: "".to_string().into(),
                transport_type: None,
            };

            session_store.add_session(session).map_err(|e| {
                HandleError::Internal(format!("add session: {e}"))
            })?;

            session_request_tx
                .send((
                    sub_msg.data.topic.clone(),
                    IncomingSessionMessage::SessionConnect(
                        0,
                        sub_msg.data.topic.clone(),
                    ),
                ))
                .map_err(|e| {
                    HandleError::Internal(format!("send session connect: {e}"))
                })?;
            Ok(())
        } else if method.as_str() == Some("wc_sessionRequest") {
            // TODO implement relay-side request queue
            let request =
                serde_json::from_value::<SessionRequestJsonRpc>(value)
                    .map_err(|e| {
                        HandleError::Client(format!(
                            "Failed to parse decrypted message: {e}"
                        ))
                    })?;

            session_request_tx
                .send((
                    sub_msg.data.topic,
                    IncomingSessionMessage::SessionRequest(request),
                ))
                .map_err(|e| {
                    HandleError::Internal(format!("send session request: {e}"))
                })?;
            Ok(())
        } else if method.as_str() == Some("wc_sessionUpdate") {
            // Parse update payload and update storage
            let update = serde_json::from_value::<
                crate::sign::protocol_types::SessionUpdateJsonRpc,
            >(value)
            .map_err(|e| HandleError::Internal(format!("parse update: {e}")))?;

            // Update local session namespaces
            if let Some(mut session) =
                session_store.get_session(sub_msg.data.topic.clone()).map_err(
                    |e| HandleError::Internal(format!("get session: {e}")),
                )?
            {
                session.session_namespaces = update.params.namespaces.clone();
                session_store.add_session(session).map_err(|e| {
                    HandleError::Internal(format!("add session: {e}"))
                })?;
            } else {
                tracing::warn!(
                    "wc_sessionUpdate received for unknown topic: {:?}",
                    sub_msg.data.topic
                );
            }
            session_request_tx
                .send((
                    sub_msg.data.topic.clone(),
                    IncomingSessionMessage::SessionUpdate(
                        update.id,
                        sub_msg.data.topic,
                        update.params.namespaces,
                    ),
                ))
                .map_err(|e| {
                    HandleError::Internal(format!("send session update: {e}"))
                })?;
            Ok(())
        } else if method.as_str() == Some("wc_sessionExtend") {
            // Parse extend payload
            let extend = serde_json::from_value::<
                crate::sign::protocol_types::SessionExtendJsonRpc,
            >(value)
            .map_err(|e| HandleError::Internal(format!("parse extend: {e}")))?;

            // Update session expiry if session exists, peer is controller, and expiry is valid (> current and <= now+7d)
            if let Some(mut session) =
                session_store.get_session(sub_msg.data.topic.clone()).map_err(
                    |e| HandleError::Internal(format!("get session: {e}")),
                )?
            {
                let now = crate::time::SystemTime::now()
                    .duration_since(crate::time::UNIX_EPOCH)
                    .map_err(|e| {
                        HandleError::Internal(format!("get now: {e}"))
                    })?
                    .as_secs();
                match crate::sign::utils::validate_extend_request(
                    &session,
                    extend.params.expiry,
                    now,
                ) {
                    Ok(accepted_expiry) => {
                        session.expiry = accepted_expiry;
                        session_store.add_session(session).map_err(|e| {
                            HandleError::Internal(format!("add session: {e}"))
                        })?;
                        // Emit extend event
                        session_request_tx
                            .send((
                                sub_msg.data.topic.clone(),
                                IncomingSessionMessage::SessionExtend(
                                    extend.id,
                                    sub_msg.data.topic,
                                ),
                            ))
                            .map_err(|e| {
                                HandleError::Internal(format!(
                                    "send session extend: {e}"
                                ))
                            })?;
                    }
                    Err(e) => {
                        tracing::warn!("ignored wc_sessionExtend: {:?}", e);
                    }
                }
            } else {
                tracing::warn!(
                    "wc_sessionExtend received for unknown topic: {:?}",
                    sub_msg.data.topic
                );
            }
            Ok(())
        } else if method.as_str() == Some("wc_sessionEvent") {
            // TODO dedup events based on JSON RPC history
            // Parse wc_sessionEvent params
            let params =
                serde_json::from_value::<
                    crate::sign::protocol_types::EventParams,
                >(value.get("params").cloned().ok_or_else(
                    || HandleError::Client("params not found".to_string()),
                )?)
                .map_err(|e| {
                    HandleError::Client(format!("parse event params: {e}"))
                })?;

            let name = params.event.name;
            let data_str =
                serde_json::to_string(&params.event.data).map_err(|e| {
                    HandleError::Client(format!("serialize event data: {e}"))
                })?;
            let chain_id = params.chain_id;

            session_request_tx
                .send((
                    sub_msg.data.topic.clone(),
                    IncomingSessionMessage::SessionEvent(
                        sub_msg.data.topic,
                        name,
                        data_str,
                        chain_id,
                    ),
                ))
                .map_err(|e| {
                    HandleError::Internal(format!("send session event: {e}"))
                })?;
            Ok(())
        } else if method.as_str() == Some("wc_sessionPing") {
            // no-op for pings
            Ok(())
        } else if method.as_str() == Some("wc_sessionDelete") {
            let delete = serde_json::from_value::<SessionDeleteJsonRpc>(value)
                .map_err(|e| {
                    HandleError::Client(format!("parse delete message: {e}"))
                })?;

            session_store.delete_session(sub_msg.data.topic.clone()).map_err(
                |e| HandleError::Internal(format!("delete session: {e}")),
            )?;
            session_request_tx
                .send((
                    sub_msg.data.topic.clone(),
                    IncomingSessionMessage::Disconnect(
                        delete.id,
                        sub_msg.data.topic,
                    ),
                ))
                .map_err(|e| {
                    HandleError::Internal(format!("send disconnect: {e}"))
                })?;
            Ok(())
        } else {
            Err(HandleError::Client(format!("unexpected method: {method}")))
        }
    } else {
        let rpc_id = value.get("id");
        if let Some(rpc_id) = rpc_id {
            let rpc_id = match rpc_id {
                Value::Number(n) => n.as_u64(),
                Value::String(s) => s.parse::<u64>().ok(),
                _ => None,
            };
            if let Some(rpc_id) = rpc_id {
                let pairing = session_store
                    .get_pairing(sub_msg.data.topic.clone(), rpc_id)
                    .map_err(|e| {
                        HandleError::Internal(format!("get pairing: {e}"))
                    })?;
                if let Some(StoragePairing { sym_key: _, self_key }) = pairing {
                    let response = serde_json::from_value::<
                        SessionProposalJsonRpcResponse,
                    >(value)
                    .map_err(|e| {
                        HandleError::Client(format!(
                            "parse proposal response: {e}"
                        ))
                    })?;
                    let response = match response {
                        SessionProposalJsonRpcResponse::Result(result) => {
                            result
                        }
                        SessionProposalJsonRpcResponse::Error(error) => {
                            tracing::error!("Proposal error: {:?}", error);

                            // Emit SessionRejected event
                            if let Err(e) = session_request_tx.send((
                                sub_msg.data.topic.clone(),
                                IncomingSessionMessage::SessionReject(
                                    rpc_id,
                                    sub_msg.data.topic,
                                ),
                            )) {
                                tracing::debug!(
                                    "Failed to emit session rejected event: {e}"
                                );
                            }

                            return Ok(());
                        }
                    };

                    let self_key = x25519_dalek::StaticSecret::from(self_key);
                    let responder_public_key =
                        hex::decode(response.result.responder_public_key)
                            .map_err(|e| {
                                HandleError::Client(format!(
                                    "decode responder public key: {e}"
                                ))
                            })?;
                    let responder_public_key: [u8; 32] =
                        responder_public_key.try_into().map_err(|e| {
                            HandleError::Client(format!(
                                "convert responder public key to fixed-size array: {e:?}"
                            ))
                        })?;
                    let responder_public_key =
                        x25519_dalek::PublicKey::from(responder_public_key);
                    let shared_secret =
                        diffie_hellman(&responder_public_key, &self_key);
                    let session_topic = topic_from_sym_key(&shared_secret);
                    session_store
                        .save_partial_session(
                            session_topic.clone(),
                            shared_secret,
                        )
                        .map_err(|e| {
                            HandleError::Internal(format!(
                                "save partial session: {e}"
                            ))
                        })?;

                    let (tx, rx) = tokio::sync::oneshot::channel();
                    crate::spawn::spawn(async move {
                        let response = rx.await;
                        tracing::debug!(
                            "Received batch subscribe response: {:?}",
                            response
                        );
                    });
                    if let Err(e) = priority_request_tx.send((
                        Params::BatchSubscribe(BatchSubscribe {
                            topics: vec![session_topic],
                        }),
                        tx,
                    )) {
                        tracing::warn!("Failed to send priority request: {e}");
                    }
                    Ok(())
                } else if sub_msg.data.tag == 1109 {
                    let response = if value.get("error").is_some() {
                        // Parse as error response
                        let error_response = serde_json::from_value::<
                            SessionRequestJsonRpcErrorResponse,
                        >(value)
                        .map_err(|e| {
                            HandleError::Client(format!(
                                "parse session request response: {e}"
                            ))
                        })?;
                        SessionRequestJsonRpcResponse::Error(error_response)
                    } else {
                        // Parse as result response
                        let result_response = serde_json::from_value::<
                            SessionRequestJsonRpcResultResponse,
                        >(value)
                        .map_err(|e| {
                            HandleError::Client(format!(
                                "parse session request response: {e}"
                            ))
                        })?;
                        SessionRequestJsonRpcResponse::Result(result_response)
                    };
                    if let Err(e) = session_request_tx.send((
                        sub_msg.data.topic.clone(),
                        IncomingSessionMessage::SessionRequestResponse(
                            rpc_id,
                            sub_msg.data.topic,
                            response,
                        ),
                    )) {
                        tracing::warn!(
                            "Failed to emit session request response event: {e}"
                        );
                    }
                    Ok(())
                } else {
                    Err(HandleError::Client(format!(
                        "ignoring message with invalid ID: {value:?}",
                    )))
                }
            } else {
                Err(HandleError::Dropped(format!(
                    "ignoring message with invalid ID: {value:?}",
                )))
            }
        } else {
            Err(HandleError::Dropped(format!(
                "ignoring message without method or ID: {value:?}",
            )))
        }
    }
}
