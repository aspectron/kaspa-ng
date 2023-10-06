pub mod client;
pub mod ipc;
pub mod server;

use crate::server::Server;
use kaspa_ng_core::app;
//use kaspa_wallet_core::rpc::Rpc;
//use kaspa_wallet_core::runtime::api::transport::{Transport, WalletClient};
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use workflow_log::*;

static mut SERVER: Option<Arc<Server>> = None;
// background script
#[wasm_bindgen]
pub async fn kaspa_ng_background() {
    log_info!("kaspa_ng_background called successfully in the background!");

    let server = Arc::new(Server::new().await);
    unsafe {
        SERVER = Some(server.clone());
    }
    server.start();
}

// extension popup
#[wasm_bindgen]
pub async fn kaspa_ng_main() {
    log_info!("kaspa_ng_main called successfully in the popup!");

    // let transport = Transport::Borsh(Arc::new(client::ClientTransport::default()));
    // let wallet_client = Arc::new(WalletClient::new(transport));

    if let Err(err) = app::kaspa_ng_main(None).await {
        log_error!("Error: {err}");
    }
}
