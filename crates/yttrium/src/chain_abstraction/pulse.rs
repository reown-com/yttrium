use {
    super::client::ExecuteAnalytics, crate::pulse::PulseMetadata,
    relay_rpc::domain::ProjectId, reqwest::Client,
};

pub const PULSE_SDK_TYPE: &str = "wkca";

pub fn pulse(
    http_client: Client,
    props: ExecuteAnalytics,
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
