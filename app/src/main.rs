#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use cfg_if::cfg_if;
use kaspa_ng_core::app::kaspa_ng_main;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {

        #[tokio::main]
        async fn main() {
            match kaspa_ng_main().await {
                Ok(_) => {},
                Err(err) => {
                    workflow_log::log_error!("Error: {err}");
                }
            }
        }

    } else {

        fn main() {

            wasm_bindgen_futures::spawn_local(async {
                kaspa_ng_main(None).await;
            });

        }
    }
}
