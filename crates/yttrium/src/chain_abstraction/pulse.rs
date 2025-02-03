use {
    super::{client::ExecuteAnalytics, spawn::spawn},
    relay_rpc::domain::ProjectId,
    reqwest::Client,
    serde::{Deserialize, Serialize},
    std::time::Duration,
    tracing::{info, warn},
    uuid::Uuid,
};

const PULSE_ENDPOINT: &str = "https://analytics-api-cf-workers-staging.walletconnect-v1-bridge.workers.dev/e";
// const PULSE_ENDPOINT: &str = "https://pulse.walletconnect.org/e";

pub fn pulse(
    http_client: Client,
    props: ExecuteAnalytics,
    project_id: ProjectId,
) {
    let event_id = Uuid::new_v4();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let analytics = Event { event_id, timestamp, props };
    info!("pulse analytics: {analytics:?}");

    let query = Query {
        project_id,
        sdk_type: SDK_TYPE,
        sdk_version: SDK_VERSION,
        sdk_platform: SDK_PLATFORM,
    };

    let fut =
        http_client.post(PULSE_ENDPOINT).query(&query).json(&analytics).send();

    spawn(async move {
        match fut.await {
            Ok(response) => {
                if response.status().is_success() {
                    info!("successfully sent execute() analytics");
                } else {
                    warn!("execute() analytics request failed: {response:?}");
                }
            }
            Err(e) => {
                warn!("execute() analytics request failed: {e}");
            }
        }
    });
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub event_id: Uuid,
    #[serde(with = "crate::chain_abstraction::client::duration_millis")]
    pub timestamp: Duration,
    pub props: ExecuteAnalytics,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Query {
    pub project_id: ProjectId,
    #[serde(rename = "st")]
    pub sdk_type: &'static str,
    #[serde(rename = "sv")]
    pub sdk_version: &'static str,
    #[serde(rename = "sp")]
    pub sdk_platform: &'static str,
}

const SDK_TYPE: &str = "ca";
const SDK_VERSION: &str = "0.0.0"; // get WK version here walletkit-swift-1.1.5 (current events sdk version)
const SDK_PLATFORM: &str = "mobile";
