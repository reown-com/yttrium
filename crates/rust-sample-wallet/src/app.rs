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
        DialogSurface, DialogTitle, Flex, Input, Label, Spinner,
        ToasterInjection,
    },
    yttrium::sign::{
        generate_key,
        protocol_types::{
            Metadata, SessionRequestJsonRpc, SessionRequestResponseJsonRpc,
            SettleNamespace,
        },
        ApprovedSession, Client, SecretKey, SessionProposal, Topic,
    },
};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct MyState {
    key: SecretKey,
    sessions: Vec<ApprovedSession>,
}

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

    let pairing_request =
        RwSignal::new(None::<RwSignal<Option<SessionProposal>>>);
    let pairing_request_open = RwSignal::new(false);
    let pair_action = Action::new({
        move |pairing_uri: &String| {
            let signal = RwSignal::new(None::<SessionProposal>);
            pairing_request_open.set(true);
            pairing_request.set(Some(signal));
            let client = client.read_value().clone();
            let pairing_uri = pairing_uri.clone();
            async move {
                let mut client = client.lock().await;
                match client.pair(&pairing_uri).await {
                    Ok(pairing) => {
                        signal.set(Some(pairing));
                    }
                    Err(e) => {
                        show_error_toast(
                            toaster,
                            format!("Pairing failed: {e}"),
                        );
                        pairing_request_open.set(false);
                        leptos::task::spawn_local(async move {
                            yttrium::time::sleep(
                                std::time::Duration::from_secs(1),
                            )
                            .await;
                            pairing_request.set(None);
                        });
                    }
                }
            }
        }
    });

    let approve_pairing_action = Action::new({
        move |pairing: &SessionProposal| {
            let pairing = pairing.clone();
            let client = client.read_value().clone();
            async move {
                let mut client = client.lock().await;

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
                                chain,
                                "0x0000000000000000000000000000000000000000"
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

                match client.approve(pairing, namespaces, metadata).await {
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
                                    &serde_json::to_string(&my_state).unwrap(),
                                )
                                .unwrap();
                            show_success_toast(
                                toaster,
                                "Pairing approved".to_owned(),
                            );
                            pairing_request_open.set(false);
                            leptos::task::spawn_local(async move {
                                yttrium::time::sleep(
                                    std::time::Duration::from_secs(1),
                                )
                                .await;
                                pairing_request.set(None);
                            });
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
        }
    });

    let signature_request =
        RwSignal::new(None::<(Topic, SessionRequestJsonRpc)>);
    let signature_request_open = RwSignal::new(false);
    let session_request_action = Action::new({
        move |request: &(Topic, SessionRequestJsonRpc)| {
            let request = request.clone();
            let client = client.read_value().clone();
            async move {
                let mut client = client.lock().await;
                match client
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
                {
                    Ok(_) => {
                        signature_request_open.set(false);
                        leptos::task::spawn_local(async move {
                            yttrium::time::sleep(
                                std::time::Duration::from_secs(1),
                            )
                            .await;
                            signature_request.set(None);
                        });
                        show_success_toast(
                            toaster,
                            "Signature approved".to_owned(),
                        );
                    }
                    Err(e) => {
                        show_error_toast(
                            toaster,
                            format!("Signature approval failed: {e}"),
                        );
                    }
                }
            }
        }
    });
    let session_request_reject_action = Action::new({
        move |_request: &(Topic, SessionRequestJsonRpc)| async move {
            show_error_toast(
                toaster,
                "Signature rejection not yet supported".to_owned(),
            );
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
                            client.set_key(state.key);
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
                                            signature_request_open.set(true);
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
                <Button
                    loading=pair_action.pending()
                    on_click=move |_| {
                        pair_action.dispatch(pairing_uri.get());
                        pairing_uri.set(String::new());
                    }>
                    "Pair"
                </Button>
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
        {move || pairing_request.get().map(|request| {
            view! {
                <Dialog open=pairing_request_open>
                    <DialogSurface>
                        <DialogBody>
                            <DialogTitle>"Approve pairing"</DialogTitle>
                            {move || request.get().map(|request| {
                                // TODO avoid flash here
                                view!{
                                    <DialogContent>
                                        {format!("{request:?}")}
                                    </DialogContent>
                                    <DialogActions>
                                        <Button
                                            loading=approve_pairing_action.pending()
                                            on_click={
                                                let request = request.clone();
                                                move |_| {
                                                    approve_pairing_action.dispatch(request.clone());
                                                }
                                            }>
                                            "Approve"
                                        </Button>
                                        <Button
                                            // loading=session_request_reject_action.pending()
                                            on_click={
                                                let _request = request.clone();
                                                move |_| {
                                                    // session_request_reject_action.dispatch(request.clone());
                                                }
                                            }>
                                                "Reject"
                                        </Button>
                                    </DialogActions>
                                }.into_any()
                            }).unwrap_or_else(|| view! {
                                <DialogContent>
                                    <Spinner/>
                                </DialogContent>
                            }.into_any())}
                        </DialogBody>
                    </DialogSurface>
                </Dialog>
            }
        })}
        {move || signature_request.get().map(|request| {
            view! {
                <Dialog open=signature_request_open>
                    <DialogSurface>
                        <DialogBody>
                            <DialogTitle>"Signature request"</DialogTitle>
                            <DialogContent>
                                {format!("{request:?}")}
                            </DialogContent>
                            <DialogActions>
                                <Button
                                    loading=session_request_action.pending()
                                    on_click={
                                        let request = request.clone();
                                        move |_| {
                                            session_request_action.dispatch(request.clone());
                                        }
                                    }>
                                    "Approve"
                                </Button>
                                <Button
                                    loading=session_request_reject_action.pending()
                                    on_click={
                                        let request = request.clone();
                                        move |_| {
                                            session_request_reject_action.dispatch(request.clone());
                                        }
                                    }>
                                        "Reject"
                                </Button>
                            </DialogActions>
                        </DialogBody>
                    </DialogSurface>
                </Dialog>
            }
        })}
    }
}
