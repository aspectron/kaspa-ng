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
    // AccountList {
    //     // account_list: Arc<Vec<Arc<dyn runtime::Account>>>,
    //     account_list: Box<Vec<Account>>,
    // },
    Wallet {
        event: Box<kaspa::Events>,
    },
    // TryUnlock(Secret),
    UnlockSuccess,
    UnlockFailure {
        message: String,
    },
    Notify {
        notification: Notification,
    },
    Close,
    // Send,
    // Deposit,
    // Overview,
    // Transactions,
    // Accounts,
    // Settings,
    Exit,
}
