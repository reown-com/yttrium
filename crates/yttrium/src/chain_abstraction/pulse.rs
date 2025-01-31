use {
    super::{client::ExecuteAnalytics, spawn::spawn},
    relay_rpc::domain::ProjectId,
    reqwest::Client,
    serde::{Deserialize, Serialize},
    tracing::{info, warn},
};

const PULSE_ENDPOINT: &str = "http://localhost:8080/analytics";
// const PULSE_ENDPOINT: &str = "https://analytics-api-cf-workers-staging.walletconnect-v1-bridge.workers.dev/e";
// const PULSE_ENDPOINT: &str = "https://pulse.walletconnect.org/e";

pub fn pulse(
    http_client: Client,
    execute_analytics: ExecuteAnalytics,
    project_id: ProjectId,
) {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let analytics = Event { event_id: "".to_owned(), timestamp };
    info!("pulse analytics: {analytics:?}");
    spawn(async move {
        match http_client
            .post(PULSE_ENDPOINT)
            .query(&Query {
                project_id,
                sdk_type: SDK_TYPE,
                sdk_version: SDK_VERSION,
                sdk_platform: SDK_PLATFORM,
            })
            .json(&execute_analytics)
            .send()
            .await
        {
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
pub struct Event {
    pub event_id: String,
    pub timestamp: u128,
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

const SDK_TYPE: &str = "events_sdk";
const SDK_VERSION: &str = "0.0.0";
const SDK_PLATFORM: &str = "rust";
