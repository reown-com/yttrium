use {
    crate::{
        sign::client_types::{Session, Storage},
        uniffi_compat::sign::ffi_types::SessionFfi,
    },
    relay_rpc::domain::Topic,
    std::sync::Arc,
};

#[uniffi::export(with_foreign)]
pub trait SessionStoreFfi: Send + Sync {
    fn add_session(&self, session: SessionFfi);
    fn delete_session(&self, topic: String) -> Option<SessionFfi>;
    fn get_session(&self, topic: String) -> Option<SessionFfi>;
    fn get_all_sessions(&self) -> Vec<SessionFfi>;
    fn get_decryption_key_for_topic(&self, topic: String) -> Option<Vec<u8>>;
    fn save_pairing_key(&self, topic: String, sym_key: Vec<u8>);
}

pub struct SessionStoreFfiProxy(pub Arc<dyn SessionStoreFfi>);

impl Storage for SessionStoreFfiProxy {
    fn add_session(&self, session: Session) {
        self.0.add_session(session.into());
    }

    fn delete_session(&self, topic: Topic) -> Option<Session> {
        self.0.delete_session(topic.to_string()).map(Into::into)
    }

    fn get_session(&self, topic: Topic) -> Option<Session> {
        self.0.get_session(topic.to_string()).map(|s| s.into())
    }

    fn get_all_sessions(&self) -> Vec<Session> {
        self.0.get_all_sessions().into_iter().map(|s| s.into()).collect()
    }

    fn get_decryption_key_for_topic(&self, topic: Topic) -> Option<[u8; 32]> {
        self.0
            .get_decryption_key_for_topic(topic.to_string())
            .map(|s| s.try_into().unwrap())
    }

    fn save_pairing_key(&self, topic: Topic, sym_key: [u8; 32]) {
        self.0.save_pairing_key(topic.to_string(), sym_key.to_vec());
    }
}
