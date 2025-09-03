use {crate::sign::client_types::Session, relay_rpc::domain::Topic};

// Implementation requirements:
// - Storage writes must be synchronously flushed
//
// Function design requirements:
// - Avoid functions that are too low-level and require multiple calls to achieve the caller's goal. For example, `get_decryption_key_for_topic()` can be used instead of `get_all_sessions()` and `get_pairing_keys()` and filtering caller-side
// - Avoid designing functions that exchange (returning or requiring) more information than necessary for the caller's needs. For example, `save_pairing_key()` is used over `update_session()`
pub trait Storage: Send + Sync {
    fn add_session(&self, session: Session);
    fn delete_session(&self, topic: Topic);
    fn get_session(&self, topic: Topic) -> Option<Session>;
    fn get_all_sessions(&self) -> Vec<Session>;
    fn get_all_topics(&self) -> Vec<Topic>;
    fn get_decryption_key_for_topic(&self, topic: Topic) -> Option<[u8; 32]>;
    fn save_pairing(
        &self,
        topic: Topic,
        rpc_id: u64,
        sym_key: [u8; 32],
        self_key: [u8; 32],
    );
    fn get_pairing(
        &self,
        topic: Topic,
        rpc_id: u64,
    ) -> Option<([u8; 32], [u8; 32])>;
    fn save_partial_session(&self, topic: Topic, sym_key: [u8; 32]);
}
