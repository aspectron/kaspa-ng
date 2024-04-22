use kaspa_ng_core::{
    imports::{KaspaRpcClient, MetricsUpdateKind, NetworkId},
    interop::{PendingRequest, Request, ServerAction},
};
use kaspa_wallet_core::rpc::{
    // ConnectOptions, ConnectStrategy, RpcCtl,
    DynRpcApi,
    Resolver,
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

type PortId = u64;
type ReqId = String;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "initPageScript")]
    fn init_page_script(tab_id: u32, args: JsValue);

    #[wasm_bindgen(js_name = "openPopup")]
    fn open_popup_window();
}

type PortEventsClosures =
    Mutex<HashMap<PortId, (Rc<chrome_runtime_port::Port>, Vec<Rc<PortEventClosure>>)>>;
pub struct Server {
    #[allow(dead_code)]
    wallet: Arc<CoreWallet>,
    wallet_server: Arc<WalletServer>,
    closure: Mutex<Option<Rc<ListenerClosure>>>,
    port_closure: Mutex<Option<Rc<PortListenerClosure>>>,
    port_events_closures: PortEventsClosures,
    chrome_extension_id: String,
    // event pending delivery after the popup is open
    pending_request: Mutex<Option<PendingRequest>>,
    // id of request waiting for response
    waiting_response: Mutex<Option<(PortId, ReqId)>>,
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

// TODO: remove this
#[derive(Debug)]
struct InternalMessage {
    #[allow(dead_code)]
    target: Target,
    #[allow(dead_code)]
    data: Vec<u8>,
}

#[derive(Debug)]
enum Message {
    Web(ExtensionMessage),
    #[allow(dead_code)]
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

    if msg_type == "web-api" {
        let info = msg.get_object("data")?;
        let action = ExtensionActions::from_str(&info.get_string("action")?)
            .expect("`action` is required for `web-api` message.");
        let data = info.get_value("data")?;
        let rid = info.try_get_string("rid")?;

        return Ok(ExtensionMessage { action, data, rid }.into());
    }

    // TODO: remove this
    if msg_type == "Internal" {
        let info = msg.get_value("data")?;
        let (target, data) = jsv_to_req(info)?;
        return Ok(InternalMessage { target, data }.into());
    }

    Err("Invalid msg: {msg_type}".to_string().into())
}

impl Server {
    pub async fn new() -> Self {
        let settings = match Settings::load().await {
            Ok(settings) => settings,
            Err(err) => {
                log_error!("Unable to load settings: {err}");
                let settings = Settings::default();
                settings.store().await.unwrap();
                settings
            }
        };

        // ! workflow_chrome::storage::__chrome_storage_unit_test().await;

        let storage = CoreWallet::local_store().unwrap_or_else(|e| {
            panic!("Failed to open local store: {}", e);
        });

        // let list = storage.wallet_list().await.unwrap();
        // log_info!("wallet_list: {:?}", list);
        // log_info!("storage storage: {:?}", storage.descriptor());

        let rpc = Self::create_rpc_client(settings.node.network.into())
            .expect("Unable to create RPC client");

        let wallet = Arc::new(
            CoreWallet::try_with_rpc(Some(rpc), storage, None).unwrap_or_else(|e| {
                panic!("Failed to create wallet instance: {}", e);
            }),
        );

        wallet.enable_metrics_kinds(&[MetricsUpdateKind::WalletMetrics]);
        wallet
            .start_metrics()
            .await
            .expect("Unable to start metrics task");

        let event_handler = Arc::new(ServerEventHandler::default());

        let wallet_server = Arc::new(WalletServer::new(wallet.clone(), event_handler));

        let _application_events = ApplicationEventsChannel::unbounded();

        Self {
            chrome_extension_id: runtime_id().unwrap(),
            closure: Mutex::new(None),
            port_closure: Mutex::new(None),
            port_events_closures: Mutex::new(HashMap::new()),
            wallet,
            wallet_server,
            pending_request: Default::default(),
            waiting_response: Default::default(),
            // runtime,
        }
    }

    pub fn create_rpc_client(network_id: NetworkId) -> Result<Rpc> {
        let resolver = Resolver::default();
        let wrpc_client = Arc::new(KaspaRpcClient::new_with_args(
            WrpcEncoding::Borsh,
            None,
            Some(resolver),
            Some(network_id),
            None,
        )?);
        let rpc_ctl = wrpc_client.ctl().clone();
        let rpc_api: Arc<DynRpcApi> = wrpc_client;
        Ok(Rpc::new(rpc_api, rpc_ctl))
    }

    pub async fn start(self: &Arc<Self>) {
        // log_info!("chrome: starting server...");
        // self.runtime.start();
        self.register_listener();
        self.register_port_listener();
        self.wallet_server.start();

        // log_info!("chrome: starting wallet...");
        self.wallet
            .start()
            .await
            .expect("Unable to start wallet service");
    }

    fn register_port_listener(self: &Arc<Self>) {
        let this = self.clone();

        let closure = Rc::new(Closure::new(
            move |port: chrome_runtime_port::Port| -> JsValue {
                // log_info!("port connected: {port:?}");
                let port = Rc::new(port);
                let port_clone = port.clone();
                let mut rng = rand::thread_rng();
                let port_id = rng.gen::<u64>();

                let this_clone = this.clone();
                let message_closure = Rc::new(Closure::new(move |msg: JsValue| -> JsValue {
                    let this_clone = this_clone.clone();
                    let port_clone = port_clone.clone();
                    spawn_local(async move {
                        let result = this_clone
                            .handle_port_event(
                                js_sys::Object::from(msg),
                                port_clone.clone(),
                                port_id,
                            )
                            .await;
                        port_clone.post_message(result);
                    });

                    JsValue::from(true)
                }));
                port.on_message().add_listener(&message_closure);

                let this_clone = this.clone();
                let port_clone = port.clone();
                let disconnect_closure = Rc::new(Closure::new(move |_| -> JsValue {
                    workflow_log::log_info!("port disconnect: {port_id}");
                    this_clone
                        .port_events_closures
                        .lock()
                        .unwrap()
                        .remove(&port_id);
                    if port_clone.name() == Some("POPUP".to_string()) {
                        let _ = this_clone.on_popup_disconnect();
                    }
                    JsValue::from(true)
                }));
                port.on_disconnect().add_listener(&disconnect_closure);

                this.port_events_closures
                    .lock()
                    .unwrap()
                    .insert(port_id, (port, vec![message_closure, disconnect_closure]));

                JsValue::from(false)
            },
        ));

        chrome_runtime_on_connect::add_on_connect_listener(&closure);
        *self.port_closure.lock().unwrap() = Some(closure);
    }

    fn on_popup_disconnect(self: &Arc<Self>) -> Result<()> {
        let rid = self.waiting_response.lock().unwrap().take();
        if rid.is_some() {
            let response = interop::Response::Canceled {
                error: "User canceled the request.".into(),
            };
            self.send_message_to_port(rid, response)?;
        }
        Ok(())
    }

    fn send_message_to_port(
        self: &Arc<Self>,
        req: Option<(PortId, ReqId)>,
        response: interop::Response,
    ) -> Result<()> {
        let ports: Vec<Rc<chrome_runtime_port::Port>> = {
            self.port_events_closures
                .lock()
                .unwrap()
                .iter()
                .filter(|(id, _)| {
                    if let Some((port_id, _)) = req.as_ref() {
                        *id == port_id
                    } else {
                        true
                    }
                })
                .map(|(_, p)| p.0.clone())
                .collect()
        };

        let object = serde_wasm_bindgen::to_value(&response).unwrap();
        if let Some((_, rid)) = req.as_ref() {
            js_sys::Reflect::set(&object, &"rid".into(), &rid.into()).unwrap();
        }

        for port in ports {
            port.post_message(object.clone());
        }

        Ok(())
    }

    async fn handle_port_event(
        self: &Arc<Self>,
        msg_jsv: js_sys::Object,
        port: Rc<chrome_runtime_port::Port>,
        port_id: PortId,
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
                        .replace(PendingRequest::new(port_id, msg.rid, Request::Connect {}));
                    open_popup_window();
                }
                ExtensionActions::TestRequestResponse => {
                    // TODO - ENQUEUE PENDING REQUEST DATA
                    self.pending_request
                        .lock()
                        .unwrap()
                        .replace(PendingRequest::new(
                            port_id,
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

        // log_info!(
        //     "[WASM] msg: {:?}, sender id:{:?}, {:?}",
        //     msg,
        //     sender.id(),
        //     callback
        // );

        let (target, data) = jsv_to_req(msg)?;
        // log_info!("[WASM] target: {target:?}, data:{data:?}");

        match target {
            Target::Wallet => {
                let msg = WalletMessage::try_from_slice(&data)?;
                spawn_local(async move {
                    let resp = resp_to_jsv(
                        Target::Wallet,
                        self.wallet_server.call_with_borsh(msg.op, &msg.data).await,
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
                // log_info!("[Server] Adaptor: action: {action:?}");
                match action {
                    ServerAction::PendingRequests => {
                        let pending_request =
                            self.pending_request
                                .lock()
                                .unwrap()
                                .take()
                                .map_or(vec![], |a| {
                                    if let Some(id) = a.id.clone() {
                                        *self.waiting_response.lock().unwrap() =
                                            Some((a.sender_id, id));
                                    }
                                    a.try_to_vec().unwrap()
                                });

                        let res = resp_to_jsv(Target::Adaptor, Ok(pending_request));
                        // log_info!("[Server] Adaptor: res: {res:?}");

                        spawn_local(async move {
                            if let Err(err) = callback.call1(&JsValue::UNDEFINED, &res) {
                                log_error!("PendingRequests: callback error: {:?}", err);
                            }
                        });
                    }
                    ServerAction::Response(port_id, rid, data) => {
                        let response = interop::Response::try_from_slice(&data).unwrap();
                        if let Some((pid, id)) = self.waiting_response.lock().unwrap().take() {
                            if pid == port_id && Some(id.clone()) == rid {
                                self.send_message_to_port(Some((port_id, id)), response)?;
                            }
                        }

                        let res = resp_to_jsv(Target::Adaptor, Ok(vec![]));

                        spawn_local(async move {
                            if let Err(err) = callback.call1(&JsValue::UNDEFINED, &res) {
                                log_error!("PendingRequests: callback error: {:?}", err);
                            }
                        });
                    }
                    ServerAction::CloseWindow => {
                        let req = Request::CloseWindow.try_to_vec().unwrap();
                        spawn_local(async move {
                            // log_info!("[SERVER] sending CloseWindow notification");
                            if let Err(err) =
                                send_message(&notify_to_jsv(Target::Runtime, &req)).await
                            {
                                log_warn!("Unable to post Request::CloseWindow: {:?}", err);
                            }
                        });

                        // let res = resp_to_jsv(Target::Adaptor, Ok(vec![]));
                        // spawn_local(async move {
                        //     if let Err(err) = callback.call1(&JsValue::UNDEFINED, &res) {
                        //         log_error!("CloseWindow: callback error: {:?}", err);
                        //     }
                        // });
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
        // log_info!("EVENT HANDLER - POSTING NOTIFICATION! {event:?}");

        let data = event.try_to_vec().unwrap();
        spawn_local(async move {
            let data = notify_to_jsv(Target::Wallet, &data);
            // log_info!("EVENT HANDLER - SENDING MESSAGE!");
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
