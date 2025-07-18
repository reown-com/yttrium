use leptos::prelude::*;
use yttrium::sign::Client;

const RELAY_URL: &str = "wss://relay.walletconnect.org";
const CLIENT_ID: &str = "123";

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| {
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
                    env!("REOWN_PROJECT_ID").into(),
                    CLIENT_ID.to_owned().into(),
                );
                leptos::task::spawn_local(async move {
                    match client.pair(&uri).await {
                        Ok(pairing) => {
                            match client.approve(pairing).await {
                                Ok(()) => {
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
        }
    })
}
