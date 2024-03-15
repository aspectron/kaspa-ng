// Chrome Extension Wallet Adaptor

use crate::imports::*;

pub enum WebEvent {
    AccountSelection(AccountId),
}

pub trait AdaptorApi: Sync + Send {
    fn post_to_server(&self, event: WebEvent);
}
