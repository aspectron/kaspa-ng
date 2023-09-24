// use std::sync::{Arc,Mutex};
// use std::rc::Rc;

// use kaspa_utils::hex::*;
use async_trait::async_trait;
// use kaspa_wallet_core::runtime;//::Wallet;
// use kaspa_wallet_core::runtime::api::*;
use kaspa_wallet_core::runtime::api::transport::*;
use kaspa_wallet_core::result::Result;
use kaspa_wallet_core::error::Error;
// use js_sys::Object;
use crate::ipc::*;
use wasm_bindgen_futures::spawn_local;
// use wasm_bindgen::prelude::*;
use workflow_core::channel::oneshot;
use workflow_log::*;


pub struct ClientTransport {
    // pub ipc : Ipc,
}

unsafe impl Send for ClientTransport {}
unsafe impl Sync for ClientTransport {}

impl ClientTransport {
    pub fn new() -> Self {
        Self {
            // ipc: Ipc::new(),
        }
    }

    // pub fn as_wallet_api(self: &Arc<Self>) -> Arc<dyn WalletApi> {
    //     Arc::new(self.clone())
    // }
}

#[async_trait]
impl BorshTransport for ClientTransport {
    async fn call(&self, op: u32, data: Vec<u8>) -> Result<Vec<u8>> {
        // let data = request.to_hex();
        // let request = Request::serialize(op,data);
        // let data = data.to_vec();
        let (tx,rx) = oneshot::<Result<Response>>();
        
        spawn_local(async move {
            
            let request = Request::new(op,data);
            // let request = Request::to_jsv(op,&data);
            match send_message(&request.into()).await {
                Ok(response) => {
                    let response = Response::try_from(response).unwrap();
                    tx.send(Ok(response)).await.unwrap();
                    
                },
                Err(err) => {
                    log_error!("error sending message: {err:?}");
                    tx.send(Err(err.into())).await.unwrap();
                }
            };

        });

        let response : Result<Response> = rx.recv().await.map_err(|_|Error::custom("Client transport receive channel error"))?;
        response?.into()

    }

}