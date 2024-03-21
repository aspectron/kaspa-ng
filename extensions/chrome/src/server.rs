use kaspa_ng_core::{
    imports::KaspaRpcClient,
    interop::{PendingRequest, Request, ServerAction},
};
use kaspa_wallet_core::rpc::{
    // ConnectOptions, ConnectStrategy, RpcCtl,
    DynRpcApi,
    NotificationMode,
    Rpc,
    WrpcEncoding,
};

use crate::imports::*;
pub type PortListenerClosure = Closure<dyn FnMut(chrome_runtime_port::Port) -> JsValue>;
pub type PortEventClosure = Closure<dyn FnMut(JsValue) -> JsValue>;
use kaspa_ng_core::interop::Target;
use rand::Rng;
use std::collections::HashMap;
use workflow_core::enums::Describe;
use workflow_wasm::extensions::ObjectExtension;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "initPageScript")]
    fn init_page_script(tab_id: u32, args: JsValue);

    #[wasm_bindgen(js_name = "openPopup")]
    fn open_popup_window();
}

pub struct Server {
    #[allow(dead_code)]
    wallet: Arc<CoreWallet>,
    wallet_server: Arc<WalletServer>,
    closure: Mutex<Option<Rc<ListenerClosure>>>,
    port_closure: Mutex<Option<Rc<PortListenerClosure>>>,
    port_events_closures:
        Mutex<HashMap<u64, (Rc<chrome_runtime_port::Port>, Vec<Rc<PortEventClosure>>)>>,
    chrome_extension_id: String,
    // event pending delivery after the popup is open
    pending_request: Mutex<Option<PendingRequest>>,
}

unsafe impl Send for Server {}
unsafe impl Sync for Server {}

#[derive(Debug, Describe)]
enum ExtensionActions {
    InjectPageScript,
    Connect,
    TestRequestResponse,
}

#[derive(Debug)]
struct ExtensionMessage {
    action: ExtensionActions,
    rid: Option<String>,
    data: JsValue,
}

#[derive(Debug)]
struct InternalMessage {
    target: Target,
    op: u64,
    data: Vec<u8>,
}

#[derive(Debug)]
enum Message {
    Web(ExtensionMessage),
    Internal(InternalMessage),
}

impl From<ExtensionMessage> for Message {
    fn from(value: ExtensionMessage) -> Self {
        Self::Web(value)
    }
}
impl From<InternalMessage> for Message {
    fn from(value: InternalMessage) -> Self {
        Self::Internal(value)
    }
}

fn msg_to_req(msg: js_sys::Object) -> Result<Message> {
    let msg_type = msg.get_string("type")?;

    if msg_type == "WebAPI" {
        let info = msg.get_object("data")?;
        let action = ExtensionActions::from_str(&info.get_string("action")?)
            .expect("`action` is required for WEBAPI message.");
        let data = info.get_value("data")?;
        let rid = info.try_get_string("rid")?;

        return Ok(ExtensionMessage { action, data, rid }.into());
    }

    if msg_type == "Internal" {
        let info = msg.get_value("data")?;
        let (target, op, data) = jsv_to_req(info)?;
        return Ok(InternalMessage { target, op, data }.into());
    }

    Err("Invalid msg: {msg_type}".to_string().into())

    // let src = Vec::<u8>::from_hex(
    //     src.as_string()
    //         .ok_or(Error::custom("expecting string"))?
    //         .as_str(),
    // )?;
    // if src.len() < 10 {
    //     return Err(Error::custom("invalid message length"));
    // }

    // let request = ClientMessage::try_from_slice(&src).unwrap();
    // Ok((request.target, request.op, request.data))
}

impl Server {
    pub async fn new() -> Self {
        // TODO @surinder
        let settings = Settings::load().await.unwrap_or_else(|err| {
            log_error!("Unable to load settings: {err}");
            Settings::default()
        });

        // let _r =
        settings.store().await.unwrap();
        workflow_chrome::storage::__chrome_storage_unit_test().await;

        let storage = CoreWallet::local_store().unwrap_or_else(|e| {
            panic!("Failed to open local store: {}", e);
        });

        let list = storage.wallet_list().await.unwrap();

        log_info!("wallet_list: {:?}", list);
        log_info!("storage storage: {:?}", storage.descriptor());

        let rpc = Self::create_rpc_client().expect("Unable to create RPC client");

        let wallet = Arc::new(
            CoreWallet::try_with_rpc(Some(rpc), storage, None).unwrap_or_else(|e| {
                panic!("Failed to create wallet instance: {}", e);
            }),
        );

        let event_handler = Arc::new(ServerEventHandler::default());

        let wallet_server = Arc::new(WalletServer::new(wallet.clone(), event_handler));

        let _application_events = ApplicationEventsChannel::unbounded();
        // let kaspa = Arc::new(KaspaService::new(application_events.clone(), &settings));
        // let runtime = Runtime::new(&[kaspa.clone()]);
        log_info!("Server init complete");

        Self {
            chrome_extension_id: runtime_id().unwrap(),
            closure: Mutex::new(None),
            port_closure: Mutex::new(None),
            port_events_closures: Mutex::new(HashMap::new()),
            wallet,
            wallet_server,
            pending_request: Default::default(),
            // runtime,
        }
    }

    pub fn create_rpc_client() -> Result<Rpc> {
        let wrpc_client = Arc::new(KaspaRpcClient::new_with_args(
            WrpcEncoding::Borsh,
            NotificationMode::MultiListeners,
            None,
            None,
            None,
        )?);
        let rpc_ctl = wrpc_client.ctl().clone();
        let rpc_api: Arc<DynRpcApi> = wrpc_client;
        Ok(Rpc::new(rpc_api, rpc_ctl))
    }

    pub async fn start(self: &Arc<Self>) {
        log_info!("chrome/src/Server starting...");
        // self.runtime.start();
        self.register_listener();
        self.register_port_listener();
        self.wallet_server.start();

        log_info!("chrome/src/Starting wallet...");
        self.wallet
            .start()
            .await
            .expect("Unable to start wallet service");
    }

    fn register_port_listener(self: &Arc<Self>) {
        let this = self.clone();

        let closure = Rc::new(Closure::new(
            move |port: chrome_runtime_port::Port| -> JsValue {
                workflow_log::log_info!("port connected: {port:?}");
                let port = Rc::new(port);
                let port_clone = port.clone();
                let mut rng = rand::thread_rng();
                let index = rng.gen::<u64>();

                let this_clone = this.clone();
                let message_closure = Rc::new(Closure::new(move |msg: JsValue| -> JsValue {
                    let this_clone = this_clone.clone();
                    let port_clone = port_clone.clone();
                    spawn_local(async move {
                        let result = this_clone
                            .handle_port_event(js_sys::Object::from(msg), port_clone.clone())
                            .await;
                        port_clone.post_message(result);
                    });

                    JsValue::from(true)
                }));
                port.on_message().add_listener(&message_closure);

                let this_clone = this.clone();
                let disconnect_closure = Rc::new(Closure::new(move |_| -> JsValue {
                    workflow_log::log_info!("port disconnect: {index}");
                    this_clone
                        .port_events_closures
                        .lock()
                        .unwrap()
                        .remove(&index);
                    JsValue::from(true)
                }));
                port.on_disconnect().add_listener(&disconnect_closure);

                this.port_events_closures
                    .lock()
                    .unwrap()
                    .insert(index, (port, vec![message_closure, disconnect_closure]));

                JsValue::from(false)
            },
        ));

        chrome_runtime_on_connect::add_on_connect_listener(&closure);
        *self.port_closure.lock().unwrap() = Some(closure);
    }

    async fn handle_port_event(
        self: &Arc<Self>,
        msg_jsv: js_sys::Object,
        port: Rc<chrome_runtime_port::Port>,
    ) -> JsValue {
        let msg = msg_to_req(msg_jsv.clone()).unwrap();
        workflow_log::log_info!("handle_port_event: msg {:?}", msg);
        match msg {
            Message::Web(msg) => match msg.action {
                ExtensionActions::InjectPageScript => {
                    let tab_id = port.sender().tab().id();
                    init_page_script(tab_id, msg.data);
                }
                ExtensionActions::Connect => {
                    self.pending_request
                        .lock()
                        .unwrap()
                        .replace(PendingRequest::new(msg.rid, Request::Connect {}));
                    open_popup_window();
                }
                ExtensionActions::TestRequestResponse => {
                    // TODO - ENQUEUE PENDING REQUEST DATA
                    self.pending_request
                        .lock()
                        .unwrap()
                        .replace(PendingRequest::new(
                            msg.rid,
                            Request::Test {
                                data: msg.data.as_string().unwrap(),
                            },
                        ));

                    // OPENED POPUP MUST CONSUME PENDING REQUEST
                    open_popup_window();
                }
            },
            Message::Internal(_) => {
                //
            }
        }

        format!("handle_port_event: got msg: {msg_jsv:?}").into()
    }

    // Handle message from kaspa-ng-core (client)
    fn register_listener(self: &Arc<Self>) {
        let this = self.clone();

        let closure = Rc::new(Closure::new(
            move |msg, sender: Sender, callback: JsValue| -> JsValue {
                let callback = js_sys::Function::from(callback);

                // let (target, op, data) = jsv_to_req(msg)?;

                match this.clone().handle_message(msg, sender, callback.clone()) {
                    Ok(_) => JsValue::from(true),
                    Err(err) => {
                        log_error!("message handling error: {:?}", err);

                        let resp = resp_to_jsv(Target::Wallet, Err(err));
                        if let Err(err) = callback.call1(&JsValue::UNDEFINED, &resp) {
                            log_error!("onMessage callback error in error handler: {:?}", err);
                        }
                        JsValue::from(false)
                    }
                }
            },
        ));

        chrome_runtime_on_message::add_listener(closure.clone().as_ref());
        *self.closure.lock().unwrap() = Some(closure);
    }

    fn handle_message(
        self: Arc<Self>,
        msg: JsValue,
        sender: Sender,
        callback: Function,
    ) -> Result<()> {
        if let Some(id) = sender.id() {
            if id != self.chrome_extension_id {
                return Err(Error::custom(
                    "Unknown sender id - foreign requests are forbidden",
                ));
            }
        } else {
            return Err(Error::custom("Sender is missing id"));
        }

        log_info!(
            "[WASM] msg: {:?}, sender id:{:?}, {:?}",
            msg,
            sender.id(),
            callback
        );

        let (target, op, data) = jsv_to_req(msg)?;
        log_info!("[WASM] target: {target:?}, op:{op}, data:{data:?}");

        match target {
            Target::Wallet => {
                spawn_local(async move {
                    let resp = resp_to_jsv(
                        Target::Wallet,
                        self.wallet_server.call_with_borsh(op, &data).await,
                    );
                    if let Err(err) = callback.call1(&JsValue::UNDEFINED, &resp) {
                        log_error!("onMessage callback error: {:?}", err);
                    }
                });
            }
            Target::Runtime => {
                todo!()
            }
            Target::Adaptor => {
                let action = ServerAction::try_from_slice(&data)?;
                log_info!("[Server] Adaptor: action: {action:?}");
                match action {
                    ServerAction::PendingRequests => {
                        let pending_request = self
                            .pending_request
                            .lock()
                            .unwrap()
                            .take()
                            .map_or(vec![], |a| a.try_to_vec().unwrap());
                        let res = resp_to_jsv(Target::Adaptor, Ok(pending_request));
                        log_info!("[Server] Adaptor: res: {res:?}");
                        spawn_local(async move {
                            if let Err(err) = callback.call1(&JsValue::UNDEFINED, &res) {
                                log_error!("PendingRequests: callback error: {:?}", err);
                            }
                        });
                    }
                    ServerAction::Response(rid, data) => {
                        let ports: Vec<Rc<chrome_runtime_port::Port>> = {
                            self.port_events_closures
                                .lock()
                                .unwrap()
                                .iter()
                                .map(|(_, p)| p.0.clone())
                                .collect()
                        };

                        let response = interop::Response::try_from_slice(&data).unwrap();
                        let object = serde_wasm_bindgen::to_value(&response).unwrap();
                        js_sys::Reflect::set(&object, &"rid".into(), &rid.into()).unwrap();
                        for port in ports {
                            port.post_message(object.clone());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    // TODO - implement
    // fn _post_notify(&self, op: u64, data: Vec<u8>) -> Result<()> {
    //     spawn_local(async move {
    //         if let Err(err) = send_message(&notify_to_jsv(op, &data)).await {
    //             log_warn!("Unable to post notification: {:?}", err);
    //         }
    //     });

    //     Ok(())
    // }

    // fn start(self: &Arc<Self>) {

    // }
}

#[derive(Default)]
struct ServerEventHandler {}

#[async_trait]
impl EventHandler for ServerEventHandler {
    async fn handle_event(&self, event: &Events) {
        log_info!("EVENT HANDLER - POSTING NOTIFICATION! {event:?}");

        let data = event.try_to_vec().unwrap();
        spawn_local(async move {
            let data = notify_to_jsv(Target::Wallet, &data);
            log_info!("EVENT HANDLER - SENDING MESSAGE!");
            if let Err(err) = send_message(&data).await {
                log_warn!("Unable to post notification: {:?}", err);
            }
        });
    }

    // async fn handle_event(&self, event: JsValue) -> Result<()> {
    //     log_info!("event: {:?}", event);
    //     Ok(())
    // }
}
