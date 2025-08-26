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
        client::{generate_client_id_key, Client},
        client_types::{ConnectParams, Session, SessionStore},
        protocol_types::{
            Metadata, SessionProposal, SessionRequestJsonRpc,
            SessionRequestJsonRpcResponse, SessionRequestJsonRpcResultResponse,
            SettleNamespace,
        },
        IncomingSessionMessage, SecretKey, Topic,
    },
};

#[derive(Serialize, Deserialize, Clone)]
struct MyState {
    key: SecretKey,
    sessions: Vec<Session>,
}

struct MySessionStore;

fn read_local_storage() -> MyState {
    let state = web_sys::window()
        .unwrap()
        .local_storage()
        .unwrap()
        .unwrap()
        .get_item("wc")
        .unwrap();
    if let Some(state) = state {
        if let Ok(state) = serde_json::from_str(&state) {
            state
        } else {
            MyState { key: generate_client_id_key(), sessions: Vec::new() }
        }
    } else {
        MyState { key: generate_client_id_key(), sessions: Vec::new() }
    }
}

fn write_local_storage(state: MyState) {
    web_sys::window()
        .unwrap()
        .local_storage()
        .unwrap()
        .unwrap()
        .set_item("wc", &serde_json::to_string(&state).unwrap())
        .unwrap();
}

impl SessionStore for MySessionStore {
    fn get_all_sessions(&self) -> Vec<Session> {
        read_local_storage().sessions
    }

    fn add_session(&self, session: Session) {
        let mut state = read_local_storage();
        state.sessions.push(session);
        write_local_storage(state);
    }

    fn delete_session(&self, topic: String) {
        let mut state = read_local_storage();
        state
            .sessions
            .retain(|session| session.topic.value().to_string() != topic);
        write_local_storage(state);
    }

    fn get_session(&self, topic: String) -> Option<Session> {
        read_local_storage()
            .sessions
            .into_iter()
            .find(|session| session.topic.value().to_string() == topic)
    }
}

// TODO reject session request

#[component]
pub fn App() -> impl IntoView {
    let toaster = ToasterInjection::expect_context();

    let sessions = RwSignal::new(Vec::new());

    let pairing_uri = RwSignal::new(String::new());
    let client =
        StoredValue::new(None::<std::sync::Arc<tokio::sync::Mutex<Client>>>);

    let pairing_request =
        RwSignal::new(None::<RwSignal<Option<SessionProposal>>>);
    let pairing_request_open = RwSignal::new(false);
    let pair_action = Action::new({
        move |pairing_uri: &String| {
            let signal = RwSignal::new(None::<SessionProposal>);
            pairing_request_open.set(true);
            pairing_request.set(Some(signal));
            let client = client.read_value().as_ref().unwrap().clone();
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
            let client = client.read_value().as_ref().unwrap().clone();
            async move {
                let mut client_guard = client.lock().await;

                let mut namespaces = HashMap::new();
                for (namespace, namespace_proposal) in
                    pairing.optional_namespaces.clone().unwrap()
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
                        chains: namespace_proposal.chains,
                    };
                    namespaces.insert(namespace, namespace_settle);
                }
                tracing::debug!("namespaces: {:?}", namespaces);

                let metadata = Metadata {
                    name: "Reown Rust Sample Wallet".to_string(),
                    description: "Reown Rust Sample Wallet".to_string(),
                    url: "https://reown.com".to_string(),
                    icons: vec![],
                    verify_url: None,
                    redirect: None,
                };

                match client_guard.approve(pairing, namespaces, metadata).await
                {
                    Ok(_approved_session) => {
                        leptos::task::spawn_local(async move {
                            show_success_toast(
                                toaster,
                                "Pairing approved".to_owned(),
                            );
                            pairing_request_open.set(false);

                            yttrium::time::sleep(
                                std::time::Duration::from_secs(1),
                            )
                            .await;
                            pairing_request.set(None);
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

    let reject_pairing_action = Action::new({
        move |pairing: &SessionProposal| {
            let pairing = pairing.clone();
            let client = client.read_value().as_ref().unwrap().clone();
            async move {
                let mut client = client.lock().await;
                match client
                    .reject(
                        pairing,
                        yttrium::sign::ErrorData {
                            code: 5000,
                            message: "User rejected.".to_owned(),
                            data: None,
                        },
                    )
                    .await
                {
                    Ok(_) => {
                        show_success_toast(
                            toaster,
                            "Pairing rejected".to_owned(),
                        );
                        pairing_request_open.set(false);

                        yttrium::time::sleep(std::time::Duration::from_secs(1))
                            .await;
                        pairing_request.set(None);
                    }
                    Err(e) => {
                        show_error_toast(
                            toaster,
                            format!("Pairing rejection failed: {e}"),
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
            let client = client.read_value().as_ref().unwrap().clone();
            async move {
                let mut client = client.lock().await;
                match client
                    .respond(
                        request.0,
                        SessionRequestJsonRpcResponse::Result(
                            SessionRequestJsonRpcResultResponse {
                                id: request.1.id,
                                jsonrpc: "2.0".to_string(),
                                result: "0x0".to_string().into(),
                            },
                        ),
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

    let connect_uri = RwSignal::new(None::<Option<String>>);
    // let connect_request_open = RwSignal::new(false);
    let connect_action = Action::new({
        move |_request: &()| {
            connect_uri.set(Some(None));
            let client = client.read_value().as_ref().unwrap().clone();
            async move {
                let mut client = client.lock().await;
                match client
                    .connect(
                        ConnectParams {
                            optional_namespaces: None,
                            relays: None,
                            session_properties: None,
                            scoped_properties: None,
                        },
                        Metadata {
                            name: "Reown Rust Sample App".to_string(),
                            description: "Reown Rust Sample App".to_string(),
                            url: "https://reown.com".to_string(),
                            icons: vec![],
                            verify_url: None,
                            redirect: None,
                        },
                    )
                    .await
                {
                    Ok(connect_result) => {
                        connect_uri.set(Some(Some(connect_result.uri)));
                        leptos::task::spawn_local(async move {
                            yttrium::time::sleep(
                                std::time::Duration::from_secs(5 * 60), // TODO use actual expiry
                            )
                            .await;
                            connect_uri.set(None);
                            show_error_toast(
                                toaster,
                                "Connection expired".to_owned(),
                            );
                        });
                    }
                    Err(e) => {
                        show_error_toast(
                            toaster,
                            format!("Connection propose failed: {e}"),
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
        let unmounted = unmounted.clone();
        move |_| {
            let unmounted = unmounted.clone();
            client.update_value(|client| {
                assert!(client.is_none());

                let (new_client, mut request_rx) = Client::new(
                    std::option_env!("REOWN_PROJECT_ID").unwrap_or("").into(),
                    read_local_storage().key,
                    Arc::new(MySessionStore),
                );
                let client_arc = Arc::new(tokio::sync::Mutex::new(new_client));
                *client = Some(client_arc.clone());

                leptos::task::spawn_local(async move {
                    {
                        let mut client = client_arc.lock().await;
                        if !read_local_storage().sessions.is_empty() {
                            client.online();
                        }
                    }
                    while !unmounted.load(std::sync::atomic::Ordering::Relaxed)
                    {
                        let next = request_rx.recv().await;
                        match next {
                            Some((topic, message)) => {
                                sessions.set(read_local_storage().sessions);
                                match message {
                                    IncomingSessionMessage::SessionRequest(request) => {
                                        tracing::info!(
                                            "signature request on topic: {:?}: {:?}",
                                            topic,
                                            request
                                        );
                                        match request.params.request.method.as_str() {
                                            "personal_sign" => {
                                                signature_request_open.set(true);
                                                signature_request.set(Some((topic, request)));
                                            }
                                            method => {
                                                tracing::error!(
                                                    "Unexpected method: {method}"
                                                );
                                            }
                                        }
                                    }
                                    IncomingSessionMessage::Disconnect(id, topic) => {
                                        tracing::info!(
                                            "session delete on topic: {id}: {topic}",
                                        );
                                    }
                                    IncomingSessionMessage::SessionEvent(id, topic, params) => {
                                        tracing::info!(
                                            "session event on topic: {id}: {topic}: {params:?}",
                                        );
                                    }
                                    IncomingSessionMessage::SessionUpdate(id, topic, params) => {
                                        tracing::info!(
                                            "session update on topic: {id}: {topic}: {params:?}",
                                        );
                                    }
                                    IncomingSessionMessage::SessionExtend(id, topic) => {
                                        tracing::info!(
                                            "session extend on topic: {id}: {topic}",
                                        );
                                    }
                                    IncomingSessionMessage::SessionConnect(id) => {
                                        tracing::info!(
                                            "session connect on topic: {id}",
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
            <Flex>
                <Button
                    on_click=move |_| {
                        connect_action.dispatch(());
                    }>
                    "Connect"
                </Button>
            </Flex>
            <ul>
                {move || sessions.get().iter().map(|session| {
                    let topic = session.topic.clone();
                    view! {
                        <li>
                            "Session"
                            <Button
                                on_click=move |_| {
                                    let topic = topic.clone();
                                    leptos::task::spawn_local(async move {
                                        let client = client.read_value().as_ref().unwrap().clone();
                                        let mut client = client.lock().await;
                                        match client.disconnect(topic).await {
                                            Ok(_) => {
                                                show_success_toast(toaster, "Disconnected".to_owned());
                                            }
                                            Err(e) => {
                                                show_error_toast(toaster, format!("Disconnect failed: {e}"));
                                            }
                                        }
                                    });
                                }>
                                "Disconnect"
                            </Button>
                        </li>
                    }
                }).collect::<Vec<_>>()}
            </ul>
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
                                            loading=reject_pairing_action.pending()
                                            on_click={
                                                let _request = request.clone();
                                                move |_| {
                                                    reject_pairing_action.dispatch(request.clone());
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
        {move || {
            view! {
                <Dialog open=connect_uri.get().is_some()>
                    <DialogSurface>
                        <DialogBody>
                            <DialogTitle>"Connect"</DialogTitle>
                            <DialogContent>
                                {move || connect_uri.get().unwrap_or_default().map(|uri| {
                                    view! {
                                        <p>{uri.clone()}</p>
                                        <Button on_click=move |_| {
                                            pair_action.dispatch(uri.clone());
                                        }>
                                            "Self connect"
                                        </Button>
                                    }.into_any()
                                }).unwrap_or_else(|| view! {
                                    <Spinner />
                                }.into_any())}
                            </DialogContent>
                            // <DialogActions>
                                // <Button
                                //     loading=session_request_action.pending()
                                //     on_click={
                                //         let request = request.clone();
                                //         move |_| {
                                //             session_request_action.dispatch(request.clone());
                                //         }
                                //     }>
                                //     "Approve"
                                // </Button>
                                // <Button
                                //     loading=session_request_reject_action.pending()
                                //     on_click={
                                //         let request = request.clone();
                                //         move |_| {
                                //             session_request_reject_action.dispatch(request.clone());
                                //         }
                                //     }>
                                //         "Reject"
                                // </Button>
                            // </DialogActions>
                        </DialogBody>
                    </DialogSurface>
                </Dialog>
            }
        }}
    }
}
