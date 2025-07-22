use leptos::{prelude::*, server::codee::string::JsonSerdeCodec};
use leptos_use::storage::use_local_storage;
use serde::{Deserialize, Serialize};
use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::Mutex;
use yttrium::sign::{ApprovedSession, Client};

#[derive(Serialize, Deserialize, Clone, Default, PartialEq)]
struct MyState {
    sessions: Vec<ApprovedSession>,
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| {
        let (state, set_state, _) =
            use_local_storage::<MyState, JsonSerdeCodec>("wc.sessions");
        let pairing_uri = RwSignal::new(String::new());
        let pairing_status = RwSignal::new(String::new());
        let (client, request_rx) =
            Client::new(include_str!("../.project-id").trim().into());
        let client = Arc::new(Mutex::new(client));
        let request_rx = StoredValue::new(Some(request_rx));

        let pair_action = Action::new({
            let client = client.clone();
            move |pairing_uri: &String| {
                let client = client.clone();
                let pairing_uri = pairing_uri.clone();
                async move {
                    let mut client = client.lock().await;
                    match client.pair(&pairing_uri).await {
                        Ok(pairing) => match client.approve(pairing).await {
                            Ok(approved_session) => {
                                set_state.update(|state| {
                                    state.sessions.push(approved_session);
                                });
                                pairing_status
                                    .set("Pairing approved".to_owned());
                            }
                            Err(e) => {
                                pairing_status
                                    .set(format!("Approval failed: {e}"));
                            }
                        },
                        Err(e) => {
                            pairing_status.set(format!("Pairing failed: {e}"));
                        }
                    }
                }
            }
        });

        let unmounted = Arc::new(AtomicBool::new(false));
        on_cleanup({
            let unmounted = unmounted.clone();
            move || {
                unmounted.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        });

        Effect::new({
            let unmounted = unmounted.clone();
            move |_| {
                let unmounted = unmounted.clone();
                request_rx.update_value(|request_rx| {
                    let request_rx = request_rx.take();
                    if let Some(mut request_rx) = request_rx {
                        leptos::task::spawn_local(async move {
                            while !unmounted
                                .load(std::sync::atomic::Ordering::Relaxed)
                            {
                                let next = request_rx.recv().await;
                                match next {
                                    Some(message) => {
                                        pairing_status.set(message);
                                    }
                                    None => {
                                        pairing_status
                                            .set("Next failed".to_owned());
                                    }
                                }
                            }
                        });
                    }
                });
            }
        });

        view! {
            <label for="pairing-uri">Pairing URI</label>
            <input type="text" id="pairing-uri" prop:value=pairing_uri on:input:target=move |ev| {
                pairing_uri.set(ev.target().value());
            } />
            <button on:click=move |_| {
                pair_action.dispatch(pairing_uri.get());
            }>Pair</button>
            <p>"Pairing status: " {pairing_status}</p>
            <ul>
                {move || state.get().sessions.iter().map(|_session| {
                    view! {
                        <li>"Session"</li>
                    }
                }).collect::<Vec<_>>()}
            </ul>
        }
    })
}
