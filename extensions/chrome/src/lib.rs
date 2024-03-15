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

    chrome_runtime_scripting::unregister_content_scripts(None).await;

    let script = RegisteredContentScript::new();
    script.set_id("kaspa-wallet-ext-content-script".to_string());
    script.set_js(vec!["content-script.js"]);
    script.set_persist_across_sessions(false);
    script.set_matches(vec!["https://*/*", "http://*/*"]);
    script.set_run_at("document_end".to_string());
    script.set_all_frames(false);
    script.set_world("ISOLATED".to_string());

    chrome_runtime_scripting::register_content_scripts(vec![script]).await;

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
