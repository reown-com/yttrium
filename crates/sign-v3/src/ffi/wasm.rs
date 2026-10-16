//! The browser surface: `WasmSignClient.create(config, storage, onEvent)`, then `call(method, json)`.

use {
    crate::{
        Listener, SignClient, SignConfig, SignEvent, Storage, StorageError,
    },
    js_sys::{Function, Promise, Reflect},
    std::sync::Arc,
    wasm_bindgen::prelude::*,
    wasm_bindgen_futures::{JsFuture, future_to_promise},
};

struct JsStorage(JsValue);

fn storage_err(e: JsValue) -> StorageError {
    StorageError::Failed { message: format!("{e:?}") }
}

impl JsStorage {
    async fn invoke(
        &self,
        name: &str,
        args: &[JsValue],
    ) -> Result<JsValue, StorageError> {
        let f: Function = Reflect::get(&self.0, &name.into())
            .map_err(storage_err)?
            .dyn_into()
            .map_err(storage_err)?;
        let arr = js_sys::Array::new();
        for a in args {
            arr.push(a);
        }
        let ret = f.apply(&self.0, &arr).map_err(storage_err)?;
        match ret.dyn_into::<Promise>() {
            Ok(p) => JsFuture::from(p).await.map_err(storage_err),
            Err(v) => Ok(v),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl Storage for JsStorage {
    async fn get(&self, key: String) -> Result<Option<String>, StorageError> {
        Ok(self.invoke("get", &[key.into()]).await?.as_string())
    }
    async fn set(
        &self,
        key: String,
        value: String,
    ) -> Result<(), StorageError> {
        self.invoke("set", &[key.into(), value.into()]).await.map(|_| ())
    }
    async fn remove(&self, key: String) -> Result<(), StorageError> {
        self.invoke("remove", &[key.into()]).await.map(|_| ())
    }
    async fn keys(&self, prefix: String) -> Result<Vec<String>, StorageError> {
        let v = self.invoke("keys", &[prefix.into()]).await?;
        Ok(js_sys::Array::from(&v)
            .iter()
            .filter_map(|k| k.as_string())
            .collect())
    }
}

struct JsListener(Function);

impl Listener for JsListener {
    fn on_event(&self, event: SignEvent) {
        let json = serde_json::to_string(&event).expect("json");
        let _ = self.0.call1(&JsValue::NULL, &json.into());
    }
}

fn js_err(e: crate::SignError) -> JsValue {
    JsValue::from_str(&super::json::error_json(&e))
}

#[wasm_bindgen]
pub struct WasmSignClient {
    client: SignClient,
}

#[wasm_bindgen]
impl WasmSignClient {
    /// `storage`: `{ get(k), set(k, v), remove(k), keys(prefix) }`, each may return a Promise.
    /// `onEvent`: called with each event as a JSON string.
    pub async fn create(
        config_json: String,
        storage: JsValue,
        on_event: Function,
    ) -> Result<WasmSignClient, JsValue> {
        let cfg: SignConfig =
            crate::protocol::ijson::decode(config_json.as_bytes())
                .ok_or_else(|| JsValue::from_str("invalid config"))?;
        let client = SignClient::init(
            cfg,
            Arc::new(JsStorage(storage)),
            Arc::new(JsListener(on_event)),
        )
        .await
        .map_err(js_err)?;
        Ok(WasmSignClient { client })
    }

    pub fn start(&self) {
        self.client.start();
    }

    pub fn stop(&self) {
        self.client.stop();
    }

    pub fn foreground(&self) {
        self.client.foreground();
    }

    #[wasm_bindgen(js_name = clientId)]
    pub fn client_id(&self) -> String {
        self.client.client_id()
    }

    /// The JSON facade: resolves with the result JSON, rejects with `{ code, message }` JSON.
    pub fn call(&self, method: String, params_json: String) -> Promise {
        let client = self.client.clone();
        future_to_promise(async move {
            super::json::call(&client, &method, &params_json)
                .await
                .map(|s| JsValue::from_str(&s))
                .map_err(js_err)
        })
    }
}
