use {
    crate::sign::utils::{DecryptedHash, EncryptedHash},
    std::sync::Arc,
    wasm_bindgen::{closure::Closure, JsCast, JsValue},
    web_sys::js_sys,
};

pub async fn create_attestation(
    encrypted_id: EncryptedHash,
    decrypted_id: DecryptedHash,
    project_id: relay_rpc::domain::ProjectId,
) -> Result<Option<String>, String> {
    let window = web_sys::window().ok_or_else(|| "no window".to_string())?;
    let document = window.document().ok_or("no document".to_string())?;
    let body = document.body().ok_or("no body".to_string())?;

    let origin =
        window.location().origin().map_err(|e| format!("get origin: {e:?}"))?;

    // URL encode the origin using js_sys
    let encoded_origin = js_sys::encode_uri_component(&origin);

    let iframe_src = format!(
        "https://verify.walletconnect.org/v3/attestation?projectId={}&id={}&decryptedId={}&origin={}",
        project_id.as_ref(),
        encrypted_id.as_str(),
        decrypted_id.as_str(),
        encoded_origin
    );

    let iframe = document
        .create_element("iframe")
        .map_err(|e| format!("create iframe: {e:?}"))?;
    iframe
        .set_attribute("src", &iframe_src)
        .map_err(|e| format!("set src: {e:?}"))?;
    iframe
        .set_attribute("style", "display: none;")
        .map_err(|e| format!("set style: {e:?}"))?;

    body.append_child(&iframe).map_err(|e| format!("append iframe: {e:?}"))?;

    let (tx, rx) =
        tokio::sync::oneshot::channel::<Result<Option<String>, String>>();
    let tx = Arc::new(std::sync::Mutex::new(Some(tx)));

    let tx_clone = tx.clone();
    let iframe_clone = iframe.clone();
    let closure = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
        let event_data = event.data();

        // The iframe sends JSON.stringify'd data, so we need to parse it
        let data = if let Some(json_str) = event_data.as_string() {
            // Parse the JSON string
            match js_sys::JSON::parse(&json_str) {
                Ok(parsed) => parsed,
                Err(_) => return, // Not valid JSON, ignore
            }
        } else if event_data.is_object() {
            // Already an object (in case the implementation changes)
            event_data
        } else {
            return; // Neither string nor object, ignore
        };

        // Now work with the data as an object
        if let Ok(obj) = data.dyn_into::<js_sys::Object>() {
            if let Ok(type_val) =
                js_sys::Reflect::get(&obj, &JsValue::from_str("type"))
            {
                if let Some(type_str) = type_val.as_string() {
                    if type_str == "verify_attestation" {
                        let attestation = js_sys::Reflect::get(
                            &obj,
                            &JsValue::from_str("attestation"),
                        )
                        .ok()
                        .and_then(|v| v.as_string());

                        if let Some(tx) = tx_clone.lock().unwrap().take() {
                            let _ = tx.send(Ok(attestation));

                            if let Some(parent) = iframe_clone.parent_node() {
                                let _ = parent.remove_child(&iframe_clone);
                            }
                        }
                    }
                }
            }
        }
    }) as Box<dyn FnMut(_)>);

    window
        .add_event_listener_with_callback(
            "message",
            closure.as_ref().unchecked_ref(),
        )
        .map_err(|e| format!("add event listener: {e:?}"))?;

    // Set a timeout of 5 seconds
    let timeout_tx = tx.clone();
    let iframe_timeout = iframe.clone();
    let timeout_closure = Closure::once(Box::new(move || {
        if let Some(tx) = timeout_tx.lock().unwrap().take() {
            tracing::warn!("Verify V3 attestation timeout");
            let _ = tx.send(Ok(None));

            // Remove the iframe
            if let Some(parent) = iframe_timeout.parent_node() {
                let _ = parent.remove_child(&iframe_timeout);
            }
        }
    }) as Box<dyn FnOnce()>);

    window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            timeout_closure.as_ref().unchecked_ref(),
            5000,
        )
        .map_err(|e| format!("set timeout: {e:?}"))?;

    // Keep closures alive
    closure.forget();
    timeout_closure.forget();

    // Wait for the result
    rx.await.map_err(|e| format!("recv: {e:?}"))?
}
