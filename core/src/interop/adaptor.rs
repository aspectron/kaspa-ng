//! Wallet Interop

use crate::imports::*;
use crate::interop::transport;
use crate::interop::{message::*, Target};

#[repr(u64)]
#[derive(Debug, Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
pub enum Action {
    Test { request: TestRequest },
    Connect { request: ConnectRequest },
    SignMessage { request: SignMessageRequest },
}
cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        impl From<Action> for wasm_bindgen::JsValue{
            fn from(action:Action)->Self{
                action.try_to_vec().unwrap().to_hex().into()
                //action.try_to_vec().unwrap().into_iter().map(wasm_bindgen::JsValue::from).collect::<js_sys::Array>().into()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
pub struct PendingRequest {
    id: Option<String>,
    action: Action,
}

impl PendingRequest {
    pub fn new(id: Option<String>, action: Action) -> Self {
        Self { id, action }
    }
}

#[repr(u64)]
#[derive(Debug, Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
pub enum ServerAction {
    PendingRequests,
    Response(Option<String>, Vec<u8>),
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
        log_info!("Adaptor:init");
        let res = self
            .sender
            .send_message(
                Target::Adaptor,
                0,
                ServerAction::PendingRequests.try_to_vec()?,
            )
            .await?;
        log_info!("Adaptor:init res: {res:?}");
        if !res.is_empty() {
            let this = self.clone();
            let PendingRequest { id, action } = PendingRequest::try_from_slice(&res)?;
            log_info!("Adaptor:init req-id:{id:?}, action: {action:?}");
            workflow_core::task::spawn(async move {
                match self.handle_message(action).await {
                    Ok(data) => {
                        log_info!("Adaptor:init handle_message: data:{data:?}");
                        let _res = this
                            .sender
                            .send_message(
                                Target::Adaptor,
                                0,
                                ServerAction::Response(id, data).try_to_vec()?,
                            )
                            .await;
                        log_info!("Adaptor: ServerAction::Response: {_res:?}");
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
        *self.action.lock().unwrap() = None;
    }

    pub fn render(&self, core: &mut Core, ui: &mut Ui) -> bool {
        let action = self.action.lock().unwrap().clone();
        let action = match action {
            Some(action) => action,
            None => return false,
        };
        match action {
            Action::Test { request } => {
                let mut ctx = ();
                Panel::new(&mut ctx)
                    .with_caption("Adaptor Test")
                    .with_body(|_ctx, ui| {
                        ui.label("Test Request:");
                        ui.label(format!("{:?}", request));
                        ui.separator();

                        if ui.button("Complete Test Response").clicked() {
                            // TODO - place something in response
                            let response: interop::Response = TestResponse {
                                response: "xyz".into(),
                            }
                            .into();
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

            Action::Connect { request } => {
                log_info!("Adaptor render -> Action::Connect: {:?}", request);
                let account_manager = core
                    .modules()
                    .get(&TypeId::of::<modules::AccountManager>())
                    .unwrap()
                    .clone();
                let account_manager = account_manager.get::<modules::AccountManager>();
                if let Some(account) = account_manager.account() {
                    let response: interop::Response = ConnectResponse {
                        address: account.receive_address().to_string(),
                    }
                    .into();
                    self.response
                        .try_send(response.try_to_vec().unwrap())
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

    pub async fn handle_message(self: Arc<Self>, action: Action) -> Result<Vec<u8>> {
        self.action.lock().unwrap().replace(action);

        let response = self.response.receiver.recv().await?;
        Ok(response)
    }
}
