/// This module handles 2 things:
/// 1. Detecting expired items and:
///    a. Cleaning them up
///    b. Emitting events if needed
/// 2. Sleeping for exactly the amount of time until the next expiry
use crate::sign::{
    storage::Storage, utils::is_expired, IncomingSessionMessage,
};
use {
    relay_rpc::domain::Topic,
    std::{collections::HashMap, sync::Arc},
    tokio::sync::Mutex,
};

pub fn start(
    storage: Arc<dyn Storage>,
    tx: tokio::sync::mpsc::UnboundedSender<(Topic, IncomingSessionMessage)>,
    mem_pairing_expiry_list: Arc<Mutex<HashMap<Topic, u64>>>,
    wake_rx: tokio::sync::broadcast::Receiver<()>,
    cleanup_rx: tokio_util::sync::CancellationToken,
) {
    // Run initial expiration check synchronously before spawning background task
    let next_expiry = check_and_cleanup_expired_items(
        storage.clone(),
        tx.clone(),
        // Technically since this is a new client, we don't have any pairings yet, so we can just use a new HashMap
        &mut HashMap::new(),
    );

    // Spawn expiration background task
    crate::spawn::spawn(expiration_task(
        next_expiry,
        storage,
        tx,
        mem_pairing_expiry_list,
        wake_rx,
        cleanup_rx,
    ));
}

/// Check for expired items and clean them up, emitting events as needed
/// Returns the next expiry time
fn check_and_cleanup_expired_items(
    storage: Arc<dyn Storage>,
    tx: tokio::sync::mpsc::UnboundedSender<(Topic, IncomingSessionMessage)>,
    mem_pairing_expiry_list: &mut HashMap<Topic, u64>,
) -> u64 {
    let mut next_expiry = u64::MAX;

    // Check expired sessions
    // Note: get_all_sessions() followed by delete_session() is not atomic, but storage
    // implementations handle thread-safety. Worst case: we miss a just-expired session
    // until next cycle, or delete a session that was just extended (harmless since
    // require_not_expired() protects usage).
    if let Ok(sessions) = storage.get_all_sessions() {
        for session in sessions {
            if session.is_expired() {
                let topic = session.topic.clone();
                let request_id = session.request_id;
                if let Err(e) = storage.delete_session(topic.clone()) {
                    tracing::warn!("Failed to delete expired session: {e}");
                } else {
                    let _ = tx.send((
                        topic.clone(),
                        IncomingSessionMessage::SessionExpired(
                            request_id, topic,
                        ),
                    ));
                }
            } else {
                next_expiry = next_expiry.min(session.expiry);
            }
        }
    }

    // Check expired pairings from storage (dapp-side)
    if let Ok(pairings) = storage.get_all_pairings() {
        for (topic, _rpc_id, expiry) in pairings {
            if is_expired(expiry) {
                if let Err(e) = storage.delete_pairing(topic.clone()) {
                    tracing::warn!("Failed to delete expired pairing: {e}");
                } else {
                    let _ = tx.send((
                        topic.clone(),
                        IncomingSessionMessage::SessionProposalExpired(topic),
                    ));
                }
            } else {
                next_expiry = next_expiry.min(expiry);
            }
        }
    }

    // Check expired pairings from memory (wallet-side)
    for (topic, expiry) in mem_pairing_expiry_list.clone().into_iter() {
        if is_expired(expiry) {
            mem_pairing_expiry_list.remove(&topic);
            let _ = tx.send((
                topic.clone(),
                IncomingSessionMessage::SessionProposalExpired(topic.clone()),
            ));
        } else {
            next_expiry = next_expiry.min(expiry);
        }
    }

    if let Ok(json_rpcs) = storage.get_all_json_rpc_with_timestamps() {
        for (request_id, _topic, insertion_timestamp) in json_rpcs {
            let expiry = insertion_timestamp + 30 * 24 * 60 * 60;
            if is_expired(expiry) {
                if let Err(e) =
                    storage.delete_json_rpc_history_by_id(request_id)
                {
                    tracing::warn!("Failed to delete expired JSON-RPC: {e}");
                }
            } else {
                next_expiry = next_expiry.min(expiry);
            }
        }
    }

    next_expiry
}

/// Background task that periodically checks for expired items
async fn expiration_task(
    initial_next_expiry: u64,
    storage: Arc<dyn Storage>,
    tx: tokio::sync::mpsc::UnboundedSender<(Topic, IncomingSessionMessage)>,
    mem_pairing_expiry_list: Arc<Mutex<HashMap<Topic, u64>>>,
    mut wake_rx: tokio::sync::broadcast::Receiver<()>,
    cleanup_rx: tokio_util::sync::CancellationToken,
) {
    let mut next_expiry = initial_next_expiry;
    loop {
        let sleep_duration = crate::time::Duration::from_secs(
            next_expiry.saturating_sub(
                crate::time::SystemTime::now()
                    .duration_since(crate::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            ),
        );

        tokio::select! {
            _ = crate::time::sleep(sleep_duration) => {
                let mut mem_pairing_expiry_list = mem_pairing_expiry_list.lock().await;
                next_expiry = check_and_cleanup_expired_items(
                    storage.clone(),
                    tx.clone(),
                    &mut mem_pairing_expiry_list,
                );
            }
            _ = wake_rx.recv() => {
                let mut mem_pairing_expiry_list = mem_pairing_expiry_list.lock().await;
                next_expiry = check_and_cleanup_expired_items(
                    storage.clone(),
                    tx.clone(),
                    &mut mem_pairing_expiry_list,
                );
            }
            _ = cleanup_rx.cancelled() => {
                break;
            }
        }
    }
}

// TODO add tests w/ a mock storage implementation
