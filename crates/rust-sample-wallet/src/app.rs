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
        client_types::{
            ConnectParams, RejectionReason, Session, SessionProposal, Storage,
        },
        protocol_types::{
            Metadata, ProposalNamespace, SessionRequestJsonRpc,
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
    pairing_keys: HashMap<Topic, [u8; 32]>,
}

struct MySessionStore {
    key: String,
}

const WALLET_KEY: &str = "wc-wallet";
const APP_KEY: &str = "wc-app";

// fn read_wallet_storage() -> MyState {
//     read_local_storage("wc-wallet")
// }

// fn read_app_storage() -> MyState {
//     read_local_storage("wc-app")
// }

fn read_local_storage(key: &str) -> MyState {
    let state = web_sys::window()
        .unwrap()
        .local_storage()
        .unwrap()
        .unwrap()
        .get_item(key)
        .unwrap();
    if let Some(state) = state {
        if let Ok(state) = serde_json::from_str(&state) {
            state
        } else {
            MyState {
                key: generate_client_id_key(),
                sessions: Vec::new(),
                pairing_keys: HashMap::new(),
            }
        }
    } else {
        MyState {
            key: generate_client_id_key(),
            sessions: Vec::new(),
            pairing_keys: HashMap::new(),
        }
    }
}

// fn write_wallet_storage(state: MyState) {
//     write_local_storage("wc-wallet", state);
// }

// fn write_app_storage(state: MyState) {
//     write_local_storage("wc-app", state);
// }

fn write_local_storage(key: &str, state: MyState) {
    web_sys::window()
        .unwrap()
        .local_storage()
        .unwrap()
        .unwrap()
        .set_item(key, &serde_json::to_string(&state).unwrap())
        .unwrap();
}

impl Storage for MySessionStore {
    fn get_all_sessions(&self) -> Vec<Session> {
        read_local_storage(&self.key).sessions
    }

    fn add_session(&self, session: Session) {
        let mut state = read_local_storage(&self.key);
        state.sessions.push(session);
        write_local_storage(&self.key, state);
    }

    fn delete_session(&self, topic: Topic) -> Option<Session> {
        let mut state = read_local_storage(&self.key);
        let session = state
            .sessions
            .iter()
            .find(|session| session.topic == topic)
            .cloned();
        state.sessions.retain(|session| session.topic != topic);
        write_local_storage(&self.key, state);
        session
    }

    fn get_session(&self, topic: Topic) -> Option<Session> {
        read_local_storage(&self.key)
            .sessions
            .into_iter()
            .find(|session| session.topic == topic)
    }

    fn get_decryption_key_for_topic(&self, topic: Topic) -> Option<[u8; 32]> {
        read_local_storage(&self.key)
            .sessions
            .into_iter()
            .find(|session| session.topic == topic)
            .map(|session| session.session_sym_key)
            .or_else(|| {
                read_local_storage(&self.key).pairing_keys.get(&topic).cloned()
            })
    }

    fn save_pairing_key(&self, topic: Topic, sym_key: [u8; 32]) {
        let mut state = read_local_storage(&self.key);
        state.pairing_keys.insert(topic, sym_key);
        write_local_storage(&self.key, state);
    }
}

// TODO reject session request

#[component]
pub fn App() -> impl IntoView {
    let toaster = ToasterInjection::expect_context();

    let wallet_sessions = RwSignal::new(Vec::new());
    let app_sessions = RwSignal::new(Vec::new());

    let pairing_uri = RwSignal::new(String::new());

    struct Clients {
        wallet_client: Client,
        app_client: Client,
    }
    let clients =
        StoredValue::new(None::<std::sync::Arc<tokio::sync::Mutex<Clients>>>);

    let pairing_request =
        RwSignal::new(None::<RwSignal<Option<SessionProposal>>>);
    let pairing_request_open = RwSignal::new(false);
    let pair_action = Action::new({
        move |pairing_uri: &String| {
            let signal = RwSignal::new(None::<SessionProposal>);
            pairing_request_open.set(true);
            pairing_request.set(Some(signal));
            let client = clients.read_value().as_ref().unwrap().clone();
            let pairing_uri = pairing_uri.clone();
            async move {
                let mut client = client.lock().await;
                match client.wallet_client.pair(&pairing_uri).await {
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
            let client = clients.read_value().as_ref().unwrap().clone();
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

                match client_guard
                    .wallet_client
                    .approve(pairing, namespaces, metadata)
                    .await
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
            let client = clients.read_value().as_ref().unwrap().clone();
            async move {
                let mut client = client.lock().await;
                match client
                    .wallet_client
                    .reject(pairing, RejectionReason::UserRejected)
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
            let client = clients.read_value().as_ref().unwrap().clone();
            async move {
                let mut client = client.lock().await;
                match client
                    .wallet_client
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
            let client = clients.read_value().as_ref().unwrap().clone();
            async move {
                let mut client = client.lock().await;
                match client
                    .app_client
                    .connect(
                        ConnectParams {
                            optional_namespaces: HashMap::from([(
                                "eip155".to_string(),
                                ProposalNamespace {
                                    chains: vec!["eip155:11155111".to_string()],
                                    methods: vec!["personal_sign".to_string()],
                                    events: vec![],
                                },
                            )]),
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

            wallet_sessions.set(read_local_storage(WALLET_KEY).sessions);
            app_sessions.set(read_local_storage(APP_KEY).sessions);

            clients.update_value(|client| {
                assert!(client.is_none());

                let (new_wallet_client, mut wallet_request_rx) = Client::new(
                    std::option_env!("REOWN_PROJECT_ID").unwrap_or("").into(),
                    read_local_storage(WALLET_KEY).key,
                    Arc::new(MySessionStore {
                        key: WALLET_KEY.to_string(),
                    }),
                );
                let (new_app_client, mut app_request_rx) = Client::new(
                    std::option_env!("REOWN_PROJECT_ID").unwrap_or("").into(),
                    read_local_storage(APP_KEY).key,
                    Arc::new(MySessionStore {
                        key: APP_KEY.to_string(),
                    }),
                );

                let client_arc = Arc::new(tokio::sync::Mutex::new(Clients {
                    wallet_client: new_wallet_client,
                    app_client: new_app_client,
                }));
                *client = Some(client_arc.clone());

                leptos::task::spawn_local(async move {
                    {
                        let mut clients = client_arc.lock().await;
                        clients.wallet_client.online();
                        clients.app_client.online();
                    }
                    while !unmounted.load(std::sync::atomic::Ordering::Relaxed)
                    {
                        tokio::select!{
                            wallet_request = wallet_request_rx.recv() => {
                                match wallet_request {
                                    Some((topic, message)) => {
                                        wallet_sessions.set(read_local_storage(WALLET_KEY).sessions);
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
                                    None => break,
                                }
                            }
                            app_request = app_request_rx.recv() => {
                                match app_request {
                                    Some((topic, message)) => {
                                        app_sessions.set(read_local_storage(APP_KEY).sessions);
                                        match message {
                                            IncomingSessionMessage::SessionConnect(id) => {
                                                tracing::info!(
                                                    "(app) session connect on topic: {topic}: {id}",
                                                );
                                                connect_uri.set(None);
                                            }
                                            e => {
                                                tracing::error!(
                                                    "Unexpected message: {e:?}"
                                                );
                                            }
                                        }
                                    }
                                    None => break,
                                }
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
                    }
                >
                    "Pair"
                </Button>
            </Flex>
            <Flex>
                <Button on_click=move |_| {
                    connect_action.dispatch(());
                }>"Connect"</Button>
            </Flex>
            <ul>
                {move || {
                    wallet_sessions
                        .get()
                        .iter()
                        .map(|session| {
                            let topic = session.topic.clone();
                            view! {
                                <li>
                                    <Flex>
                                        "Wallet session"
                                        <Button on_click=move |_| {
                                            let topic = topic.clone();
                                            leptos::task::spawn_local(async move {
                                                let client = clients.read_value().as_ref().unwrap().clone();
                                                let mut client = client.lock().await;
                                                match client.wallet_client.disconnect(topic).await {
                                                    Ok(_) => {
                                                        show_success_toast(toaster, "Disconnected".to_owned());
                                                    }
                                                    Err(e) => {
                                                        show_error_toast(
                                                            toaster,
                                                            format!("Disconnect failed: {e}"),
                                                        );
                                                    }
                                                }
                                            });
                                        }>"Disconnect"</Button>
                                    </Flex>
                                </li>
                            }
                        })
                        .collect::<Vec<_>>()
                }}
            </ul>
            <ul>
                {move || {
                    app_sessions
                        .get()
                        .iter()
                        .map(|session| {
                            let topic = session.topic.clone();
                            view! {
                                <li>
                                    <Flex>
                                        "App session"
                                        <Button on_click=move |_| {
                                            let topic = topic.clone();
                                            leptos::task::spawn_local(async move {
                                                let client = clients.read_value().as_ref().unwrap().clone();
                                                let mut client = client.lock().await;
                                                match client.app_client.disconnect(topic).await {
                                                    Ok(_) => {
                                                        show_success_toast(
                                                            toaster,
                                                            "Disconnected (app)".to_owned(),
                                                        );
                                                    }
                                                    Err(e) => {
                                                        show_error_toast(
                                                            toaster,
                                                            format!("Disconnect failed (app): {e}"),
                                                        );
                                                    }
                                                }
                                            });
                                        }>"Disconnect"</Button>
                                    </Flex>
                                </li>
                            }
                        })
                        .collect::<Vec<_>>()
                }}
            </ul>
        </Flex>
        {move || {
            pairing_request
                .get()
                .map(|request| {
                    view! {
                        <Dialog open=pairing_request_open>
                            <DialogSurface>
                                <DialogBody>
                                    <DialogTitle>"Approve pairing"</DialogTitle>
                                    {move || {
                                        request
                                            .get()
                                            .map(|request| {
                                                // TODO avoid flash here
                                                view! {
                                                    <DialogContent>{format!("{request:?}")}</DialogContent>
                                                    <DialogActions>
                                                        <Button
                                                            loading=approve_pairing_action.pending()
                                                            on_click={
                                                                let request = request.clone();
                                                                move |_| {
                                                                    approve_pairing_action.dispatch(request.clone());
                                                                }
                                                            }
                                                        >
                                                            "Approve"
                                                        </Button>
                                                        <Button
                                                            loading=reject_pairing_action.pending()
                                                            on_click={
                                                                let _request = request.clone();
                                                                move |_| {
                                                                    reject_pairing_action.dispatch(request.clone());
                                                                }
                                                            }
                                                        >
                                                            "Reject"
                                                        </Button>
                                                    </DialogActions>
                                                }
                                                    .into_any()
                                            })
                                            .unwrap_or_else(|| {
                                                view! {
                                                    <DialogContent>
                                                        <Spinner />
                                                    </DialogContent>
                                                }
                                                    .into_any()
                                            })
                                    }}
                                </DialogBody>
                            </DialogSurface>
                        </Dialog>
                    }
                })
        }}
        {move || {
            signature_request
                .get()
                .map(|request| {
                    view! {
                        <Dialog open=signature_request_open>
                            <DialogSurface>
                                <DialogBody>
                                    <DialogTitle>"Signature request"</DialogTitle>
                                    <DialogContent>{format!("{request:?}")}</DialogContent>
                                    <DialogActions>
                                        <Button
                                            loading=session_request_action.pending()
                                            on_click={
                                                let request = request.clone();
                                                move |_| {
                                                    session_request_action.dispatch(request.clone());
                                                }
                                            }
                                        >
                                            "Approve"
                                        </Button>
                                        <Button
                                            loading=session_request_reject_action.pending()
                                            on_click={
                                                let request = request.clone();
                                                move |_| {
                                                    session_request_reject_action.dispatch(request.clone());
                                                }
                                            }
                                        >
                                            "Reject"
                                        </Button>
                                    </DialogActions>
                                </DialogBody>
                            </DialogSurface>
                        </Dialog>
                    }
                })
        }}
        {move || {
            view! {
                <Dialog open=connect_uri.get().is_some()>
                    <DialogSurface>
                        <DialogBody>
                            <DialogTitle>"Connect"</DialogTitle>
                            <DialogContent>
                                {move || {
                                    connect_uri
                                        .get()
                                        .unwrap_or_default()
                                        .map(|uri| {
                                            view! {
                                                <p>{uri.clone()}</p>
                                                <Button on_click=move |_| {
                                                    pair_action.dispatch(uri.clone());
                                                }>"Self connect"</Button>
                                            }
                                                .into_any()
                                        })
                                        .unwrap_or_else(|| view! { <Spinner /> }.into_any())
                                }}
                            </DialogContent>
                        </DialogBody>
                    </DialogSurface>
                </Dialog>
            }
        }}
    }
}
