use {
    crate::sign::{
        relay::Attestation,
        utils::{DecryptedHash, EncryptedHash},
        verify::VERIFY_SERVER_URL,
    },
    serde::Deserialize,
    std::{cell::RefCell, rc::Rc, sync::Arc},
    wasm_bindgen::{closure::Closure, JsCast},
    web_sys::js_sys,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AttestationResponse {
    r#type: String,
    attestation: String,
}

pub fn create_attestation(
    encrypted_id: EncryptedHash,
    decrypted_id: DecryptedHash,
    project_id: relay_rpc::domain::ProjectId,
) -> Result<tokio::sync::oneshot::Receiver<Attestation>, String> {
    let window = web_sys::window().ok_or_else(|| "no window".to_string())?;
    let document = window.document().ok_or("no document".to_string())?;
    let body = document.body().ok_or("no body".to_string())?;

    let origin =
        window.location().origin().map_err(|e| format!("get origin: {e:?}"))?;

    let encoded_origin = js_sys::encode_uri_component(&origin);
    let iframe_src = format!(
        "{VERIFY_SERVER_URL}/v3/attestation?projectId={}&id={}&decryptedId={}&origin={}",
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

    let (tx, rx) = tokio::sync::oneshot::channel();
    let tx = Arc::new(std::sync::Mutex::new(Some(tx)));

    let closure_rc: Rc<RefCell<Option<Closure<_>>>> =
        Rc::new(RefCell::new(None));

    let closure = {
        let closure_rc = closure_rc.clone();
        let window = window.clone();
        Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
            if event.origin() != VERIFY_SERVER_URL {
                return;
            }

            let event_data = event.data();
            let data = if let Some(json_str) = event_data.as_string() {
                match serde_json::from_str::<AttestationResponse>(&json_str) {
                    Ok(parsed) => parsed,
                    Err(_) => {
                        tracing::warn!(
                            "postmessage from Verify origin but didn't parse as AttestationResponse"
                        );
                        return;
                    }
                }
            } else {
                tracing::warn!(
                    "postmessage from Verify origin but was not a string"
                );
                return;
            };

            if data.r#type == "verify_attestation" {
                let closure = closure_rc.borrow_mut().take();
                if let Some(closure) = closure {
                    if let Some(tx) = tx.lock().unwrap().take() {
                        let _ = tx.send(Some(data.attestation.into()));
                    } else {
                        tracing::error!("tx is already taken");
                    }

                    if let Err(e) = window.remove_event_listener_with_callback(
                        "message",
                        closure.as_ref().unchecked_ref(),
                    ) {
                        tracing::error!("remove event listener: {e:?}");
                    }

                    if let Err(e) = body.remove_child(&iframe) {
                        tracing::error!("remove child: {e:?}");
                    }
                }
            }
        }) as Box<dyn FnMut(_)>)
    };
    *closure_rc.borrow_mut() = Some(closure);

    window
        .add_event_listener_with_callback(
            "message",
            closure_rc.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
        )
        .map_err(|e| format!("add event listener: {e:?}"))?;

    Ok(rx)
}
