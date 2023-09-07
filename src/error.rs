
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Custom(String),

    #[error(transparent)]
    WalletError(#[from] kaspa_wallet_core::error::Error),
    
    // #[error("downcast error for {0}")]
    // DowncastError(String),
}

impl Error {
    pub fn custom<T: Into<String>>(msg: T) -> Self {
        Error::Custom(msg.into())
    }
}

// impl<T> From<downcast::DowncastError<T>> for Error {
//     fn from(e: downcast::DowncastError<T>) -> Self {
//         Error::DowncastError(e.to_string())
//     }
// }
