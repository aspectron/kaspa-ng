pub mod client;
pub mod imports;
pub mod server;
pub mod transport;

use crate::imports::*;
use client::*;
use kaspa_ng_core::interop::Client;

static mut SERVER: Option<Arc<Server>> = None;
// background script
#[wasm_bindgen]
pub async fn kaspa_ng_background() {
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

    log_info!("Kaspa NG {} (background)", kaspa_ng_core::app::VERSION);
}

#[wasm_bindgen]
pub async fn kaspa_ng_main() {
    // log_info!("kaspa_ng_main called successfully in the popup!");
    workflow_wasm::panic::init_console_panic_hook();

    let application_events = ApplicationEventsChannel::unbounded();

    let sender = Arc::new(ClientSender::default());
    let client = Arc::new(Client::new(sender.clone(), application_events.clone()));

    let receiver = Arc::new(ClientReceiver::new(
        sender.clone(),
        client.clone(),
        application_events.clone(),
    ));
    receiver.start();
    if let Err(err) = client.clone().init().await {
        log_error!("Error: {err}");
    }

    let borsh_transport = Codec::Borsh(sender.clone());
    let wallet_client: Arc<dyn WalletApi> = Arc::new(WalletClient::new(borsh_transport));

    let application_context = app::ApplicationContext::new(
        Some(wallet_client),
        Some(application_events),
        Some(client.adaptor().clone()),
    );

    if let Err(err) = app::kaspa_ng_main(application_context).await {
        log_error!("Error: {err}");
    }
}
