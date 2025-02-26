use {
    crate::{
        blockchain_api::PROXY_ENDPOINT_PATH,
        chain_abstraction::{
            pulse::{PulseMetadata, PULSE_SDK_TYPE},
            send_transaction::RpcRequestAnalytics,
        },
    },
    alloy::{
        rpc::{
            client::ClientBuilder,
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
    tracing::warn,
    uuid::Uuid,
};

/// Creates Blockchain API Reqwest clients for each chain and will return the same provider for subsequent calls
#[derive(Clone)]
pub struct ProviderPool {
    pub client: ReqwestClient,
    // pub providers: Arc<RwLock<HashMap<String, RootProvider>>>,
    pub blockchain_api_base_url: Url,
    pub project_id: ProjectId,
    pub rpc_overrides: HashMap<String, Url>,
    pub session_id: Uuid,
    pub pulse_metadata: PulseMetadata,
}

impl ProviderPool {
    pub fn new(
        project_id: ProjectId,
        client: ReqwestClient,
        pulse_metadata: PulseMetadata,
        blockchain_api_base_url: Url,
    ) -> Self {
        Self {
            client,
            // providers: Arc::new(RwLock::new(HashMap::new())),
            blockchain_api_base_url,
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

    pub async fn get_provider(&self, chain_id: &str) -> RootProvider {
        self.get_provider_with_tracing(chain_id, None).await
    }

    pub async fn get_provider_with_tracing(
        &self,
        chain_id: &str,
        tracing: Option<std::sync::mpsc::Sender<RpcRequestAnalytics>>,
    ) -> RootProvider {
        // let providers = self.providers.read().await;
        // let cached_provider = providers.get(chain_id);
        let cached_provider = None as Option<&RootProvider>;
        if let Some(provider) = cached_provider {
            provider.clone()
        } else {
            // std::mem::drop(providers);

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
                    .append_pair(
                        "sessionId",
                        self.session_id.to_string().as_str(),
                    )
                    .append_pair("st", PULSE_SDK_TYPE)
                    .append_pair(
                        "sv",
                        self.pulse_metadata.sdk_version.as_str(),
                    );
                url
            };

            ProviderBuilder::new().disable_recommended_fillers().on_client({
                let transport =
                    ProxyReqwestClient::new(self.client.clone(), url, tracing);
                ClientBuilder::default()
                    // .layer(RpcRequestModifyingLayer { tracing })
                    .transport(transport, false)
                    .with_poll_interval(polling_interval_for_chain_id(chain_id))
            })

            // self.providers
            //     .write()
            //     .await
            //     .insert(chain_id.to_owned(), provider.clone());

            // provider
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

type TracingType = Option<std::sync::mpsc::Sender<RpcRequestAnalytics>>;

#[derive(Clone)]
pub struct ProxyReqwestClient {
    client: ReqwestClient,
    url: Url,
    tracing: TracingType,
}

impl ProxyReqwestClient {
    pub fn new(client: ReqwestClient, url: Url, tracing: TracingType) -> Self {
        Self { client, url, tracing }
    }
}

impl Service<RequestPacket> for ProxyReqwestClient {
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

impl ProxyReqwestClient {
    // Copied from alloy `Http<RequestClient>` impl
    async fn do_reqwest(
        self,
        req: RequestPacket,
        tracing: TracingType,
    ) -> TransportResult<ResponsePacket> {
        let resp = self
            .client
            .post(self.url)
            .json(&req)
            .send()
            .await
            .map_err(TransportErrorKind::custom)?;
        let status = resp.status();

        let req_id = resp
            .headers()
            .get("x-request-id")
            .and_then(|hv| hv.to_str().ok())
            .map(|s| s.to_string());
        if let Some(tracing) = tracing {
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
                    warn!("ProxyReqwestClient: send: {e} {rpc_method}");
                }
            }
        }

        // Unpack data from the response body. We do this regardless of
        // the status code, as we want to return the error in the body
        // if there is one.
        let body = resp.bytes().await.map_err(TransportErrorKind::custom)?;

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
