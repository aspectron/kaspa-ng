use kaspa_ng_core::imports::KaspaRpcClient;
// use kaspa_wallet_core::rpc::{Rpc, WrpcEncoding};
use kaspa_wallet_core::rpc::{
    ConnectOptions, ConnectStrategy, DynRpcApi, NotificationMode, Rpc, RpcCtl, WrpcEncoding,
};

use crate::imports::*;

pub struct Server {
    #[allow(dead_code)]
    wallet: Arc<CoreWallet>,
    wallet_server: Arc<WalletServer>,
    closure: Mutex<Option<Rc<ListenerClosure>>>,
    chrome_extension_id: String,
}

unsafe impl Send for Server {}
unsafe impl Sync for Server {}

// impl Default for Server {
//     fn default() -> Self {
//         Self::new()
//     }
// }

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
            wallet,
            wallet_server,
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
        log_info!("Server starting...");
        // self.runtime.start();
        self.register_listener();
        self.wallet_server.start();

        log_info!("Starting wallet...");
        self.wallet
            .start()
            .await
            .expect("Unable to start wallet service");
    }

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

        add_listener(closure.clone().as_ref());
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
    async fn handle_event(&self, event: &Box<Events>) {
        log_info!("EVENT HANDLER - POSTING NOTIFICATION!");

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
