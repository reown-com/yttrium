use {
    crate::{
        sign::{client_types::Session, SessionStore},
        uniffi_compat::sign::ffi_types::SessionFfi,
    },
    std::sync::Arc,
};

#[uniffi::export(with_foreign)]
pub trait SessionStoreFfi: Send + Sync {
    fn add_session(&self, session: SessionFfi);
    fn delete_session(&self, topic: String);
    fn get_session(&self, topic: String) -> Option<SessionFfi>;
    fn get_all_sessions(&self) -> Vec<SessionFfi>;
}

pub struct SessionStoreFfiProxy(pub Arc<dyn SessionStoreFfi>);

impl SessionStore for SessionStoreFfiProxy {
    fn add_session(&self, session: Session) {
        self.0.add_session(session.into());
    }

    fn delete_session(&self, topic: String) {
        self.0.delete_session(topic);
    }

    fn get_session(&self, topic: String) -> Option<Session> {
        self.0.get_session(topic).map(|s| s.into())
    }

    fn get_all_sessions(&self) -> Vec<Session> {
        self.0.get_all_sessions().into_iter().map(|s| s.into()).collect()
    }
}
