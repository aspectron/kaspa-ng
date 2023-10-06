use crate::ipc::*;
use js_sys::Function;
use kaspa_ng_core::events::ApplicationEventsChannel;
// use kaspa_ng_core::runtime::kaspa::KaspaService;
// use kaspa_ng_core::runtime::Runtime;
use kaspa_ng_core::settings::Settings;
use kaspa_wallet_core::error::Error;
use kaspa_wallet_core::result::Result;
use kaspa_wallet_core::runtime;
//use kaspa_wallet_core::runtime::api::transport::*;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use workflow_log::*;

type ListenerClosure = Closure<dyn FnMut(JsValue, Sender, JsValue) -> JsValue>;

pub struct Server {
    #[allow(dead_code)]
    wallet: Arc<runtime::Wallet>,
    //wallet_server: Arc<WalletServer>,
    closure: Mutex<Option<Rc<ListenerClosure>>>,
    // runtime: Runtime,
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

        let _r = settings.store().await.unwrap();
        workflow_store::fs::__chrome_storage_unit_test().await;

        let storage = runtime::Wallet::local_store().unwrap_or_else(|e| {
            panic!("Failed to open local store: {}", e);
        });

        let list = storage.wallet_list().await.unwrap();

        log_info!("wallet_list: {:?}", list);
        log_info!("storage storage: {:?}", storage.descriptor());

        let wallet = Arc::new(
            runtime::Wallet::try_with_rpc(None, storage, None).unwrap_or_else(|e| {
                panic!("Failed to create wallet instance: {}", e);
            }),
        );

        //let wallet_server = Arc::new(WalletServer::new(wallet.clone()));

        let _application_events = ApplicationEventsChannel::unbounded(None);
        // TODO @surinder
        // let kaspa = Arc::new(KaspaService::new(application_events.clone(), &settings));
        // TODO @surinder
        // let runtime = Runtime::new(&[kaspa.clone()]);

        Self {
            chrome_extension_id: runtime_id().unwrap(),
            closure: Mutex::new(None),
            wallet,
            //wallet_server,
            // runtime,
        }
    }

    pub fn start(self: &Arc<Self>) {
        // self.runtime.start();
        self.register_listener();
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

                        let resp = resp_to_jsv(Err(err));
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
                    // let resp = resp_to_jsv(self.wallet_server.call_with_borsh(op, &data).await);
                    // if let Err(err) = callback.call1(&JsValue::UNDEFINED, &resp) {
                    //     log_error!("onMessage callback error: {:?}", err);
                    // }
                });
            }
            Target::Interop => {
                todo!()
            }
        }

        Ok(())
    }

    // TODO - implement
    fn _post_notify(&self, op: u64, data: Vec<u8>) -> Result<()> {
        spawn_local(async move {
            if let Err(err) = send_message(&notify_to_jsv(op, &data)).await {
                log_warning!("Unable to post notification: {:?}", err);
            }
        });

        Ok(())
    }
}
