use {
    super::client::ExecuteAnalytics,
    crate::serde::systemtime_millis,
    relay_rpc::domain::ProjectId,
    reqwest::{Client, Url},
    serde::{Deserialize, Serialize},
    tracing::{debug, warn},
    uuid::Uuid,
    web_time::SystemTime,
};

// const PULSE_ENDPOINT: &str = "https://analytics-api-cf-workers-staging.walletconnect-v1-bridge.workers.dev/e";
const PULSE_ENDPOINT: &str = "https://pulse.walletconnect.org/e";

pub fn pulse(
    http_client: Client,
    props: ExecuteAnalytics,
    project_id: ProjectId,
    pulse_metadata: &PulseMetadata,
) {
    let event_id = Uuid::new_v4();
    let timestamp = SystemTime::now();
    let analytics = Event {
        event_id,
        url: pulse_metadata.url.as_ref().map(|url| url.to_string()),
        domain: pulse_metadata
            .url
            .as_ref()
            .map(|url| url.origin().ascii_serialization()),
        bundle_id: pulse_metadata.bundle_id.clone(),
        timestamp,
        props,
    };
    debug!("pulse analytics: {analytics:?}");

    let query = Query {
        project_id,
        sdk_type: PULSE_SDK_TYPE,
        sdk_version: pulse_metadata.sdk_version.clone(),
        sdk_platform: pulse_metadata.sdk_platform.clone(),
    };

    // println!(
    //     "url: {}",
    //     http_client.post(PULSE_ENDPOINT).query(&query).build().unwrap().url()
    // );

    let mut builder =
        http_client.post(PULSE_ENDPOINT).query(&query).json(&analytics);

    #[cfg(not(target_arch = "wasm32"))]
    {
        builder =
            builder.header("User-Agent", pulse_metadata.sdk_version.clone());
    }

    let fut = builder.send();
    let fut = async move {
        let result = fut.await;
        match result {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    debug!("successfully sent execute() analytics");
                } else {
                    match response.text().await {
                        Ok(text) => {
                            warn!("execute() analytics request failed: {status} {text}");
                        }
                        Err(e) => {
                            warn!("execute() analytics request failed: {e}");
                        }
                    }
                }
            }
            Err(e) => {
                warn!("execute() analytics request failed: {e}");
            }
        }
    };

    #[cfg(not(test))]
    super::spawn::spawn(fut);

    #[cfg(test)]
    std::thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(fut)
    })
    .join()
    .unwrap();
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub event_id: Uuid,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub bundle_id: Option<String>,
    #[serde(with = "systemtime_millis")]
    pub timestamp: SystemTime,
    pub props: ExecuteAnalytics,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Query {
    pub project_id: ProjectId,
    #[serde(rename = "st")]
    pub sdk_type: &'static str,
    #[serde(rename = "sv")]
    pub sdk_version: String,
    #[serde(rename = "sp")]
    pub sdk_platform: String,
}

pub const PULSE_SDK_TYPE: &str = "wkca";

#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[cfg_attr(
    feature = "wasm",
    derive(tsify_next::Tsify),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct PulseMetadata {
    // web
    pub url: Option<Url>,
    // iOS
    pub bundle_id: Option<String>,
    // Android
    // FIXME this param is not used yet
    pub package_name: Option<String>,
    pub sdk_version: String,
    pub sdk_platform: String,
}
