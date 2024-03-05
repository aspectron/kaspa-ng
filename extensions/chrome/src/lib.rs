pub mod client;
pub mod imports;
pub mod ipc;
pub mod server;

use crate::imports::*;

static mut SERVER: Option<Arc<Server>> = None;
// background script
#[wasm_bindgen]
pub async fn kaspa_ng_background() {
    log_info!("kaspa_ng_background called successfully in the background!");
    workflow_wasm::panic::init_console_panic_hook();

    let server = Arc::new(Server::new().await);
    unsafe {
        SERVER = Some(server.clone());
    }
    server.start().await;
}

// extension popup
#[wasm_bindgen]
pub async fn kaspa_ng_main() {
    log_info!("kaspa_ng_main called successfully in the popup!");
    workflow_wasm::panic::init_console_panic_hook();

    let application_events = ApplicationEventsChannel::unbounded();

    let client_transport = Arc::new(client::ClientTransport::new(application_events.clone()));
    let borsh_transport = Codec::Borsh(client_transport.clone());
    let wallet_client: Arc<dyn WalletApi> = Arc::new(WalletClient::new(borsh_transport));

    log_info!("STARTING CLIENT TRANSPORT");
    client_transport.start();

    let response = wallet_client
        .clone()
        .ping(Some("hello world!".to_string()))
        .await
        .expect("ping failed");
    log_info!("Client received response: {response:?}");

    if let Err(err) = app::kaspa_ng_main(Some(wallet_client), Some(application_events)).await {
        log_error!("Error: {err}");
    }
}
