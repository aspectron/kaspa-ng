#![warn(clippy::all, rust_2018_idioms)]
// hide console window on Windows in release mode
#![cfg_attr(
    all(not(debug_assertions), not(feature = "console")),
    windows_subsystem = "windows"
)]

use cfg_if::cfg_if;
use kaspa_ng_core::app::{ApplicationContext, kaspa_ng_main};
use workflow_log::*;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {

        fn main() {

            #[cfg(feature = "console")] {
                unsafe {
                    std::env::set_var("RUST_BACKTRACE", "full");
                }
            }

            kaspa_alloc::init_allocator_with_default_settings();

            // rustls 0.23 cannot auto-select a crypto provider when more than one
            // is compiled in. reqwest 0.13 pulls `aws-lc-rs` while the wRPC/
            // websocket (tungstenite) stack pulls `ring`, so auto-selection panics
            // ("Could not automatically determine the process-level CryptoProvider").
            // Install `ring` explicitly so reqwest HTTPS and the wRPC client agree.
            // `install_default` is idempotent here; ignore the error if a provider
            // was already installed.
            let _ = rustls::crypto::ring::default_provider().install_default();

            let body = async {
                if let Err(err) = kaspa_ng_main(ApplicationContext::default()).await {
                    log_error!("Error: {err}");
                }
            };

            #[allow(clippy::expect_used, clippy::diverging_sub_expression)]
            //{
                tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Failed building the Runtime")
                    .block_on(body);
            //};

            #[cfg(feature = "console")]
            {
                println!("Press Enter to exit...");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).expect("Failed to read line");
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
