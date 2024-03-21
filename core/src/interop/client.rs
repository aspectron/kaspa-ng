// use crate::events::Events;
use crate::imports::*;
use crate::interop::transport;
use crate::interop::transport::Target;
use crate::interop::{Adaptor, Request};
pub use kaspa_wallet_core::api::transport::BorshCodec;
pub use kaspa_wallet_core::api::transport::{EventHandler, WalletServer};

pub struct Client {
    sender: Arc<dyn transport::Sender>,
    application_events: ApplicationEventsChannel,
    adaptor: Arc<Adaptor>,
}

impl Client {
    pub fn new(
        sender: Arc<dyn transport::Sender>,
        application_events: ApplicationEventsChannel,
    ) -> Self {
        let adaptor = Arc::new(Adaptor::new(sender.clone(), application_events.clone()));
        Self {
            sender,
            application_events,
            adaptor,
        }
    }

    pub async fn init(self: Arc<Self>) -> Result<()> {
        self.adaptor.clone().init().await
    }

    pub fn adaptor(&self) -> &Arc<Adaptor> {
        &self.adaptor
    }

    pub async fn handle_message(
        &self,
        target: Target,
        _op: u64,
        data: Vec<u8>,
    ) -> Result<Option<Vec<u8>>> {
        match target {
            Target::Wallet => {
                let event = Box::new(kaspa_wallet_core::events::Events::try_from_slice(&data)?);

                self.application_events
                    .sender
                    .try_send(kaspa_ng_core::events::Events::Wallet { event })
                    .unwrap();

                Ok(None)
            }
            Target::Runtime => Ok(None),
            Target::Adaptor => {
                let action = Request::try_from_slice(&data)?;

                let response = self.adaptor.clone().handle_message(action).await?;
                // @surinder TODO - make sure that chrome extension client handles
                // the response properly here and returns it to the originator
                Ok(Some(response))
            }
        }
    }
}
