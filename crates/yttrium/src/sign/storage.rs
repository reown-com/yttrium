use {
    crate::sign::client_types::{Session, TransportType},
    relay_rpc::domain::Topic,
    serde::{Deserialize, Serialize},
};

// Implementation requirements:
// - Storage writes must be synchronously flushed
//
// Function design requirements:
// - Avoid functions that are too low-level and require multiple calls to achieve the caller's goal. For example, `get_decryption_key_for_topic()` can be used instead of `get_all_sessions()` and `get_pairing_keys()` and filtering caller-side
// - Avoid designing functions that exchange (returning or requiring) more information than necessary for the caller's needs. For example, `save_pairing_key()` is used over `update_session()`
pub trait Storage: Send + Sync {
    fn add_session(&self, session: Session) -> Result<(), StorageError>;
    fn delete_session(&self, topic: Topic) -> Result<(), StorageError>;
    fn get_session(
        &self,
        topic: Topic,
    ) -> Result<Option<Session>, StorageError>;
    fn get_all_sessions(&self) -> Result<Vec<Session>, StorageError>;
    fn get_all_topics(&self) -> Result<Vec<Topic>, StorageError>;
    fn get_decryption_key_for_topic(
        &self,
        topic: Topic,
    ) -> Result<Option<[u8; 32]>, StorageError>;
    fn save_pairing(
        &self,
        topic: Topic,
        rpc_id: u64,
        sym_key: [u8; 32],
        self_key: [u8; 32],
    ) -> Result<(), StorageError>;
    fn get_pairing(
        &self,
        topic: Topic,
        rpc_id: u64,
    ) -> Result<Option<StoragePairing>, StorageError>;
    fn save_partial_session(
        &self,
        topic: Topic,
        sym_key: [u8; 32],
    ) -> Result<(), StorageError>;

    // JSON-RPC History
    fn insert_json_rpc_history(
        &self,
        request_id: u64,
        topic: String,
        method: String,
        body: String,
        transport_type: Option<TransportType>,
    ) -> Result<(), StorageError>;

    fn update_json_rpc_history_response(
        &self,
        request_id: u64,
        response: String,
    ) -> Result<(), StorageError>;

    fn delete_json_rpc_history_by_topic(
        &self,
        topic: String,
    ) -> Result<(), StorageError>;

    fn does_json_rpc_exist(
        &self,
        request_id: u64,
    ) -> Result<bool, StorageError>;
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Runtime: {0}")]
    Runtime(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoragePairing {
    pub sym_key: [u8; 32],
    pub self_key: [u8; 32],
}
