use relay_rpc::domain::ProjectId;

pub struct WalletConnectPay {
    _project_id: ProjectId,
}

impl WalletConnectPay {
    pub fn new(project_id: ProjectId) -> Self {
        Self { _project_id: project_id }
    }
}
