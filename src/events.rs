use crate::imports::*;
use crate::result::Result;
use kaspa_wallet_core::events as kaspa;

// pub type EventChannel = Channel<Events>;

#[derive(Clone, Debug)]
pub enum Events {
    Error(String),
    Wallet(kaspa::Events),
    // TryUnlock(Secret),
    UnlockSuccess,
    UnlockFailure { message : String },
    Lock,
    Send,
    Deposit,
    Overview,
    Transactions,
    Accounts,
    Settings,
    Exit,
}

impl Events {
    pub fn info(&self) -> String {
        match self {
            Events::Error(err) => format!("Error: {}", err),
            Events::Wallet(_) => "Wallet".to_string(),
            // Events::TryUnlock(_) => "TryUnlock".to_string(),
            Events::UnlockSuccess {..} => "UnlockSuccess".to_string(),
            Events::UnlockFailure {..} => "UnlockFailure".to_string(),
            Events::Lock {..} => "Lock".to_string(),
            Events::Send {..} => "Send".to_string(),
            Events::Deposit {..} => "Deposit".to_string(),
            Events::Overview {..} => "Overview".to_string(),
            Events::Transactions {..} => "Transactions".to_string(),
            Events::Accounts {..} => "Accounts".to_string(),
            Events::Settings {..} => "Settings".to_string(),
            Events::Exit {..} => "Exit".to_string(),
        }
    }

}