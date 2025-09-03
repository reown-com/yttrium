use {
    crate::{
        sign::{client_types::Session, storage::Storage},
        uniffi_compat::sign::ffi_types::SessionFfi,
    },
    relay_rpc::domain::Topic,
    std::sync::Arc,
};

#[uniffi::export(with_foreign)]
pub trait StorageFfi: Send + Sync {
    fn add_session(&self, session: SessionFfi);
    fn delete_session(&self, topic: String);
    fn get_session(&self, topic: String) -> Option<SessionFfi>;
    fn get_all_sessions(&self) -> Vec<SessionFfi>;
    fn get_all_topics(&self) -> Vec<Topic>;
    fn get_decryption_key_for_topic(&self, topic: String) -> Option<Vec<u8>>;
    fn save_pairing(
        &self,
        topic: String,
        rpc_id: u64,
        sym_key: Vec<u8>,
        self_key: Vec<u8>,
    );
    fn get_pairing(&self, topic: String, rpc_id: u64) -> Option<PairingFfi>;
    fn save_partial_session(&self, topic: String, sym_key: Vec<u8>);
}

pub struct StorageFfiProxy(pub Arc<dyn StorageFfi>);

impl Storage for StorageFfiProxy {
    fn add_session(&self, session: Session) {
        self.0.add_session(session.into());
    }

    fn delete_session(&self, topic: Topic) {
        self.0.delete_session(topic.to_string())
    }

    fn get_session(&self, topic: Topic) -> Option<Session> {
        self.0.get_session(topic.to_string()).map(|s| s.into())
    }

    fn get_all_sessions(&self) -> Vec<Session> {
        self.0.get_all_sessions().into_iter().map(|s| s.into()).collect()
    }

    fn get_all_topics(&self) -> Vec<Topic> {
        self.0.get_all_topics()
    }

    fn get_decryption_key_for_topic(&self, topic: Topic) -> Option<[u8; 32]> {
        self.0
            .get_decryption_key_for_topic(topic.to_string())
            .map(|s| s.try_into().unwrap())
    }

    fn save_pairing(
        &self,
        topic: Topic,
        rpc_id: u64,
        sym_key: [u8; 32],
        self_key: [u8; 32],
    ) {
        self.0.save_pairing(
            topic.to_string(),
            rpc_id,
            sym_key.to_vec(),
            self_key.to_vec(),
        );
    }

    fn get_pairing(
        &self,
        topic: Topic,
        rpc_id: u64,
    ) -> Option<([u8; 32], [u8; 32])> {
        self.0.get_pairing(topic.to_string(), rpc_id).map(|pairing| {
            (
                pairing.sym_key.try_into().unwrap(),
                pairing.self_key.try_into().unwrap(),
            )
        })
    }

    fn save_partial_session(&self, topic: Topic, sym_key: [u8; 32]) {
        self.0.save_partial_session(topic.to_string(), sym_key.to_vec());
    }
}

#[derive(uniffi::Record)]
pub struct PairingFfi {
    rpc_id: u64,
    sym_key: Vec<u8>,
    self_key: Vec<u8>,
}
