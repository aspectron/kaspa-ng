use crate::ipc::*;
use js_sys::Function;
use kaspa_wallet_core::error::Error;
use kaspa_wallet_core::result::Result;
use kaspa_wallet_core::runtime;
use kaspa_wallet_core::runtime::api::transport::*;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use workflow_log::*;
use kaspa_ng_core::events::ApplicationEventsChannel;
use kaspa_ng_core::settings::Settings;
use kaspa_ng_core::runtime::kaspa::KaspaService;
use kaspa_ng_core::runtime::Runtime;

type ListenerClosure = Closure<dyn FnMut(JsValue, Sender, JsValue) -> JsValue>;

pub struct Server {
    wallet : Arc<runtime::Wallet>,
    wallet_server: Arc<WalletServer>,
    // closure: Mutex<Option<Rc<Closure<dyn FnMut(JsValue, Sender, JsValue) -> JsValue>>>>,
    closure: Mutex<Option<Rc<ListenerClosure>>>,
    runtime : Runtime,
    extension_id: String,
}

unsafe impl Send for Server {}
unsafe impl Sync for Server {}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    pub fn new() -> Self {

        let settings = Settings::load().unwrap_or_else(|err| {
            log_error!("Unable to load settings: {err}");
            Settings::default()
        });


        let storage = runtime::Wallet::local_store().unwrap_or_else(|e| {
            panic!("Failed to open local store: {}", e);
        });

        let wallet = Arc::new(runtime::Wallet::try_with_rpc(None, storage, None).unwrap_or_else(|e| {
            panic!("Failed to create wallet instance: {}", e);
        }));

        let wallet_server = Arc::new(WalletServer::new(wallet.clone()));

        let application_events = ApplicationEventsChannel::unbounded(None);
        let kaspa = Arc::new(KaspaService::new(application_events.clone(), &settings));
        let runtime = Runtime::new(&[kaspa.clone()]);

        Self {
            extension_id: runtime_id().unwrap(),
            closure: Mutex::new(None),
            wallet,
            wallet_server,
            runtime,
        }
    }

    pub fn init(self: &Arc<Self>) {
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
            if id != self.extension_id {
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
                    let resp = resp_to_jsv(self.wallet_server.call_with_borsh(op, &data).await);
                    if let Err(err) = callback.call1(&JsValue::UNDEFINED, &resp) {
                        log_error!("onMessage callback error: {:?}", err);
                    }
                });
            }
            Target::Interop => {
                todo!()
            }
        }

        Ok(())
    }

    fn post_notify(&self, data : Vec<u8>) -> Result<()> {
        spawn_local(async move {
            if let Err(err) = send_message(&notify_to_jsv(0x0, &data)).await {
                log_warning!("Unable to post notification: {:?}", err);
            }
        });

        Ok(())
    }
}

