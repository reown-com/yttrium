use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};
use thaw::{Button, Flex, Input, Label, ToastOptions, ToasterInjection};
use yttrium::sign::{
    generate_key, protocol_types::SessionRequestResponseJsonRpc,
    ApprovedSession, Client, SecretKey,
};

use crate::toast::{show_error_toast, show_success_toast, show_toast};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct MyState {
    key: SecretKey,
    sessions: Vec<ApprovedSession>,
}

#[component]
pub fn App() -> impl IntoView {
    let toaster = ToasterInjection::expect_context();

    let my_state = RwSignal::new(None::<MyState>);

    let pairing_uri = RwSignal::new(String::new());

    let (client, request_rx) =
        Client::new(include_str!("../.project-id").trim().into());
    let client = StoredValue::new(Arc::new(tokio::sync::Mutex::new(client)));
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
                                show_success_toast(
                                    toaster,
                                    "Pairing approved".to_owned(),
                                );
                            });
                        }
                        Err(e) => {
                            show_error_toast(
                                toaster,
                                format!("Approval failed: {e}"),
                            );
                        }
                    },
                    Err(e) => {
                        show_error_toast(
                            toaster,
                            format!("Pairing failed: {e}"),
                        );
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
                                    tracing::info!(
                                        "message on topic: {:?}: {:?}",
                                        message.0,
                                        message.1
                                    );
                                    show_toast(
                                        toaster,
                                        serde_json::to_string(&message)
                                            .unwrap(),
                                        ToastOptions::default().with_timeout(
                                            Duration::from_secs(10),
                                        ),
                                    );
                                    match message.1.params.request.method.as_str() {
                                        "personal_sign" => {
                                            let mut client =
                                                client.lock().await;
                                            client
                                                .respond(
                                                    message.0,
                                                    message.1.id,
                                                    SessionRequestResponseJsonRpc {
                                                        id: message.1.id,
                                                        jsonrpc: "2.0".to_string(),
                                                        result: "0x0".to_string().into(),
                                                    },
                                                )
                                                .await
                                                .unwrap();
                                        }
                                        method => {
                                            tracing::error!("Unexpected method: {method}");
                                        }
                                    }
                                }
                                None => {
                                    show_error_toast(
                                        toaster,
                                        "Next failed".to_owned(),
                                    );
                                }
                            }
                        }
                    });
                }
            });
        }
    });

    view! {
        <Flex vertical=true>
            <Flex>
                <Label prop:for="pairing-uri">"Pairing URI"</Label>
                <Input id="pairing-uri" value=pairing_uri />
                <Button on_click=move |_| {
                    pair_action.dispatch(pairing_uri.get());
                }>"Pair"</Button>
            </Flex>
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
        </Flex>
    }
}
