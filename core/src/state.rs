use kaspa_consensus_core::network::NetworkId;
use kaspa_wallet_core::events::SyncState;

#[derive(Default)]
pub struct State {
    pub is_open: bool,
    pub is_connected: bool,
    pub is_synced: Option<bool>,
    pub sync_state: Option<SyncState>,
    pub server_version: Option<String>,
    pub url: Option<String>,
    pub network_id: Option<NetworkId>,
    pub current_daa_score: Option<u64>,
    pub network_load: Option<f32>,
}

impl State {
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    pub fn is_synced(&self) -> bool {
        self.is_synced.unwrap_or(false) || matches!(self.sync_state, Some(SyncState::Synced))
    }

    pub fn sync_state(&self) -> &Option<SyncState> {
        &self.sync_state
    }

    pub fn server_version(&self) -> &Option<String> {
        &self.server_version
    }

    pub fn url(&self) -> &Option<String> {
        &self.url
    }

    pub fn network_id(&self) -> &Option<NetworkId> {
        &self.network_id
    }

    pub fn current_daa_score(&self) -> Option<u64> {
        self.current_daa_score
    }
}
