use {
    crate::{serde::systemtime_millis, time::SystemTime},
    relay_rpc::domain::ProjectId,
    reqwest::{Client, Url},
    serde::{Deserialize, Serialize},
    uuid::Uuid,
};

// const PULSE_ENDPOINT: &str = "https://analytics-api-cf-workers-staging.walletconnect-v1-bridge.workers.dev/e";
const PULSE_ENDPOINT: &str = "https://pulse.walletconnect.org/e";

pub fn pulse(
    http_client: Client,
    props: impl Serialize + std::fmt::Debug,
    project_id: ProjectId,
    sdk_type: &'static str,
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
    tracing::debug!("pulse analytics: {analytics:?}");

    let query = Query {
        project_id,
        sdk_type,
        sdk_version: pulse_metadata.sdk_version.clone(),
        sdk_platform: pulse_metadata.sdk_platform.clone(),
    };

    // println!(
    //     "url: {}",
    //     http_client.post(PULSE_ENDPOINT).query(&query).build().unwrap().url()
    // );

    let builder =
        http_client.post(PULSE_ENDPOINT).query(&query).json(&analytics);

    #[cfg(not(target_arch = "wasm32"))]
    let builder =
        builder.header("User-Agent", pulse_metadata.sdk_version.clone());

    let fut = builder.send();
    let fut = async move {
        let result = fut.await;
        match result {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    tracing::debug!("successfully sent execute() analytics");
                } else {
                    match response.text().await {
                        Ok(text) => {
                            tracing::warn!("execute() analytics request failed: {status} {text}");
                        }
                        Err(e) => {
                            tracing::warn!(
                                "execute() analytics request failed: {e}"
                            );
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("execute() analytics request failed: {e}");
            }
        }
    };

    #[cfg(not(test))]
    crate::spawn::spawn(fut);

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
pub struct Event<Props: Serialize> {
    pub event_id: Uuid,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub bundle_id: Option<String>,
    #[serde(with = "systemtime_millis")]
    pub timestamp: SystemTime,
    pub props: Props,
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
    // iOS & Android
    pub bundle_id: Option<String>,
    pub sdk_version: String,
    pub sdk_platform: String,
}

#[cfg(test)]
pub fn get_pulse_metadata() -> PulseMetadata {
    PulseMetadata {
        url: None,
        bundle_id: Some("com.reown.yttrium.tests".to_owned()),
        sdk_version: "yttrium-tests-0.0.0".to_owned(),
        sdk_platform: "mobile".to_owned(),
    }
}
