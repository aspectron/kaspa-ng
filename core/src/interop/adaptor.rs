//! Wallet Interop

use crate::imports::*;
use crate::interop::message::*;
use crate::interop::transport;

#[repr(u64)]
#[derive(Debug, Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
pub enum Action {
    Test { request: TestRequest },
    Connect { request: ConnectRequest },
    SignMessage { request: SignMessageRequest },
}

pub struct Adaptor {
    sender: Arc<dyn transport::Sender>,
    application_events: ApplicationEventsChannel,
    action: Mutex<Option<Action>>,
    response: Channel<Vec<u8>>,
}

impl Adaptor {
    pub fn new(
        sender: Arc<dyn transport::Sender>,
        application_events: ApplicationEventsChannel,
    ) -> Self {
        Self {
            sender,
            application_events,
            action: Mutex::new(None),
            response: Channel::unbounded(),
        }
    }

    pub async fn init(self: Arc<Self>) -> Result<()> {
        // @surinder
        // TODO use `self.sender` to send a message to the runtime and get pending requests

        Ok(())
    }

    // clear the current action (must be called after the response is sent)
    fn clear(&self) {
        *self.action.lock().unwrap() = None;
    }

    pub fn render(&self, _core: &mut Core, ui: &mut Ui) -> bool {
        let action = self.action.lock().unwrap().clone();
        match action {
            Some(Action::Test { request }) => {
                let mut ctx = ();
                Panel::new(&mut ctx)
                    .with_caption("Adaptor Test")
                    .with_body(|_ctx, ui| {
                        ui.label("Test Request:");
                        ui.label(format!("{:?}", request));
                        ui.separator();

                        if ui.button("Complete Test Response").clicked() {
                            // TODO - place something in response
                            let response = TestResponse {};
                            self.response
                                .try_send(response.try_to_vec().unwrap())
                                .unwrap();

                            // clear the action
                            self.clear();
                        }
                    })
                    .render(ui);

                // consume rendering - do not render main UI
                true
            }
            // TODO - handle connect and sign message etc.
            _ => {
                // continue rendering to standard KNG UI
                false
            }
        }
    }

    // pub async fn handle_message(self: Arc<Self>, data: Vec<u8>) -> Result<Vec<u8>> {
    pub async fn handle_message(self: Arc<Self>, action: Action) -> Result<Vec<u8>> {
        self.action.lock().unwrap().replace(action);

        let response = self.response.receiver.recv().await?;
        Ok(response)
    }
}
