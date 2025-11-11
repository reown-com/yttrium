use {
    crate::sign::{
        client::MaybeVerifiedRequest,
        client_errors::RequestError,
        client_types::{Session, TransportType},
        envelope_type0,
        protocol_types::{
            methods, GenericJsonRpcMessage, GenericJsonRpcResponse, Metadata,
            SessionDeleteJsonRpc, SessionProposalJsonRpcResponse,
            SessionRequestJsonRpc, SessionRequestJsonRpcErrorResponse,
            SessionRequestJsonRpcResponse, SessionRequestJsonRpcResultResponse,
            SessionSettle,
        },
        relay::IncomingSessionMessage,
        storage::{Storage, StoragePairing},
        utils::{
            diffie_hellman, topic_from_sym_key, DecryptedHash, EncryptedHash,
        },
        verify::{handle_verify, VERIFY_SERVER_URL},
    },
    chacha20poly1305::{aead::Aead, ChaCha20Poly1305, KeyInit, Nonce},
    data_encoding::BASE64,
    relay_rpc::{
        domain::Topic,
        rpc::{Params, Response, Subscribe, Subscription},
    },
    sha2::Digest,
    std::{collections::HashMap, sync::Arc},
    tracing::Instrument,
};

#[derive(Debug, thiserror::Error)]
pub enum HandleError {
    // Unrecoverable logic or runtime errors e.g. storage but where retry is allowed
    #[error("Internal: {0}")]
    Temporary(String),

    // Unrecoverable errors resulting in the message being ignored because of not being recognized and a JSON-RPC error response cannot be sent back
    #[error("Dropped: {0}")]
    Dropped(String),

    // Unrecoverable client errors that, in theory, we could send a JSON-RPC error response back to the sender
    #[error("Client: {0}")]
    Peer(String),

    #[error("Already handled")]
    AlreadyHandled,
}

pub async fn handle(
    storage: Arc<dyn Storage>,
    http_client: reqwest::Client,
    sub_msg: Subscription,
    session_request_tx: tokio::sync::mpsc::UnboundedSender<(
        Topic,
        IncomingSessionMessage,
    )>,
    priority_request_tx: tokio::sync::mpsc::UnboundedSender<(
        MaybeVerifiedRequest,
        tokio::sync::oneshot::Sender<Result<Response, RequestError>>,
    )>,
    probe_group: Option<String>,
) -> Result<(), HandleError> {
    // WARNING: This function must complete in <4s not including network latency, so don't do blocking operations such as network requests

    let key = storage
        .get_decryption_key_for_topic(sub_msg.data.topic.clone())
        .map_err(|e| {
            HandleError::Temporary(format!("get decryption key for topic: {e}"))
        })?;
    let session_sym_key = if let Some(session_sym_key) = key {
        session_sym_key
    } else {
        return Err(HandleError::Dropped(
            "No decryption key found to decrypt message, ACKing and ignoring"
                .to_string(),
        ));
    };

    let encrypted_hash = EncryptedHash(hex::encode(sha2::Sha256::digest(
        sub_msg.data.message.as_bytes(),
    )));
    let decoded = BASE64
        .decode(sub_msg.data.message.as_bytes())
        .map_err(|e| HandleError::Peer(format!("decode message: {e}")))?;

    let envelope = envelope_type0::deserialize_envelope_type0(&decoded)
        .map_err(|e| HandleError::Peer(format!("deserialize envelope: {e}")))?;
    let key = ChaCha20Poly1305::new(&session_sym_key.into());
    let decrypted = key
        .decrypt(&Nonce::from(envelope.iv), envelope.sb.as_slice())
        .map_err(|e| HandleError::Peer(format!("decrypt message: {e}")))?;
    tracing::debug!(
        "decrypted message: {}",
        String::from_utf8_lossy(&decrypted)
    );
    let value = serde_json::from_slice::<serde_json::Value>(&decrypted)
        .map_err(|e| HandleError::Peer(format!("parse JSON: {e}")))?;
    let message = serde_json::from_slice::<GenericJsonRpcMessage>(&decrypted)
        .map_err(|e| {
        HandleError::Peer(format!("parse message: {e}: {value}"))
    })?;

    match message {
        GenericJsonRpcMessage::Request(request) => {
            let request_id = request.id.into_value();
            let method = request.method;

            let exists =
                storage.does_json_rpc_exist(request_id).map_err(|e| {
                    HandleError::Temporary(format!("history exists check: {e}"))
                })?;
            if exists {
                return Err(HandleError::AlreadyHandled);
            }

            match method.as_str() {
                methods::SESSION_SETTLE => {
                    let request = serde_json::from_value::<SessionSettle>(
                        request.params.clone(),
                    )
                    .map_err(|e| {
                        HandleError::Peer(format!("parse settle message: {e}"))
                    })?;

                    let controller_key = Some(
                        hex::decode(request.controller.public_key.clone())
                            .map_err(|e| HandleError::Peer(format!("decode controller public key: {e}")))?
                            .try_into()
                            .map_err(|e| HandleError::Peer(format!("convert controller public key to fixed-size array: {e:?}")))?,
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
                        peer_meta_data: Some(
                            request.controller.metadata.clone(),
                        ),
                        session_namespaces: request.namespaces.clone(),
                        required_namespaces: HashMap::new(),
                        optional_namespaces: HashMap::new(),
                        session_properties: request.session_properties.clone(),
                        scoped_properties: request.scoped_properties.clone(),
                        is_acknowledged: false,
                        pairing_topic: "".to_string().into(),
                        transport_type: None,
                    };

                    storage.add_session(session).map_err(|e| {
                        HandleError::Temporary(format!("add session: {e}"))
                    })?;

                    // FIXME program could stop here and we wouldn't insert the JSON-RPC history. No worse than having it above (insert history but not sessions), but still an issue
                    // TODO combine storage operation with add_session, atomically
                    // TODO later, remove separate JSON-RPC history and just use sessions table for idempotency
                    storage
                        .insert_json_rpc_history(
                            request_id,
                            sub_msg.data.topic.to_string(),
                            method,
                            value.to_string(),
                            Some(TransportType::Relay),
                        )
                        .map_err(|e| {
                            HandleError::Temporary(format!(
                                "insert history: {e}"
                            ))
                        })?;

                    // At-most-once delivery guarantee, app can call getConnections()
                    if let Err(e) = session_request_tx
                        .send((
                            sub_msg.data.topic.clone(),
                            IncomingSessionMessage::SessionConnect(
                                0,
                                sub_msg.data.topic.clone(),
                            ),
                        ))
                        .map_err(|e| {
                            HandleError::Temporary(format!(
                                "send session connect: {e}"
                            ))
                        })
                    {
                        // Don't need to trigger a delivery retry. App can call getConnections()
                        tracing::debug!(
                            "Failed to emit session connect event: {e}"
                        );
                    }

                    Ok(())
                }
                methods::SESSION_REQUEST => {
                    // TODO implement relay-side request queue
                    let request =
                        serde_json::from_value::<SessionRequestJsonRpc>(
                            value.clone(),
                        )
                        .map_err(|e| {
                            HandleError::Peer(format!(
                                "Failed to parse decrypted message: {e}"
                            ))
                        })?;

                    let session = storage
                        .get_session(sub_msg.data.topic.clone())
                        .map_err(|e| {
                            HandleError::Temporary(format!("get session: {e}"))
                        })?
                        .ok_or(HandleError::Peer(
                            "should never happen:session not found".to_string(),
                        ))?;

                    // Warning: this is a network call!!!!
                    let decrypted_hash = DecryptedHash(hex::encode(
                        sha2::Sha256::digest(&decrypted),
                    ));
                    let attestation = handle_verify(
                        VERIFY_SERVER_URL.to_string(),
                        decrypted_hash,
                        http_client,
                        storage.clone(),
                        sub_msg.data.attestation.clone(),
                        encrypted_hash,
                        session.peer_meta_data.as_ref().unwrap().url.clone(),
                        probe_group.clone(),
                    )
                    .await;

                    storage
                        .insert_json_rpc_history(
                            request_id,
                            sub_msg.data.topic.to_string(),
                            method,
                            value.to_string(),
                            Some(TransportType::Relay),
                        )
                        .map_err(|e| {
                            HandleError::Temporary(format!(
                                "insert history: {e}"
                            ))
                        })?;

                    // TODO implement request queue
                    // No delivery guarantee, session request queue exists
                    if let Err(e) = session_request_tx
                        .send((
                            sub_msg.data.topic.clone(),
                            IncomingSessionMessage::SessionRequest(
                                request,
                                attestation,
                            ),
                        ))
                        .map_err(|e| {
                            HandleError::Temporary(format!(
                                "send session request: {e}"
                            ))
                        })
                    {
                        // Don't need to trigger a delivery retry. Session request queue exists
                        tracing::debug!(
                            "Failed to emit session request event: {e}"
                        );
                    }

                    Ok(())
                }
                methods::SESSION_UPDATE => {
                    // Parse update payload and update storage
                    let update = serde_json::from_value::<
                        crate::sign::protocol_types::SessionUpdateJsonRpc,
                    >(value.clone())
                    .map_err(|e| {
                        HandleError::Temporary(format!("parse update: {e}"))
                    })?;

                    // Update local session namespaces
                    if let Some(mut session) = storage
                        .get_session(sub_msg.data.topic.clone())
                        .map_err(|e| {
                            HandleError::Temporary(format!("get session: {e}"))
                        })?
                    {
                        session.session_namespaces =
                            update.params.namespaces.clone();
                        storage.add_session(session).map_err(|e| {
                            HandleError::Temporary(format!("add session: {e}"))
                        })?;
                    } else {
                        tracing::warn!(
                            "wc_sessionUpdate received for unknown topic: {:?}",
                            sub_msg.data.topic
                        );
                    }

                    storage
                        .insert_json_rpc_history(
                            request_id,
                            sub_msg.data.topic.to_string(),
                            method,
                            value.to_string(),
                            Some(TransportType::Relay),
                        )
                        .map_err(|e| {
                            HandleError::Temporary(format!(
                                "insert history: {e}"
                            ))
                        })?;

                    // At-most-once delivery guarantee, app can call getConnections()
                    if let Err(e) = session_request_tx.send((
                        sub_msg.data.topic.clone(),
                        IncomingSessionMessage::SessionUpdate(
                            update.id,
                            sub_msg.data.topic,
                            update.params.namespaces,
                        ),
                    )) {
                        // Don't need to trigger a delivery retry. App can call getConnections()
                        tracing::debug!(
                            "Failed to emit session update event: {e}"
                        );
                    }

                    Ok(())
                }
                methods::SESSION_EXTEND => {
                    // Parse extend payload
                    let extend = serde_json::from_value::<
                        crate::sign::protocol_types::SessionExtendJsonRpc,
                    >(value.clone())
                    .map_err(|e| {
                        HandleError::Peer(format!("parse extend: {e}"))
                    })?;

                    // Update session expiry if session exists, peer is controller, and expiry is valid (> current and <= now+7d)
                    if let Some(mut session) = storage
                        .get_session(sub_msg.data.topic.clone())
                        .map_err(|e| {
                            HandleError::Temporary(format!("get session: {e}"))
                        })?
                    {
                        let now = crate::time::SystemTime::now()
                            .duration_since(crate::time::UNIX_EPOCH)
                            .map_err(|e| {
                                HandleError::Temporary(format!("get now: {e}"))
                            })?
                            .as_secs();
                        match crate::sign::utils::validate_extend_request(
                            &session,
                            extend.params.expiry,
                            now,
                        ) {
                            Ok(accepted_expiry) => {
                                session.expiry = accepted_expiry;
                                storage.add_session(session).map_err(|e| {
                                    HandleError::Temporary(format!(
                                        "add session: {e}"
                                    ))
                                })?;

                                storage
                                    .insert_json_rpc_history(
                                        request_id,
                                        sub_msg.data.topic.to_string(),
                                        method,
                                        value.to_string(),
                                        Some(TransportType::Relay),
                                    )
                                    .map_err(|e| {
                                        HandleError::Temporary(format!(
                                            "insert history: {e}"
                                        ))
                                    })?;

                                // Emit extend event
                                // At-most-once delivery guarantee, app can call getConnections()
                                if let Err(e) = session_request_tx.send((
                                    sub_msg.data.topic.clone(),
                                    IncomingSessionMessage::SessionExtend(
                                        extend.id,
                                        sub_msg.data.topic.clone(),
                                    ),
                                )) {
                                    // Don't need to trigger a delivery retry. App can call getConnections()
                                    tracing::debug!(
                                        "Failed to emit session extend event: {e}"
                                    );
                                }
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "ignored wc_sessionExtend: {:?}",
                                    e
                                );

                                storage
                                    .insert_json_rpc_history(
                                        request_id,
                                        sub_msg.data.topic.to_string(),
                                        method,
                                        value.to_string(),
                                        Some(TransportType::Relay),
                                    )
                                    .map_err(|e| {
                                        HandleError::Temporary(format!(
                                            "insert history: {e}"
                                        ))
                                    })?;
                            }
                        }
                    } else {
                        tracing::warn!(
                            "wc_sessionExtend received for unknown topic: {:?}",
                            sub_msg.data.topic
                        );

                        storage
                            .insert_json_rpc_history(
                                request_id,
                                sub_msg.data.topic.to_string(),
                                method,
                                value.to_string(),
                                Some(TransportType::Relay),
                            )
                            .map_err(|e| {
                                HandleError::Temporary(format!(
                                    "insert history: {e}"
                                ))
                            })?;
                    }

                    Ok(())
                }
                methods::SESSION_EVENT => {
                    // Parse wc_sessionEvent params
                    let params = serde_json::from_value::<
                        crate::sign::protocol_types::EventParams,
                    >(request.params)
                    .map_err(|e| {
                        HandleError::Peer(format!("parse event params: {e}"))
                    })?;

                    let name = params.event.name;
                    let data_value = params.event.data;
                    let chain_id = params.chain_id;

                    storage
                        .insert_json_rpc_history(
                            request_id,
                            sub_msg.data.topic.to_string(),
                            method,
                            value.to_string(),
                            Some(TransportType::Relay),
                        )
                        .map_err(|e| {
                            HandleError::Temporary(format!(
                                "insert history: {e}"
                            ))
                        })?;

                    if let Err(e) = session_request_tx.send((
                        sub_msg.data.topic.clone(),
                        IncomingSessionMessage::SessionEvent(
                            sub_msg.data.topic.clone(),
                            name,
                            data_value,
                            chain_id,
                        ),
                    )) {
                        // At-most-once delivery guarantee, unfortunately. Session events can be lost.
                        tracing::debug!("Failed to emit session event: {e}");
                    }

                    Ok(())
                }
                methods::SESSION_PING => {
                    // no-op for pings
                    Ok(())
                }
                methods::SESSION_DELETE => {
                    let delete =
                        serde_json::from_value::<SessionDeleteJsonRpc>(
                            value.clone(),
                        )
                        .map_err(|e| {
                            HandleError::Peer(format!(
                                "parse delete message: {e}"
                            ))
                        })?;

                    storage
                        .delete_session(sub_msg.data.topic.clone())
                        .map_err(|e| {
                            HandleError::Temporary(format!(
                                "delete session: {e}"
                            ))
                        })?;

                    // IMO delete_session and emitting disconnect events should be idempotent. But this is unfortunately not currently the API spec
                    // If they were, we would not need to record the JSON-RPC history here and could always just do both operations: delete and emit. No conditionals
                    storage
                        .insert_json_rpc_history(
                            request_id,
                            sub_msg.data.topic.to_string(),
                            method,
                            value.to_string(),
                            Some(TransportType::Relay),
                        )
                        .map_err(|e| {
                            HandleError::Temporary(format!(
                                "insert history: {e}"
                            ))
                        })?;

                    // At-most-once delivery guarantee, app can call getConnections()
                    if let Err(e) = session_request_tx.send((
                        sub_msg.data.topic.clone(),
                        IncomingSessionMessage::Disconnect(
                            delete.id,
                            sub_msg.data.topic,
                        ),
                    )) {
                        // Don't need to trigger a delivery retry. App can call getConnections()
                        tracing::debug!("Failed to emit disconnect event: {e}");
                    }

                    Ok(())
                }
                _ => Err(HandleError::Peer(format!(
                    "unexpected method: {method}"
                ))),
            }
        }
        GenericJsonRpcMessage::Response(response) => {
            let rpc_id = match response {
                GenericJsonRpcResponse::Success(success) => {
                    success.id.into_value()
                }
                GenericJsonRpcResponse::Error(error) => error.id.into_value(),
            };

            let pairing = storage
                .get_pairing(sub_msg.data.topic.clone(), rpc_id)
                .map_err(|e| {
                    HandleError::Temporary(format!("get pairing: {e}"))
                })?;
            if let Some(StoragePairing { sym_key: _, self_key }) = pairing {
                let response = serde_json::from_value::<
                    SessionProposalJsonRpcResponse,
                >(value.clone())
                .map_err(|e| {
                    HandleError::Peer(format!("parse proposal response: {e}"))
                })?;
                let response = match response {
                    SessionProposalJsonRpcResponse::Result(result) => result,
                    SessionProposalJsonRpcResponse::Error(error) => {
                        tracing::error!("Proposal error: {:?}", error);

                        // Update JSON-RPC history with proposal response
                        storage
                            .update_json_rpc_history_response(
                                rpc_id,
                                value.to_string(),
                            )
                            .map_err(|e| {
                                HandleError::Temporary(format!(
                                    "update history response: {e}"
                                ))
                            })?;

                        // Emit SessionRejected event
                        // At-most-once delivery guarantee
                        if let Err(e) = session_request_tx.send((
                            sub_msg.data.topic.clone(),
                            IncomingSessionMessage::SessionReject(
                                rpc_id,
                                sub_msg.data.topic,
                            ),
                        )) {
                            // If app dies, it must establish a new connection anyway rather than resuming the pending one
                            tracing::debug!(
                                "Failed to emit session rejected event: {e}"
                            );
                        }

                        return Ok(());
                    }
                };

                let session_topic = {
                    let self_key = x25519_dalek::StaticSecret::from(self_key);
                    let responder_public_key =
                        hex::decode(response.result.responder_public_key)
                            .map_err(|e| {
                                HandleError::Peer(format!(
                                    "decode responder public key: {e}"
                                ))
                            })?;
                    let responder_public_key: [u8; 32] =
                        responder_public_key.try_into().map_err(|e| {
                            HandleError::Peer(format!(
                                "convert responder public key to fixed-size array: {e:?}"
                            ))
                        })?;
                    let responder_public_key =
                        x25519_dalek::PublicKey::from(responder_public_key);
                    let shared_secret =
                        diffie_hellman(&responder_public_key, &self_key);
                    let session_topic = topic_from_sym_key(&shared_secret);
                    storage
                        .save_partial_session(
                            session_topic.clone(),
                            shared_secret,
                        )
                        .map_err(|e| {
                            HandleError::Temporary(format!(
                                "save partial session: {e}"
                            ))
                        })?;
                    session_topic
                };

                // Update JSON-RPC history with proposal response
                storage
                    .update_json_rpc_history_response(rpc_id, value.to_string())
                    .map_err(|e| {
                        HandleError::Temporary(format!(
                            "update history response: {e}"
                        ))
                    })?;

                // No SessionConnect emit yet... that's emitted when the sessionSettle request comes in

                {
                    let params =
                        Params::Subscribe(Subscribe { topic: session_topic });
                    let (tx, rx) = tokio::sync::oneshot::channel();
                    crate::spawn::spawn(
                        async move {
                            // Consume the response to avoid a publish error
                            let response = rx.await;
                            tracing::debug!(
                                "Received subscribe response: {:?}",
                                response
                            );
                        }
                        .instrument(
                            tracing::debug_span!(
                                "subscribe_response",
                                group = probe_group.clone()
                            ),
                        ),
                    );
                    if let Err(e) = priority_request_tx
                        .send((MaybeVerifiedRequest::Unverified(params), tx))
                    {
                        tracing::warn!("Failed to send priority request: {e}");
                    }
                }

                Ok(())
            } else if sub_msg.data.tag == 1109 {
                let response = if value.get("error").is_some() {
                    // Parse as error response
                    let error_response = serde_json::from_value::<
                        SessionRequestJsonRpcErrorResponse,
                    >(value.clone())
                    .map_err(|e| {
                        HandleError::Peer(format!(
                            "parse session request response: {e}"
                        ))
                    })?;
                    SessionRequestJsonRpcResponse::Error(error_response)
                } else {
                    // Parse as result response
                    let result_response = serde_json::from_value::<
                        SessionRequestJsonRpcResultResponse,
                    >(value.clone())
                    .map_err(|e| {
                        HandleError::Peer(format!(
                            "parse session request response: {e}"
                        ))
                    })?;
                    SessionRequestJsonRpcResponse::Result(result_response)
                };

                // Update JSON-RPC history with session request response
                storage
                    .update_json_rpc_history_response(rpc_id, value.to_string())
                    .map_err(|e| {
                        HandleError::Temporary(format!(
                            "update history response: {e}"
                        ))
                    })?;

                // At-most-once delivery guarantee
                if let Err(e) = session_request_tx.send((
                    sub_msg.data.topic.clone(),
                    IncomingSessionMessage::SessionRequestResponse(
                        rpc_id,
                        sub_msg.data.topic,
                        response,
                    ),
                )) {
                    // If app dies, will have to send new session request
                    tracing::warn!(
                        "Failed to emit session request response event: {e}"
                    );
                }

                Ok(())
            } else {
                tracing::debug!("ignoring message: {value:?}");
                Ok(())
            }
        }
    }
}
