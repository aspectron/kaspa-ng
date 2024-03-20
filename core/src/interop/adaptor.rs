//! Wallet Interop

use crate::imports::*;
use crate::interop::message::*;
use crate::interop::transport;

pub struct Adaptor {
    sender: Arc<dyn transport::Sender>,
    application_events: ApplicationEventsChannel,
}

impl Adaptor {
    pub fn new(
        sender: Arc<dyn transport::Sender>,
        application_events: ApplicationEventsChannel,
    ) -> Self {
        Self {
            sender,
            application_events,
        }
    }

    pub async fn init(self: Arc<Self>) -> Result<()> {
        Ok(())
    }

    pub async fn test(self: Arc<Self>, _request: TestRequest) -> Result<TestResponse> {
        Ok(TestResponse {})
    }

    pub async fn connect(self: Arc<Self>, _request: ConnectRequest) -> Result<ConnectResponse> {
        Ok(ConnectResponse {})
    }

    pub async fn sign_message(
        self: Arc<Self>,
        _request: SignMessageRequest,
    ) -> Result<SignMessageResponse> {
        Ok(SignMessageResponse {})
    }

    pub fn render(&self, _core: &mut Core, _ui: &mut Ui) -> bool {
        false
    }

    pub async fn handle_message(self: Arc<Self>, action: Action, data: Vec<u8>) -> Result<Vec<u8>> {
        match action {
            Action::Test => {
                let request = TestRequest::try_from_slice(&data)?;
                let response = self.test(request).await?;
                Ok(response.try_to_vec()?)
            }
            Action::Connect => {
                let request = ConnectRequest::try_from_slice(&data)?;
                let response = self.connect(request).await?;
                Ok(response.try_to_vec()?)
            }
            Action::SignMessage => {
                let request = SignMessageRequest::try_from_slice(&data)?;
                let response = self.sign_message(request).await?;
                Ok(response.try_to_vec()?)
            }
        }
    }
}

// ???
#[repr(u64)]
#[derive(Debug, Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
pub enum Action {
    Test,
    Connect,
    SignMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
pub struct AdaptorEvent {
    action: Action,
    request: Vec<u8>,
}

pub type AdaptorEventsChannel = workflow_core::channel::Channel<AdaptorEvent>;
