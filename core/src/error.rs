use std::net::AddrParseError;
use thiserror::Error;
use wasm_bindgen::JsValue;
use workflow_core::channel::{ChannelError, RecvError, SendError, TryRecvError, TrySendError};

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Custom(String),

    #[error("{0}")]
    Eframe(String),

    #[error(transparent)]
    WalletError(#[from] kaspa_wallet_core::error::Error),

    #[error("Not a local wallet")]
    WalletIsNotLocal,

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("Channel send() error")]
    SendError,

    #[error("Channel recv() error")]
    RecvError,

    #[error("Channel try_send() error")]
    TrySendError,

    #[error("Channel try_recv() error")]
    TryRecvError,

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
    Metrics(#[from] kaspa_metrics_core::error::Error),

    #[error(transparent)]
    AddrParseError(#[from] AddrParseError),

    #[error(transparent)]
    I18n(#[from] workflow_i18n::error::Error),

    #[error("Network id is not valid during the wallet open operation")]
    WalletOpenNetworkId,

    #[error("Account descriptors are not valid during the wallet open operation")]
    WalletOpenAccountDescriptors,

    #[error(transparent)]
    AddressError(#[from] kaspa_addresses::AddressError),

    #[error("Invalid network type")]
    InvalidNetworkType,

    #[error("Invalid network '{0}'")]
    InvalidNetwork(String),

    #[error("Http error: {0}")]
    HttpError(#[from] workflow_http::error::Error),

    #[error("Invalid JSON: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    NetworkType(#[from] kaspa_consensus_core::network::NetworkTypeError),

    #[error("Account creation error")]
    AccountCreationError,

    #[error(transparent)]
    Toml(#[from] toml::de::Error),

    #[error("{0}")]
    JsError(workflow_wasm::jserror::JsErrorData),

    #[error("ParseInt")]
    ParseInt(#[from] std::num::ParseIntError),
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

impl From<JsValue> for Error {
    fn from(err: JsValue) -> Self {
        Error::JsError(workflow_wasm::jserror::JsErrorData::from(err))
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

impl From<RecvError> for Error {
    fn from(_: RecvError) -> Self {
        Error::RecvError
    }
}

impl From<TryRecvError> for Error {
    fn from(_: TryRecvError) -> Self {
        Error::TryRecvError
    }
}

impl<T> From<ChannelError<T>> for Error {
    fn from(err: ChannelError<T>) -> Self {
        Error::ChannelError(err.to_string())
    }
}
