use {
    crate::toast::{show_error_toast, show_success_toast},
    leptos::prelude::*,
    serde::{Deserialize, Serialize},
    std::{
        collections::HashMap,
        sync::{atomic::AtomicBool, Arc},
    },
    thaw::{
        Button, Dialog, DialogActions, DialogBody, DialogContent,
        DialogSurface, DialogTitle, Flex, Input, Label, ToasterInjection,
    },
    yttrium::sign::{
        generate_key,
        protocol_types::{
            Metadata, SessionRequestJsonRpc, SessionRequestResponseJsonRpc,
            SettleNamespace,
        },
        ApprovedSession, Client, SecretKey, Topic,
    },
};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct MyState {
    key: SecretKey,
    sessions: Vec<ApprovedSession>,
}

// TODO refactor to use actions and separate components for session proposal and session request
// loading indicators on 2 approve buttons AND session proposal loading dialog (i.e. display immediately)
// TODO disconnect support
// TODO reject session proposal
// TODO reject session request

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
                    Ok(pairing) => {
                        let mut namespaces = HashMap::new();
                        for (namespace, namespace_proposal) in
                            pairing.requested_namespaces.clone()
                        {
                            let accounts = namespace_proposal
                                .chains
                                .iter()
                                .map(|chain| {
                                    format!(
                                        "{}:{}",
                                        chain, "0x0000000000000000000000000000000000000000"
                                    )
                                })
                                .collect();
                            let namespace_settle = SettleNamespace {
                                accounts,
                                methods: namespace_proposal.methods,
                                events: namespace_proposal.events,
                            };
                            namespaces.insert(namespace, namespace_settle);
                        }
                        tracing::debug!("namespaces: {:?}", namespaces);

                        let metadata = Metadata {
                            name: "Reown Rust Sample Wallet".to_string(),
                            description: "Reown Rust Sample Wallet".to_string(),
                            url: "https://reown.com".to_string(),
                            icons: vec![],
                        };

                        match client
                            .approve(pairing, namespaces, metadata)
                            .await
                        {
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
                        }
                    }
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

    let signature_request =
        RwSignal::new(None::<(Topic, SessionRequestJsonRpc)>);

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
                                        "signature request on topic: {:?}: {:?}",
                                        message.0,
                                        message.1
                                    );
                                    match message
                                        .1
                                        .params
                                        .request
                                        .method
                                        .as_str()
                                    {
                                        "personal_sign" => {
                                            signature_request
                                                .set(Some(message));
                                        }
                                        method => {
                                            tracing::error!(
                                                "Unexpected method: {method}"
                                            );
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
        {move || signature_request.get().map(|request| {
            view! {
                <Dialog open=true>
                    <DialogSurface>
                        <DialogBody>
                            <DialogTitle>"Signature request"</DialogTitle>
                            <DialogContent>
                                {format!("{:?}", request)}
                            </DialogContent>
                            <DialogActions>
                                <Button on_click=move |_| {
                                    let request = request.clone();
                                    let client = client.read_value().clone();
                                    // TODO move to action
                                    // TODO handle error
                                    // TODO loading indicator
                                    leptos::task::spawn_local(async move {
                                        let mut client =
                                                client.lock().await;
                                        client
                                            .respond(
                                                request.0,
                                                request.1.id,
                                                SessionRequestResponseJsonRpc {
                                                    id: request.1.id,
                                                    jsonrpc: "2.0".to_string(),
                                                    result: "0x0".to_string().into(),
                                                },
                                            )
                                                .await
                                                .unwrap();
                                        signature_request.set(None);
                                        show_success_toast(
                                            toaster,
                                            "Signature approved".to_owned(),
                                        );
                                    });
                                }>
                                    "Approve"
                                 </Button>
                            </DialogActions>
                        </DialogBody>
                    </DialogSurface>
                </Dialog>
            }
        })}
    }
}
