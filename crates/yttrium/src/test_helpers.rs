fn format_foundry_dir(path: &str) -> String {
    format!(
        "{}/../../../../.foundry/{}",
        std::env::var("OUT_DIR").unwrap(),
        path
    )
}

pub fn spawn_anvil() -> (AnvilInstance, String, ReqwestProvider, SigningKey) {
    let anvil = Anvil::at(format_foundry_dir("bin/anvil")).spawn();
    let rpc_url = anvil.endpoint();
    let provider = ReqwestProvider::<Ethereum>::new_http(anvil.endpoint_url());
    let private_key = anvil.keys().first().unwrap().clone();
    (
        anvil,
        rpc_url,
        provider,
        SigningKey::from_bytes(&private_key.to_bytes()).unwrap(),
    )
}
