use crate::imports::*;
use kaspa_metrics::MetricsSnapshot;
use kaspa_wallet_core::events as kaspa;

pub type ApplicationEventsChannel = crate::channel::Channel<Events>;

// impl Notify {
// }

#[derive(Clone)]
pub enum Events {
    UpdateLogs,
    Metrics {
        snapshot: Box<MetricsSnapshot>,
    },
    Error(Box<String>),
    WalletList {
        wallet_list: Arc<Vec<WalletDescriptor>>,
    },
    Wallet {
        event: Box<kaspa::Events>,
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
