use std::sync::{Arc,Mutex};
use std::rc::Rc;

// use kaspa_utils::hex::*;
// use async_trait::async_trait;
use kaspa_wallet_core::runtime;//::Wallet;
// use kaspa_wallet_core::runtime::api::*;
use kaspa_wallet_core::runtime::api::transport::*;
use kaspa_wallet_core::result::Result;
use kaspa_wallet_core::error::Error;
use js_sys::Function;
use crate::ipc::*;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::prelude::*;
// use workflow_core::channel::oneshot;
use workflow_log::*;

// use workfow_core::task::dispatch;


pub struct Server {
    // pub ipc : Ipc,
    // pub wallet_api : Arc<dyn WalletApi>,
    wallet_server : Arc<WalletServer>,
    closure: Mutex<Option<Rc<Closure<dyn FnMut(JsValue, Sender, JsValue) -> JsValue>>>>,
    // closure: Mutex<Option<Rc<Closure<dyn FnMut(JsValue, Sender, JsValue)>>>>,
    runtime_id: String,

}

impl Server {
    pub fn new() -> Self {

        let storage = runtime::Wallet::local_store().unwrap_or_else(|e| {
            panic!("Failed to open local store: {}", e);
        });

        let wallet =
            // runtime::Wallet::try_with_rpc(None, storage, Some(settings.node.network.into()))
            runtime::Wallet::try_with_rpc(None, storage, None)
                .unwrap_or_else(|e| {
                    panic!("Failed to create wallet instance: {}", e);
                });

        let wallet_server = Arc::new(WalletServer::new(Arc::new(wallet)));

        
        Self {
            // ipc: Ipc::new(),
            runtime_id : runtime_id().unwrap(),
            closure: Mutex::new(None),
            wallet_server: wallet_server.clone(),
        }
        // server.init();
        // server
    }

    pub fn init(self : &Arc<Self>) {

        // let wallet_api = self.wallet_server.wallet_api().clone();
        // let wallet_server = self.wallet_server.clone();
        let this = self.clone();
        
        // let closure = Rc::new(Closure::new(move |msg, sender: Sender, callback: &js_sys::Function| {
        // let closure = Rc::new(Closure::new(move |msg, sender: Sender, callback: js_sys::Function| {
        // let closure = Rc::new(Closure::new(move |msg, sender: Sender, callback: JsValue| -> JsValue {
        let closure = Rc::new(Closure::new(move |msg, sender: Sender, callback: JsValue| -> JsValue {
            let callback = js_sys::Function::from(callback);

            match this.clone().handle_message(msg, sender, callback.clone()) {
                Ok(_) => {
                    JsValue::from(true)
                },
                Err(err) => {
                    log_error!("message handling error: {:?}", err);
                    
                    let response = Response::new(Err(err));
                    if let Err(err) = callback.call1(&JsValue::UNDEFINED,&response.into()) {
                        log_error!("onMessage callback error in error handler: {:?}", err);
                    }
                    JsValue::from(false)
                }
            }
        }));
        // add_listener(closure.clone().as_ref().unwrap());
        add_listener(closure.clone().as_ref());
        *self.closure.lock().unwrap() = Some(closure);

        // server

    }

    // fn handle_message(self : Arc<Self>, msg: JsValue, sender: Sender, callback: JsValue) -> Result<()> {
    fn handle_message(self : Arc<Self>, msg: JsValue, sender: Sender, callback: Function) -> Result<()> {
        if let Some(id) = sender.id() {
log_info!("######## RUNTIME ID: {id}");
            if id != self.runtime_id {
                log_info!("NOT THE SAME SAME RUNTIME ID!!!");
                return Err(Error::custom("Not the same runtime ID..."));
            }
        } else {
            return Err(Error::custom("NO RUNTIME ID..."));
        }

        log_info!(
            "[WASM] msg: {:?}, sender id:{:?}, {:?}",
            msg,
            sender.id(),
            callback
        );


        let Request { op, data } = Request::try_from(msg)?;

        // let op = js_sys::Reflect::get(&msg, &"op".into()).unwrap();
        // let data = js_sys::Reflect::get(&msg, &"data".into()).unwrap();
        // log_info!("op: {:?}, data: {:?}", op, data);

        // let op = op.as_f64().expect("no op property") as u32;
        // let request: Vec<u8> = data.as_string().expect("no data property").from_hex().expect("invalid hex");

        spawn_local(async move {

            let response = Response::new(self.wallet_server.call_with_borsh(op, &data).await);
            if let Err(err) = callback.call1(&JsValue::UNDEFINED,&response.into()) {
                log_error!("onMessage callback error: {:?}", err);
            }
            // match  {
            //     Ok(response) => {
            //         log_info!("response: {:?}", response);

            //         if let Err(err) = callback.call1(&JsValue::UNDEFINED,&JsValue::UNDEFINED) {
            //             log_error!("callback error: {:?}", err);
            //         }
            //     },
            //     Err(err) => {
            //         let err_str = err.to_string();

            //         if let Err(err) = callback.call1(&JsValue::UNDEFINED,&JsValue::UNDEFINED) {
            //             log_error!("callback error: {:?}", err);
            //         }

            //     }
            // }
        });

        Ok(())
    }
}

/*
#[async_trait]
impl Transport for Server {
    async fn call(&self, op: u32, request: &[u8]) -> Result<Vec<u8>> {
        // let msg = Object::new();
        // let data = request.to_hex();
        // js_sys::Reflect::set(&msg, &"op".into(), &JsValue::from(op)).unwrap();
        // js_sys::Reflect::set(&msg, &"data".into(), &JsValue::from(data)).unwrap();
        // self.ipc.send_message(msg);
    }
}
*/
