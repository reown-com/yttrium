//! Persisted state (guide §4). Records hold no secrets; private keys and sealing keys live under
//! `secret/`, which the platform maps to its secure store.

use {
    crate::{
        host::{Storage, StorageError},
        protocol::{
            encoding::{b64u, from_b64u},
            replay::ReplayWindow,
            types::{DappIntent, Metadata, Mode, Role, WireMessage},
        },
        rt::{BoxFut, boxed},
    },
    ed25519_dalek::SigningKey,
    serde::{Deserialize, Serialize, de::DeserializeOwned},
    serde_json::Value,
    std::sync::Arc,
    zeroize::Zeroizing,
};

pub struct Store {
    storage: Arc<dyn Storage>,
    prefix: String,
}

fn corrupt(key: &str) -> StorageError {
    StorageError::Failed { message: format!("corrupt value at {key}") }
}

impl Store {
    pub fn new(storage: Arc<dyn Storage>, prefix: String) -> Self {
        Self { storage, prefix }
    }

    fn k(&self, key: &str) -> String {
        format!("{}{key}", self.prefix)
    }

    fn get_raw(
        &self,
        key: &str,
    ) -> BoxFut<'_, Result<Option<String>, StorageError>> {
        let k = self.k(key);
        boxed(async move { self.storage.get(k).await })
    }

    fn put_raw(
        &self,
        key: &str,
        value: String,
    ) -> BoxFut<'_, Result<(), StorageError>> {
        let k = self.k(key);
        boxed(async move { self.storage.set(k, value).await })
    }

    pub async fn get<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, StorageError> {
        match self.get_raw(key).await? {
            None => Ok(None),
            Some(s) => crate::protocol::ijson::decode(s.as_bytes())
                .map(Some)
                .ok_or_else(|| corrupt(key)),
        }
    }

    pub async fn put<T: Serialize>(
        &self,
        key: &str,
        value: &T,
    ) -> Result<(), StorageError> {
        self.put_raw(key, serde_json::to_string(value).expect("serializable"))
            .await
    }

    pub fn remove(&self, key: &str) -> BoxFut<'_, Result<(), StorageError>> {
        let k = self.k(key);
        boxed(async move { self.storage.remove(k).await })
    }

    /// Keys under `sub`, without the store prefix.
    pub async fn keys(&self, sub: &str) -> Result<Vec<String>, StorageError> {
        let full = self.k(sub);
        Ok(self
            .storage
            .keys(full)
            .await?
            .into_iter()
            .filter_map(|k| k.strip_prefix(&self.prefix).map(str::to_string))
            .collect())
    }
}

pub fn key_to_b64(k: &SigningKey) -> String {
    b64u(&k.to_bytes())
}

pub fn key_from_b64(s: &str) -> Option<SigningKey> {
    let b = from_b64u(s).ok()?;
    Some(SigningKey::from_bytes(&b.try_into().ok()?))
}

pub fn arr32(s: &str) -> Option<[u8; 32]> {
    from_b64u(s).ok()?.try_into().ok()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnRecord {
    pub id: String,
    pub role: Role,
    pub version: u32,
    pub binding_pub: String,
    pub dapp_pub: String,
    pub wallet_pub: String,
    pub dapp_mode: Mode,
    pub wallet_mode: Mode,
    pub namespaces: Value,
    pub peer_metadata: Metadata,
    pub expiry: u64,
    pub next_seq: u64,
    pub recv: ReplayWindow,
    pub applied_update: u64,
    pub applied_extend: u64,
}

impl ConnRecord {
    pub fn own_mode(&self) -> Mode {
        if self.role == Role::Dapp { self.dapp_mode } else { self.wallet_mode }
    }
    pub fn peer_mode(&self) -> Mode {
        if self.role == Role::Dapp { self.wallet_mode } else { self.dapp_mode }
    }
    pub fn peer_pub(&self) -> [u8; 32] {
        arr32(if self.role == Role::Dapp {
            &self.wallet_pub
        } else {
            &self.dapp_pub
        })
        .expect("stored pub")
    }
    pub fn binding(&self) -> [u8; 32] {
        arr32(&self.binding_pub).expect("stored bindingPub")
    }
}

#[derive(Serialize, Deserialize)]
pub struct ConnSecrets {
    pub pairing: String,
    pub d2w: String,
    pub w2d: String,
}

pub struct Conn {
    pub rec: ConnRecord,
    pub key: SigningKey,
    pub d2w: Zeroizing<[u8; 32]>,
    pub w2d: Zeroizing<[u8; 32]>,
}

impl Conn {
    pub fn send_key(&self) -> &[u8; 32] {
        if self.rec.role == Role::Dapp { &self.d2w } else { &self.w2d }
    }
    pub fn recv_key(&self) -> &[u8; 32] {
        if self.rec.role == Role::Dapp { &self.w2d } else { &self.d2w }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DappReq {
    pub id: String,
    pub intent: DappIntent,
    pub versions: Vec<u32>,
    pub proposal_id: i64,
    pub expires_at: u64,
    pub failed: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ReqSecrets {
    pub sym: String,
    pub pairing: String,
}

pub struct Req {
    pub rec: DappReq,
    pub sym: Zeroizing<[u8; 32]>,
    pub key: SigningKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PendingAnswer {
    #[serde(rename_all = "camelCase")]
    Bind {
        body: Value,
        own: Mode,
        expiry: u64,
        namespaces: Value,
    },
    Reject {
        body: Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletProp {
    pub id: String,
    pub dapp_intent: DappIntent,
    pub dapp_pub: String,
    pub dapp_versions: Vec<u32>,
    pub version: u32,
    pub sealable: bool,
    pub proposal_id: i64,
    pub params: Value,
    pub metadata: Metadata,
    pub expires_at: u64,
    pub pending: Option<PendingAnswer>,
}

#[derive(Serialize, Deserialize)]
pub struct PropSecrets {
    pub sym: String,
    pub pairing: Option<String>,
}

pub struct Prop {
    pub rec: WalletProp,
    pub sym: Zeroizing<[u8; 32]>,
    pub key: Option<SigningKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Outbox {
    pub conn: String,
    pub seq: u64,
    pub message: WireMessage,
    pub ttl: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// An incoming request the app has not answered yet. Its method decides the response's tag,
/// as v2 reads it back from history.
pub struct InReq {
    pub conn: String,
    pub id: i64,
    pub method: String,
    pub deadline: u64,
    pub params: Value,
}

/// An outgoing request awaiting its response, so a response or expiry after a restart still
/// matches it by JSON-RPC id.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutReq {
    pub conn: String,
    pub id: i64,
    pub method: String,
    pub seq: u64,
    pub deadline: u64,
    pub params: Value,
}
