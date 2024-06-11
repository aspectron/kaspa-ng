//! Wallet Interop

use crate::imports::*;
use crate::interop::transport;
use crate::interop::{message::*, Target};

#[derive(Debug, Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
pub struct PendingRequest {
    pub sender_id: u64,
    pub id: Option<String>,
    request: Request,
}

impl PendingRequest {
    pub fn new(sender_id: u64, id: Option<String>, request: Request) -> Self {
        Self {
            sender_id,
            id,
            request,
        }
    }
}

#[repr(u64)]
#[derive(Debug, Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
pub enum ServerAction {
    PendingRequests,
    Response(u64, Option<String>, Vec<u8>),
    CloseWindow,
}

pub struct Adaptor {
    sender: Arc<dyn transport::Sender>,
    _application_events: ApplicationEventsChannel,
    request: Mutex<Option<Request>>,
    response: Channel<Vec<u8>>,
}

impl Adaptor {
    pub fn new(
        sender: Arc<dyn transport::Sender>,
        _application_events: ApplicationEventsChannel,
    ) -> Self {
        Self {
            sender,
            _application_events,
            request: Mutex::new(None),
            response: Channel::unbounded(),
        }
    }

    pub async fn init(self: Arc<Self>) -> Result<()> {
        log_info!("Adaptor::init()");
        let res = self
            .sender
            .send_message(
                Target::Adaptor,
                borsh::to_vec(&ServerAction::PendingRequests)?,
            )
            .await?;
        // log_info!("Adaptor:init res: {res:?}");
        if !res.is_empty() {
            let this = self.clone();
            let PendingRequest {
                sender_id,
                id,
                request,
            } = PendingRequest::try_from_slice(&res)?;
            // log_info!("Adaptor:init req-id:{id:?}, action: {request:?}");
            workflow_core::task::spawn(async move {
                match self.handle_message(request).await {
                    Ok(data) => {
                        // log_info!("Adaptor:init handle_message: data:{data:?}");
                        let res = this
                            .sender
                            .send_message(
                                Target::Adaptor,
                                borsh::to_vec(&ServerAction::Response(sender_id, id, data))?,
                            )
                            .await;
                        if res.is_ok() {
                            //TODO: should we check which request require autoclose?
                            //log_info!("Adaptor:init sending window close msg");
                            //let _ = this.sender.send_message(Target::Adaptor, ServerAction::CloseWindow.try_to_vec()?).await;
                            #[cfg(target_arch = "wasm32")]
                            let _ = workflow_dom::utils::window().close();
                        }
                        Ok(())
                    }
                    Err(err) => Err(err),
                }
            });
        }
        Ok(())
    }

    // clear the current action (must be called after the response is sent)
    fn clear(&self) {
        *self.request.lock().unwrap() = None;
    }

    pub fn render(&self, core: &mut Core, ui: &mut Ui) -> bool {
        let request = match self.request.lock().unwrap().clone() {
            Some(request) => request,
            None => return false,
        };

        match request {
            Request::Test { data } => {
                let mut ctx = ();
                Panel::new(&mut ctx)
                    .with_caption("Adaptor Test")
                    .with_body(|_ctx, ui| {
                        ui.label("Test Request:");
                        ui.label(format!("{:?}", data));
                        ui.separator();

                        if ui.button("Complete Test Response").clicked() {
                            // TODO - place something in response
                            let response = interop::Response::Test {
                                response: "xyz".into(),
                            };
                            self.response
                                .try_send(borsh::to_vec(&response).unwrap())
                                .unwrap();

                            // clear the action
                            self.clear();
                        }
                    })
                    .render(ui);

                // consume rendering - do not render main UI
                true
            }

            Request::Connect {} => {
                log_info!("Adaptor render -> Action::Connect: {:?}", request);
                let account_manager = core
                    .modules()
                    .get(&TypeId::of::<modules::AccountManager>())
                    .unwrap()
                    .clone();
                let account_manager = account_manager.get::<modules::AccountManager>();
                if let Some(account) = account_manager.account() {
                    let response = interop::Response::Connect {
                        address: account.receive_address().to_string(),
                    };
                    self.response
                        .try_send(borsh::to_vec(&response).unwrap())
                        .unwrap();
                    self.clear();
                }
                false
            }

            // TODO - handle connect and sign message etc.
            _ => {
                // continue rendering to standard KNG UI
                false
            }
        }
    }

    pub async fn handle_message(self: Arc<Self>, request: Request) -> Result<Vec<u8>> {
        self.request.lock().unwrap().replace(request);

        let response = self.response.receiver.recv().await?;
        Ok(response)
    }
}
