use crate::ipc::*;
use async_trait::async_trait;
use kaspa_wallet_core::error::Error;
use kaspa_wallet_core::result::Result;
//use kaspa_wallet_core::runtime::api::transport::*;
use wasm_bindgen_futures::spawn_local;
use workflow_core::channel::oneshot;
use workflow_log::*;

#[derive(Default)]
pub struct ClientTransport {}

unsafe impl Send for ClientTransport {}
unsafe impl Sync for ClientTransport {}

impl ClientTransport {
    // pub fn new() -> Self {
    //     Self {}
    // }
}

// #[async_trait]
// impl BorshTransport for ClientTransport {
//     async fn call(&self, op: u64, data: Vec<u8>) -> Result<Vec<u8>> {
//         let (tx, rx) = oneshot::<Result<Vec<u8>>>();
//         spawn_local(async move {
//             match send_message(&req_to_jsv(Target::Wallet, op, &data)).await {
//                 Ok(jsv) => {
//                     let resp = jsv_to_resp(&jsv);
//                     tx.send(resp).await.unwrap();
//                 }
//                 Err(err) => {
//                     log_error!("error sending message: {err:?}");
//                     tx.send(Err(err.into())).await.unwrap();
//                 }
//             };
//         });
//         rx.recv()
//             .await
//             .map_err(|_| Error::custom("Client transport receive channel error"))?
//     }
// }
