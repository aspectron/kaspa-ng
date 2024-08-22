use crate::imports::*;
use kaspa_wallet_core::error::Error;
use kaspa_wallet_core::result::Result;

#[repr(u8)]
#[derive(Debug, BorshSerialize, BorshDeserialize)]
#[borsh(use_discriminant = true)]
pub enum Target {
    Wallet = 0,
    Runtime = 1,
    Adaptor = 2,
}

impl TryFrom<u8> for Target {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Target::Wallet),
            1 => Ok(Target::Runtime),
            2 => Ok(Target::Adaptor),
            _ => Err(Error::custom("invalid message target")),
        }
    }
}

#[async_trait]
pub trait Sender: Send + Sync {
    async fn send_message(&self, target: Target, data: Vec<u8>) -> Result<Vec<u8>>;
}
