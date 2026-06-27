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

            // Install the pure-Rust `ring` rustls crypto provider before any TLS
            // (workflow-http/reqwest or the wRPC client). reqwest 0.13 ships with
            // `rustls-no-provider`, so a provider must be installed explicitly;
            // `install_default` is idempotent (no-op if already set).
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
