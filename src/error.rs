
use thiserror::Error;
use workflow_core::channel::TrySendError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Custom(String),

    #[error(transparent)]
    WalletError(#[from] kaspa_wallet_core::error::Error),
    
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("Channel try_send() error")]
    TrySendError,
    // #[error("downcast error for {0}")]
    // DowncastError(String),
}

impl Error {
    pub fn custom<T: Into<String>>(msg: T) -> Self {
        Error::Custom(msg.into())
    }
}

impl<T> From<TrySendError<T>> for Error {
    fn from(_: TrySendError<T>) -> Self {
        Error::TrySendError
    }
}

// impl<T> From<downcast::DowncastError<T>> for Error {
//     fn from(e: downcast::DowncastError<T>) -> Self {
//         Error::DowncastError(e.to_string())
//     }
// }
