use kaspa_wallet_core::error::Error as WalletError;
use std::ops::Deref;
use thiserror::Error;

#[derive(Debug, Error)]
#[error(transparent)]
pub struct Error(Box<WalletError>);

impl Error {
    pub fn custom<S: Into<String>>(msg: S) -> Self {
        Self(Box::new(WalletError::Custom(msg.into())))
    }

    pub fn into_inner(self) -> WalletError {
        *self.0
    }
}

impl Deref for Error {
    type Target = WalletError;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> From<E> for Error
where
    WalletError: From<E>,
{
    fn from(e: E) -> Self {
        Self(Box::new(e.into()))
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
