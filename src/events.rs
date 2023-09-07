use crate::imports::*;
use crate::result::Result;

#[derive(Debug)]
pub enum Events {
    TryUnlock(Secret),
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
            Events::TryUnlock(_) => "TryUnlock".to_string(),
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

    pub fn handle(&self, wallet : &mut Wallet) -> Result<()> {
        match self {
            Events::TryUnlock(_secret) => {
                let mut unlock = wallet.get_mut::<section::Unlock>();
                unlock.message = Some("Error unlocking wallet...".to_string());
                unlock.lock();
            },
            Events::UnlockSuccess => {
                wallet.select::<section::Overview>();
            },
            Events::UnlockFailure {..} => {

            },
            _ => unimplemented!()
        }

        Ok(())        
    }
}