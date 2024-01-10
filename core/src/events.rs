use crate::imports::*;
use crate::market::*;
use crate::utils::Release;
use kaspa_metrics_core::MetricsSnapshot;
use kaspa_wallet_core::{events as kaspa, storage::PrvKeyDataInfo};

pub type ApplicationEventsChannel = crate::runtime::channel::Channel<Events>;

#[derive(Clone)]
pub enum Events {
    VisibilityChange(VisibilityState),
    VersionUpdate(Release),
    ThemeChange,
    StoreSettings,
    UpdateLogs,
    Market(MarketUpdate),
    Metrics {
        snapshot: Box<MetricsSnapshot>,
    },
    MempoolSize {
        mempool_size: usize,
    },
    Error(Box<String>),
    ServerList {
        server_list: Arc<Vec<Server>>,
    },
    WalletList {
        wallet_list: Arc<Vec<WalletDescriptor>>,
    },
    Wallet {
        event: Box<kaspa::Events>,
    },
    PrvKeyDataInfo {
        prv_key_data_info_map: HashMap<PrvKeyDataId, Arc<PrvKeyDataInfo>>,
    },
    UnlockSuccess,
    UnlockFailure {
        message: String,
    },
    Notify {
        user_notification: UserNotification,
    },
    Close,
    Exit,
}
