use thiserror::Error;
use wasm_bindgen::JsValue;
use workflow_core::channel::{ChannelError, SendError, TrySendError};

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Custom(String),

    #[error("{0}")]
    Eframe(String),

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
    WorkflowStorage(#[from] workflow_store::error::Error),

    #[error("Channel error: {0}")]
    ChannelError(String),

    #[error(transparent)]
    Bip32(#[from] kaspa_bip32::Error),

    #[error("Missing external kaspad node binary")]
    MissingExternalKaspadBinary,

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error(transparent)]
    RpcError(#[from] kaspa_rpc_core::error::RpcError),

    #[error("Node startup error: {0}")]
    NodeStartupError(std::io::Error),

    #[error("Unable to acquire node stdout handle")]
    NodeStdoutHandleError,

    #[error("Metrics: {0}")]
    Metrics(#[from] kaspa_metrics::error::Error),
}

impl Error {
    pub fn custom<T: Into<String>>(msg: T) -> Self {
        Error::Custom(msg.into())
    }
}

impl From<eframe::Error> for Error {
    fn from(err: eframe::Error) -> Self {
        Error::Eframe(err.to_string())
    }
}

impl From<Error> for JsValue {
    fn from(err: Error) -> Self {
        JsValue::from_str(&err.to_string())
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

impl<T> From<ChannelError<T>> for Error {
    fn from(err: ChannelError<T>) -> Self {
        Error::ChannelError(err.to_string())
    }
}

// impl<T> From<downcast::DowncastError<T>> for Error {
//     fn from(e: downcast::DowncastError<T>) -> Self {
//         Error::DowncastError(e.to_string())
//     }
// }
