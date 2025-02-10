use {
    crate::{blockchain_api::{BLOCKCHAIN_API_URL, PROXY_ENDPOINT_PATH}, chain_abstraction::pulse::{PulseMetadata, PULSE_SDK_TYPE}},
    alloy::{
        network::Ethereum, rpc::client::RpcClient, transports::http::Http,
    },
    alloy_provider::ReqwestProvider,
    relay_rpc::domain::ProjectId,
    reqwest::{Client as ReqwestClient, Url},
    std::{collections::HashMap, sync::Arc, time::Duration},
    tokio::sync::RwLock,
    uuid::Uuid,
};

/// Creates Blockchain API Reqwest clients for each chain and will return the same provider for subsequent calls
#[derive(Clone)]
pub struct ProviderPool {
    pub client: ReqwestClient,
    pub providers: Arc<RwLock<HashMap<String, ReqwestProvider>>>,
    pub blockchain_api_base_url: Url,
    pub project_id: ProjectId,
    pub rpc_overrides: HashMap<String, Url>,
    pub session_id: Uuid,
    pub pulse_metadata: PulseMetadata,
}

impl ProviderPool {
    pub fn new(project_id: ProjectId, client: ReqwestClient, pulse_metadata: PulseMetadata) -> Self {
        Self {
            client,
            providers: Arc::new(RwLock::new(HashMap::new())),
            blockchain_api_base_url: BLOCKCHAIN_API_URL.parse().unwrap(),
            project_id,
            rpc_overrides: HashMap::new(),
            session_id: Uuid::new_v4(),
            pulse_metadata,
        }
    }

    // TODO: Ultimate design: provide 2 callbacks: key and value
    // First callback is called with a variety of things: RPC payload, UserOperation, etc. Function returns a key. E.g. the RPC payload chain_id or the UserOperation chain_id
    // Second is called if the key is not found in the cache. E.g. Blockchain API ReqwestProvider
    pub fn with_rpc_overrides(
        mut self,
        rpc_overrides: HashMap<String, Url>,
    ) -> Self {
        self.rpc_overrides = rpc_overrides;
        self
    }

    pub fn set_rpc_overrides(&mut self, rpc_overrides: HashMap<String, Url>) {
        self.rpc_overrides = rpc_overrides;
    }

    pub async fn get_provider(&self, chain_id: &str) -> ReqwestProvider {
        let providers = self.providers.read().await;
        if let Some(provider) = providers.get(chain_id) {
            provider.clone()
        } else {
            std::mem::drop(providers);

            let url = if let Some(rpc_override) =
                self.rpc_overrides.get(chain_id).cloned()
            {
                rpc_override
            } else {
                // TODO use universal version: https://linear.app/reown/issue/RES-142/universal-provider-router
                // TODO i.e. checking if chain is supported ahead of time? - but if we support "all" chains then maybe this is a moot point
                let mut url = self
                    .blockchain_api_base_url
                    .join(PROXY_ENDPOINT_PATH)
                    .unwrap();
                url.query_pairs_mut()
                    .append_pair("chainId", chain_id)
                    .append_pair("projectId", self.project_id.as_ref())
                    .append_pair("sessionId", self.session_id.to_string().as_str())
                    .append_pair("st", PULSE_SDK_TYPE)
                    .append_pair("sv", self.pulse_metadata.sdk_version.as_str());
                url
            };

            let provider = ReqwestProvider::<Ethereum>::new(
                RpcClient::new(
                    Http::with_client(self.client.clone(), url),
                    false,
                )
                .with_poll_interval(polling_interval_for_chain_id(chain_id)),
            );
            self.providers
                .write()
                .await
                .insert(chain_id.to_owned(), provider.clone());
            provider
        }
    }
}

fn polling_interval_for_chain_id(chain_id: &str) -> Duration {
    const ONE: Duration = Duration::from_secs(1);
    match chain_id {
        network::BASE | network::OPTIMISM | network::ARBITRUM => ONE,
        _ => Duration::from_secs(7), // alloy's current default
    }
}

mod network {
    pub const BASE: &str = "eip155:8453";
    pub const OPTIMISM: &str = "eip155:10";
    pub const ARBITRUM: &str = "eip155:42161";
}
