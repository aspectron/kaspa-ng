#![warn(clippy::all, rust_2018_idioms)]
// hide console window on Windows in release mode
#![cfg_attr(
    all(not(debug_assertions), feature = "console"),
    windows_subsystem = "windows"
)]

use cfg_if::cfg_if;
use kaspa_ng_core::app::{kaspa_ng_main, ApplicationContext};
use workflow_log::*;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {

        fn main() {

            kaspa_alloc::init_allocator_with_default_settings();

            let body = async {
                if let Err(err) = kaspa_ng_main(ApplicationContext::default()).await {
                    log_error!("Error: {err}");
                }
            };

            #[allow(clippy::expect_used, clippy::diverging_sub_expression)]
            {
                return tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Failed building the Runtime")
                    .block_on(body);
            }
        }

    } else {

        fn main() {

            wasm_bindgen_futures::spawn_local(async {
                if let Err(err) = kaspa_ng_main(ApplicationContext::default()).await {
                    log_error!("Error: {err}");
                }
            });

        }
    }
}
