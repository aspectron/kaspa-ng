use crate::imports::*;
use kaspa_wallet_core::events as kaspa;

// impl Notify {
// }

#[derive(Clone)]
pub enum Events {
    Error(Box<String>),
    WalletList {
        wallet_list: Arc<Vec<WalletDescriptor>>,
    },
    AccountList {
        // account_list: Arc<Vec<Arc<dyn runtime::Account>>>,
        account_list: Arc<Vec<Account>>,
    },
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
