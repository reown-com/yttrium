#[cfg(feature = "chain_abstraction_client")]
use crate::chain_abstraction::{
    pulse::{PulseMetadata, PULSE_SDK_TYPE},
    send_transaction::RpcRequestAnalytics,
};
use {
    crate::{
        blockchain_api::{PROXY_ENDPOINT_PATH, WALLET_ENDPOINT_PATH},
        wallet_provider::WalletProvider,
    },
    alloy::{
        rpc::{
            client::{ClientBuilder, RpcClient},
            json_rpc::{RequestPacket, ResponsePacket},
        },
        transports::{
            TransportError, TransportErrorKind, TransportFut, TransportResult,
        },
    },
    alloy_provider::{ProviderBuilder, RootProvider},
    relay_rpc::domain::ProjectId,
    reqwest::{Client as ReqwestClient, Url},
    std::{collections::HashMap, task, time::Duration},
    tower::Service,
    tracing::{info, trace},
    uuid::Uuid,
};

/// Creates Blockchain API Reqwest clients for each chain and will return the same provider for subsequent calls
#[derive(Clone)]
pub struct ProviderPool {
    pub client: ReqwestClient,
    pub eip155_providers:
        std::sync::Arc<tokio::sync::RwLock<HashMap<String, RootProvider>>>,
    pub blockchain_api_base_url: Url,
    pub project_id: ProjectId,
    pub rpc_overrides: HashMap<String, Url>,
    pub session_id: Uuid,
    #[cfg(feature = "chain_abstraction_client")]
    pub pulse_metadata: PulseMetadata,
    #[cfg(feature = "sui")]
    pub sui_clients: std::sync::Arc<
        tokio::sync::RwLock<HashMap<String, sui_sdk::SuiClient>>,
    >,
    #[cfg(feature = "ton")]
    pub ton_clients: std::sync::Arc<
        tokio::sync::RwLock<HashMap<String, crate::ton_provider::TonProvider>>,
    >,
}

impl ProviderPool {
    pub fn new(
        project_id: ProjectId,
        client: ReqwestClient,
        #[cfg(feature = "chain_abstraction_client")]
        pulse_metadata: PulseMetadata,
        blockchain_api_base_url: Url,
    ) -> Self {
        let session_id = Uuid::new_v4();
        info!("ProviderPool session_id: {}", session_id);
        Self {
            client,
            eip155_providers: std::sync::Arc::new(tokio::sync::RwLock::new(
                HashMap::new(),
            )),
            blockchain_api_base_url,
            project_id,
            rpc_overrides: HashMap::new(),
            session_id,
            #[cfg(feature = "chain_abstraction_client")]
            pulse_metadata,
            #[cfg(feature = "sui")]
            sui_clients: std::sync::Arc::new(tokio::sync::RwLock::new(
                HashMap::new(),
            )),
            #[cfg(feature = "ton")]
            ton_clients: std::sync::Arc::new(tokio::sync::RwLock::new(
                HashMap::new(),
            )),
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

    pub async fn get_provider(&self, chain_id: &str) -> RootProvider {
        self.get_provider_with_tracing(chain_id, None).await
    }

    pub async fn get_provider_with_tracing(
        &self,
        chain_id: &str,
        #[cfg(feature = "chain_abstraction_client")] tracing: Option<
            std::sync::mpsc::Sender<RpcRequestAnalytics>,
        >,
        #[cfg(not(feature = "chain_abstraction_client"))] _tracing: Option<()>,
    ) -> RootProvider {
        self.get_provider_inner(
            chain_id,
            #[cfg(feature = "chain_abstraction_client")]
            tracing,
            #[cfg(not(feature = "chain_abstraction_client"))]
            None,
            PROXY_ENDPOINT_PATH,
            vec![("chainId", chain_id)],
        )
        .await
    }

    pub async fn get_wallet_provider(
        &self,
        #[cfg(feature = "chain_abstraction_client")] tracing: Option<
            std::sync::mpsc::Sender<RpcRequestAnalytics>,
        >,
        #[cfg(not(feature = "chain_abstraction_client"))] _tracing: Option<()>,
        url_override: Option<Url>,
    ) -> WalletProvider {
        WalletProvider {
            client: self
                .get_rpc_client(
                    #[cfg(feature = "chain_abstraction_client")]
                    tracing,
                    #[cfg(not(feature = "chain_abstraction_client"))]
                    _tracing,
                    WALLET_ENDPOINT_PATH,
                    vec![],
                    url_override,
                    None,
                )
                .await,
        }
    }

    pub async fn get_provider_inner(
        &self,
        chain_id: &str,
        #[cfg(feature = "chain_abstraction_client")] tracing: Option<
            std::sync::mpsc::Sender<RpcRequestAnalytics>,
        >,
        #[cfg(not(feature = "chain_abstraction_client"))] _tracing: Option<()>,
        path: &str,
        additional_query_params: impl IntoIterator<Item = (&str, &str)>,
    ) -> RootProvider {
        let cached_provider =
            self.eip155_providers.read().await.get(chain_id).cloned();
        if let Some(provider) = cached_provider {
            provider
        } else {
            let url_override = self.rpc_overrides.get(chain_id).cloned();

            let provider = ProviderBuilder::new()
                .disable_recommended_fillers()
                .on_client({
                    self.get_rpc_client(
                        #[cfg(feature = "chain_abstraction_client")]
                        tracing,
                        #[cfg(not(feature = "chain_abstraction_client"))]
                        _tracing,
                        path,
                        additional_query_params,
                        url_override,
                        None,
                    )
                    .await
                    .with_poll_interval(polling_interval_for_chain_id(chain_id))
                });

            self.eip155_providers
                .write()
                .await
                .insert(chain_id.to_owned(), provider.clone());

            provider
        }
    }

    pub async fn get_rpc_client(
        &self,
        #[cfg(feature = "chain_abstraction_client")] tracing: Option<
            std::sync::mpsc::Sender<RpcRequestAnalytics>,
        >,
        #[cfg(not(feature = "chain_abstraction_client"))] _tracing: Option<()>,
        path: &str,
        additional_query_params: impl IntoIterator<Item = (&str, &str)>,
        url_override: Option<Url>,
        blockchain_api_base_url_override: Option<&Url>,
    ) -> RpcClient {
        let url = if let Some(rpc_override) = url_override {
            rpc_override
        } else {
            // TODO use universal version: https://linear.app/reown/issue/RES-142/universal-provider-router
            // TODO i.e. checking if chain is supported ahead of time? - but if we support "all" chains then maybe this is a moot point

            let blockchain_api_base_url =
                if let Some(blockchain_api_base_url_override) =
                    blockchain_api_base_url_override
                {
                    blockchain_api_base_url_override
                } else {
                    &self.blockchain_api_base_url
                };

            let mut url = blockchain_api_base_url.join(path).unwrap();
            url.query_pairs_mut()
                .append_pair("projectId", self.project_id.as_ref())
                .append_pair("sessionId", self.session_id.to_string().as_str());

            #[cfg(feature = "chain_abstraction_client")]
            {
                url.query_pairs_mut()
                    .append_pair("st", PULSE_SDK_TYPE)
                    .append_pair(
                        "sv",
                        self.pulse_metadata.sdk_version.as_str(),
                    );
            }

            url.query_pairs_mut().extend_pairs(additional_query_params);
            url
        };

        let transport = CustomClient::new(
            self.client.clone(),
            url,
            #[cfg(feature = "chain_abstraction_client")]
            tracing,
            #[cfg(not(feature = "chain_abstraction_client"))]
            _tracing,
        );
        ClientBuilder::default()
            // .layer(RpcRequestModifyingLayer { tracing })
            .transport(transport, false)
    }

    #[cfg(feature = "sui")]
    pub async fn get_sui_client(
        &self,
        sui_chain_id: String,
    ) -> sui_sdk::SuiClient {
        assert!(sui_chain_id.starts_with("sui:"), "Invalid Sui chain ID");
        let sui_client =
            self.sui_clients.read().await.get(&sui_chain_id).cloned();
        if let Some(sui_client) = sui_client {
            sui_client
        } else {
            let mut url =
                self.blockchain_api_base_url.join(PROXY_ENDPOINT_PATH).unwrap();
            url.query_pairs_mut()
                .append_pair("projectId", self.project_id.as_ref())
                .append_pair("sessionId", self.session_id.to_string().as_str());

            #[cfg(feature = "chain_abstraction_client")]
            {
                url.query_pairs_mut()
                    .append_pair("st", PULSE_SDK_TYPE)
                    .append_pair(
                        "sv",
                        self.pulse_metadata.sdk_version.as_str(),
                    );
            }

            url.query_pairs_mut().append_pair("chainId", sui_chain_id.as_str());
            let sui_client =
                sui_sdk::SuiClientBuilder::default().build(url).await.unwrap();
            self.sui_clients
                .write()
                .await
                .insert(sui_chain_id, sui_client.clone());
            sui_client
        }
    }

    #[cfg(feature = "stacks")]
    pub async fn get_stacks_client(
        &self,
        network: &str,
        tracing: Option<std::sync::mpsc::Sender<RpcRequestAnalytics>>,
        url_override: Option<Url>,
    ) -> crate::stacks_provider::StacksProvider {
        crate::stacks_provider::StacksProvider {
            client: self
                .get_rpc_client(
                    tracing,
                    PROXY_ENDPOINT_PATH,
                    vec![("chainId", network)],
                    url_override,
                    None,
                )
                .await,
        }
    }

    #[cfg(feature = "ton")]
    pub async fn get_ton_client(
        &self,
        network: &str,
        tracing: Option<std::sync::mpsc::Sender<RpcRequestAnalytics>>,
        url_override: Option<Url>,
    ) -> crate::ton_provider::TonProvider {
        let ton_client = self.ton_clients.read().await.get(network).cloned();
        if let Some(ton_client) = ton_client {
            ton_client
        } else {
            // Create RpcClient using the same pattern as Stacks
            let rpc_client = self
                .get_rpc_client(
                    #[cfg(feature = "chain_abstraction_client")]
                    tracing,
                    #[cfg(not(feature = "chain_abstraction_client"))]
                    None,
                    crate::blockchain_api::PROXY_ENDPOINT_PATH,
                    vec![("chainId", network)],
                    url_override,
                    None,
                )
                .await;

            let ton_client = crate::ton_provider::TonProvider::new(rpc_client);

            self.ton_clients
                .write()
                .await
                .insert(network.to_owned(), ton_client.clone());
            ton_client
        }
    }
}

fn polling_interval_for_chain_id(chain_id: &str) -> Duration {
    const ONE: Duration = Duration::from_secs(1);
    match chain_id {
        network::eip155::BASE
        | network::eip155::OPTIMISM
        | network::eip155::ARBITRUM => ONE,
        _ => Duration::from_secs(7), // alloy's current default
    }
}

pub mod network {
    pub mod eip155 {
        pub const BASE: &str = "eip155:8453";
        pub const OPTIMISM: &str = "eip155:10";
        pub const ARBITRUM: &str = "eip155:42161";
    }

    pub mod sui {
        pub const MAINNET: &str = "sui:mainnet";
        pub const DEVNET: &str = "sui:devnet";
        pub const TESTNET: &str = "sui:testnet";
    }

    pub mod stacks {
        pub const MAINNET: &str = "stacks:1";
        pub const TESTNET: &str = "stacks:2147483648";
    }

    pub mod ton {
        pub const MAINNET: &str = "ton:-239";
        pub const TESTNET: &str = "ton:-3";
    }
}

#[cfg(feature = "chain_abstraction_client")]
type TracingType = Option<std::sync::mpsc::Sender<RpcRequestAnalytics>>;

#[cfg(not(feature = "chain_abstraction_client"))]
type TracingType = Option<()>;

/// Custom client that enables adding things like tracing to the requests.
#[derive(Clone)]
pub struct CustomClient {
    client: ReqwestClient,
    url: Url,
    tracing: TracingType,
}

impl CustomClient {
    pub fn new(client: ReqwestClient, url: Url, tracing: TracingType) -> Self {
        Self { client, url, tracing }
    }
}

impl Service<RequestPacket> for CustomClient {
    type Response = ResponsePacket;
    type Error = TransportError;
    type Future = TransportFut<'static>;

    #[inline]
    fn poll_ready(
        &mut self,
        _cx: &mut task::Context<'_>,
    ) -> task::Poll<Result<(), Self::Error>> {
        // `reqwest` always returns `Ok(())`.
        task::Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, req: RequestPacket) -> Self::Future {
        Box::pin(self.clone().do_reqwest(req, self.tracing.clone()))
    }
}

impl CustomClient {
    // Copied from alloy `Http<RequestClient>` impl
    #[tracing::instrument(skip_all)]
    async fn do_reqwest(
        self,
        req: RequestPacket,
        _tracing: TracingType,
    ) -> TransportResult<ResponsePacket> {
        tracing::debug!("rpc POST url={} body={}", self.url, serde_json::to_string(&req).unwrap());
        
        let resp = self
            .client
            .post(self.url)
            .header("X-Ton-Client-Version", "15.3.1")
            .json(&req)
            .send()
            .await
            .map_err(TransportErrorKind::custom)?;
        let status = resp.status();
        tracing::debug!("res_status: {}", status);

        let req_id = resp
            .headers()
            .get("x-request-id")
            .and_then(|hv| hv.to_str().ok())
            .map(|s| s.to_string());
        trace!("req_id: {}", req_id.as_deref().unwrap_or("none"));

        #[cfg(feature = "chain_abstraction_client")]
        if let Some(tracing) = _tracing {
            let rpcs = match req {
                RequestPacket::Single(req) => {
                    vec![(req.id().clone(), req.method().to_owned())]
                }
                RequestPacket::Batch(reqs) => reqs
                    .iter()
                    .map(|req| (req.id().clone(), req.method().to_owned()))
                    .collect(),
            };
            for (rpc_id, rpc_method) in rpcs {
                if let Err(e) = tracing.send(RpcRequestAnalytics {
                    req_id: req_id.clone(),
                    rpc_id: rpc_id.to_string(),
                }) {
                    tracing::warn!(
                        "ProxyReqwestClient: send: {e} {rpc_method}"
                    );
                }
            }
        }

        // Unpack data from the response body. We do this regardless of
        // the status code, as we want to return the error in the body
        // if there is one.
        let body = resp.bytes().await.map_err(TransportErrorKind::custom)?;
        tracing::debug!("res_body: {}", String::from_utf8_lossy(&body));

        if !status.is_success() {
            return Err(TransportErrorKind::http_error(
                status.as_u16(),
                String::from_utf8_lossy(&body).into_owned(),
            ));
        }

        // Deserialize a Box<RawValue> from the body. If deserialization fails, return
        // the body as a string in the error. The conversion to String
        // is lossy and may not cover all the bytes in the body.
        serde_json::from_slice(&body).map_err(|err| {
            TransportError::deser_err(err, String::from_utf8_lossy(&body))
        })
    }
}
