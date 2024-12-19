use {
    crate::{
        blockchain_api::{BLOCKCHAIN_API_URL, PROXY_ENDPOINT_PATH},
        config::Config,
    },
    alloy::{
        network::Ethereum, rpc::client::RpcClient, transports::http::Http,
    },
    alloy_provider::ReqwestProvider,
    relay_rpc::domain::ProjectId,
    reqwest::{Client as ReqwestClient, Url},
    std::{collections::HashMap, sync::Arc},
    tokio::sync::RwLock,
};

/// Creates Blockchain API Reqwest clients for each chain and will return the same provider for subsequent calls
#[derive(Clone)]
pub struct ProviderPool {
    pub client: ReqwestClient,
    pub providers: Arc<RwLock<HashMap<String, ReqwestProvider>>>,
    pub blockchain_api_base_url: Url,
    pub project_id: ProjectId,
    pub config: Config,
}

impl ProviderPool {
    // TODO remove `config` param
    pub fn new(project_id: ProjectId, config: Config) -> Self {
        Self {
            client: ReqwestClient::new(),
            providers: Arc::new(RwLock::new(HashMap::new())),
            blockchain_api_base_url: BLOCKCHAIN_API_URL.parse().unwrap(),
            project_id,
            config,
        }
    }

    pub async fn get_provider(&self, chain_id: &str) -> ReqwestProvider {
        let providers = self.providers.read().await;
        if let Some(provider) = providers.get(chain_id) {
            provider.clone()
        } else {
            std::mem::drop(providers);

            // TODO use universal version: https://linear.app/reown/issue/RES-142/universal-provider-router
            let mut url =
                self.blockchain_api_base_url.join(PROXY_ENDPOINT_PATH).unwrap();
            url.query_pairs_mut()
                .append_pair("chainId", chain_id)
                .append_pair("projectId", self.project_id.as_ref());
            let provider = ReqwestProvider::<Ethereum>::new(RpcClient::new(
                Http::with_client(self.client.clone(), url),
                false,
            ));
            self.providers
                .write()
                .await
                .insert(chain_id.to_owned(), provider.clone());
            provider
        }
    }
}
