use leptos::{prelude::*, server::codee::string::JsonSerdeCodec};
use leptos_use::storage::use_local_storage;
use serde::{Deserialize, Serialize};
use yttrium::sign::{ApprovedSession, Client};

const RELAY_URL: &str = "wss://relay.walletconnect.org";
const CLIENT_ID: &str = "123";

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

        view! {
            <label for="pairing-uri">Pairing URI</label>
            <input type="text" id="pairing-uri" prop:value=pairing_uri on:input:target=move |ev| {
                pairing_uri.set(ev.target().value());
            } />
            <button on:click=move |_| {
                let uri = pairing_uri.get();
                let mut client = Client::new(
                    RELAY_URL.to_owned(),
                    include_str!("../.project-id").trim().into(),
                    CLIENT_ID.to_owned().into(),
                );
                leptos::task::spawn_local(async move {
                    match client.pair(&uri).await {
                        Ok(pairing) => {
                            match client.approve(pairing).await {
                                Ok(approved_session) => {
                                    set_state.update(|state| {
                                        state.sessions.push(approved_session);
                                    });
                                    pairing_status.set("Pairing approved".to_owned());
                                }
                                Err(e) => {
                                    pairing_status.set(format!("Approval failed: {e}"));
                                }
                            }
                        }
                        Err(e) => {
                            pairing_status.set(format!("Pairing failed: {e}"));
                        }
                    }
                    pairing_uri.set(String::new());
                });
            }>Pair</button>
            <p>"Pairing status: "{pairing_status}</p>
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
