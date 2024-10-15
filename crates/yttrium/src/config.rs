use dotenvy::dotenv;
use std::env;

const LOCAL_RPC_URL: &str = "http://localhost:8545";
const LOCAL_BUNDLER_URL: &str = "http://localhost:4337";
const LOCAL_PAYMASTER_URL: &str = "http://localhost:3000";

#[derive(Clone, Debug, PartialEq)]
#[cfg(feature = "uniffi")]
#[derive(uniffi_macros:: Record)]
pub struct Config {
    pub endpoints: Endpoints,
}

impl Config {
    pub fn local() -> Self {
        Config { endpoints: Endpoints::local() }
    }

    pub fn pimlico() -> Self {
        Config { endpoints: Endpoints::pimlico() }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg(feature = "uniffi")]
#[derive(uniffi_macros:: Record)]
pub struct Endpoints {
    pub rpc: Endpoint,
    pub bundler: Endpoint,
    pub paymaster: Endpoint,
}

impl Endpoints {
    pub fn live() -> Self {
        dotenv().unwrap();

        let rpc = {
            let api_key = env::var("RPC_API_KEY")
                .expect("You've not set the RPC_API_KEY");
            let base_url = env::var("RPC_BASE_URL")
                .expect("You've not set the RPC_BASE_URL");
            Endpoint { api_key, base_url }
        };

        let bundler = {
            let api_key = env::var("BUNDLER_API_KEY")
                .expect("You've not set the BUNDLER_API_KEY");
            let base_url = env::var("BUNDLER_BASE_URL")
                .expect("You've not set the BUNDLER_BASE_URL");
            Endpoint { api_key, base_url }
        };

        Endpoints { rpc, paymaster: bundler.clone(), bundler }
    }

    pub fn local() -> Self {
        Endpoints {
            rpc: Endpoint::local_rpc(),
            bundler: Endpoint::local_bundler(),
            paymaster: Endpoint::local_paymaster(),
        }
    }

    pub fn pimlico() -> Self {
        dotenv().unwrap();

        let api_key = env::var("PIMLICO_API_KEY")
            .expect("You've not set the PIMLICO_API_KEY");

        let rpc = {
            let base_url = env::var("PIMLICO_RPC_URL")
                .expect("You've not set the PIMLICO_RPC_URL");
            Endpoint { api_key: api_key.clone(), base_url }
        };

        let bundler = {
            let base_url = env::var("PIMLICO_BUNDLER_URL")
                .expect("You've not set the PIMLICO_BUNDLER_URL");
            Endpoint { api_key: api_key.clone(), base_url }
        };

        Endpoints { rpc, paymaster: bundler.clone(), bundler }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg(feature = "uniffi")]
#[derive(uniffi_macros:: Record)]
pub struct Endpoint {
    pub base_url: String,
    pub api_key: String,
}

impl Endpoint {
    pub fn local_rpc() -> Self {
        Endpoint {
            base_url: LOCAL_RPC_URL.to_string(),
            api_key: "".to_string(),
        }
    }

    pub fn local_bundler() -> Self {
        Endpoint {
            base_url: LOCAL_BUNDLER_URL.to_string(),
            api_key: "".to_string(),
        }
    }

    pub fn local_paymaster() -> Self {
        Endpoint {
            base_url: LOCAL_PAYMASTER_URL.to_string(),
            api_key: "".to_string(),
        }
    }
}
