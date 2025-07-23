use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{atomic::AtomicBool, Arc};
use yttrium::sign::{generate_key, ApprovedSession, Client, SecretKey};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct MyState {
    key: SecretKey,
    sessions: Vec<ApprovedSession>,
}

fn main() {
    console_error_panic_hook::set_once();
    tracing_subscriber::fmt()
        .with_writer(
            // To avoide trace events in the browser from showing their
            // JS backtrace, which is very annoying, in my opinion
            tracing_subscriber_wasm::MakeConsoleWriter::default()
                .map_trace_level_to(tracing::Level::DEBUG),
        )
        .with_max_level(tracing::Level::INFO)
        // For some reason, if we don't do this in the browser, we get
        // a runtime error.
        .without_time()
        .init();
    leptos::mount::mount_to_body(|| {
        let my_state = RwSignal::new(None::<MyState>);

        let pairing_uri = RwSignal::new(String::new());
        let pairing_status = RwSignal::new(String::new());

        let (client, request_rx) =
            Client::new(include_str!("../.project-id").trim().into());
        let client =
            StoredValue::new(Arc::new(tokio::sync::Mutex::new(client)));
        let request_rx = StoredValue::new(Some(request_rx));

        let pair_action = Action::new({
            move |pairing_uri: &String| {
                let client = client.read_value().clone();
                let pairing_uri = pairing_uri.clone();
                async move {
                    let mut client = client.lock().await;
                    match client.pair(&pairing_uri).await {
                        // TODO separate action & UI for approval
                        Ok(pairing) => match client.approve(pairing).await {
                            Ok(_approved_session) => {
                                my_state.update(|my_state| {
                                    let my_state = my_state.as_mut().unwrap();
                                    my_state.sessions = client.get_sessions();
                                    web_sys::window()
                                        .unwrap()
                                        .local_storage()
                                        .unwrap()
                                        .unwrap()
                                        .set_item(
                                            "wc",
                                            &serde_json::to_string(&my_state)
                                                .unwrap(),
                                        )
                                        .unwrap();
                                    pairing_status
                                        .set("Pairing approved".to_owned());
                                });
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
            let client = client.read_value().clone();
            let unmounted = unmounted.clone();
            move |_| {
                let client = client.clone();
                let unmounted = unmounted.clone();
                request_rx.update_value(|request_rx| {
                    let request_rx = request_rx.take();
                    if let Some(mut request_rx) = request_rx {
                        leptos::task::spawn_local(async move {
                            let sessions = web_sys::window()
                                .unwrap()
                                .local_storage()
                                .unwrap()
                                .unwrap()
                                .get_item("wc")
                                .unwrap();
                            if let Some(sessions) = sessions {
                                let state =
                                    serde_json::from_str::<MyState>(&sessions)
                                        .unwrap();
                                my_state.set(Some(state.clone()));
                                let mut client = client.lock().await;
                                client.set_key(state.key);
                                if !state.sessions.is_empty() {
                                    client.add_sessions(state.sessions);
                                    client.online().await;
                                }
                            } else {
                                let state = MyState {
                                    key: generate_key(),
                                    sessions: Vec::new(),
                                };
                                let mut client = client.lock().await;
                                client.set_key(state.key.clone());
                                my_state.set(Some(state.clone()));
                                web_sys::window()
                                    .unwrap()
                                    .local_storage()
                                    .unwrap()
                                    .unwrap()
                                    .set_item(
                                        "wc",
                                        &serde_json::to_string(&state).unwrap(),
                                    )
                                    .unwrap();
                            }

                            while !unmounted
                                .load(std::sync::atomic::Ordering::Relaxed)
                            {
                                let next = request_rx.recv().await;
                                match next {
                                    Some(message) => {
                                        // TODO display signature request dialog
                                        tracing::info!("message: {}", message);
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
            {move || my_state.get().map(|my_state| {
                view! {
                    <ul>
                        {move || my_state.sessions.iter().map(|_session| {
                            view! {
                                <li>"Session"</li>
                            }
                        }).collect::<Vec<_>>()}
                    </ul>
                }
            })}
        }
    })
}
