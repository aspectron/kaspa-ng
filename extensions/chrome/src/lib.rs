use kaspa_ng_core::app;
use workflow_log::log_info;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn kaspa_ng_background() {
    log_info!("kaspa_ng_background called successfully in the background!");
}

#[wasm_bindgen]
pub async fn kaspa_ng_main() {
    log_info!("kaspa_ng_main called successfully in the popup!");
    app::kaspa_ng_main().await;    
}

