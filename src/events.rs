use crate::imports::*;
use kaspa_wallet_core::events as kaspa;

#[derive(Clone)]
pub enum Events {
    Error(Box<String>),
    WalletList {
        wallet_list: Arc<Vec<String>>,
    },
    AccountList {
        account_list: Arc<Vec<Arc<dyn runtime::Account>>>,
    },
    Wallet(Box<kaspa::Events>),
    // TryUnlock(Secret),
    UnlockSuccess,
    UnlockFailure {
        message: String,
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
