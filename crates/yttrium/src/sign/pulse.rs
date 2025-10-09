use {
    crate::pulse::PulseMetadata, relay_rpc::domain::ProjectId, reqwest::Client,
    serde::Serialize,
};

pub const PULSE_SDK_TYPE: &str = "wks";

pub fn pulse(
    http_client: Client,
    props: SignAnalytics,
    project_id: ProjectId,
    pulse_metadata: &PulseMetadata,
) {
    crate::pulse::pulse(
        http_client,
        props,
        project_id,
        PULSE_SDK_TYPE,
        pulse_metadata,
    );
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignAnalytics {
    // pub orchestration_id: String,
    // pub error: Option<String>,
    // #[serde(with = "systemtime_millis")]
    // pub start: SystemTime,
    // #[serde(with = "duration_millis")]
    // pub route_latency: Duration,
    // pub route: Vec<TransactionAnalytics>,
    // #[serde(with = "option_duration_millis")]
    // pub status_latency: Option<Duration>,
    // pub initial_txn: Option<TransactionAnalytics>,
    // #[serde(with = "duration_millis")]
    // pub latency: Duration,
    // #[serde(with = "systemtime_millis")]
    // pub end: SystemTime,
}
