use thiserror::Error;
use workflow_core::channel::{SendError, TrySendError};

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Custom(String),

    #[error(transparent)]
    WalletError(#[from] kaspa_wallet_core::error::Error),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("Channel send() error")]
    SendError,

    #[error("Channel try_send() error")]
    TrySendError,

    #[error(transparent)]
    WrpcClientError(#[from] kaspa_wrpc_client::error::Error),

    #[error(transparent)]
    Bip32(#[from] kaspa_bip32::Error),
}

impl Error {
    pub fn custom<T: Into<String>>(msg: T) -> Self {
        Error::Custom(msg.into())
    }
}

impl<T> From<SendError<T>> for Error {
    fn from(_: SendError<T>) -> Self {
        Error::SendError
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
