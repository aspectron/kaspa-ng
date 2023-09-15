#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use cfg_if::cfg_if;
use kaspa_egui::interop;
use kaspa_egui::settings::Settings;
use workflow_log::*;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {

        // When compiling natively:
        #[tokio::main]
        async fn main() -> eframe::Result<()> {
            // use std::sync::Arc;

            use std::sync::{Arc, Mutex};

            interop::panic::init_panic_handler();

            // Log to stderr (if you run with `RUST_LOG=debug`).
            env_logger::init();

            let settings = Settings::load().unwrap_or_else(|err| {
                log_error!("Unable to load settings: {err}");
                Settings::default()
            });

            let interop: Arc<Mutex<Option<interop::Interop>>> = Arc::new(Mutex::new(None));
            let delegate = interop.clone();
            println!("spawn done");
            let native_options = eframe::NativeOptions::default();
            eframe::run_native(
                "DAG Wallet",
                native_options,
                Box::new(move |cc| {
                    let interop = interop::Interop::new(&cc.egui_ctx, &settings);
                    delegate.lock().unwrap().replace(interop.clone());
                    interop::signals::Signals::bind(&interop);
                    interop.start();

                    Box::new(kaspa_egui::Wallet::new(cc, interop, settings))
                }),
            )?;
            println!("exit initiated...");

            let interop = interop.lock().unwrap().take().unwrap();
            println!("wallet shutdown");
            interop.shutdown();
            println!("worker join");
            interop.join().await;
            println!("exit");
            interop.drop();
            Ok(())
        }
    } else {

        // use wasm_bindgen::prelude::*;

        // When compiling to web using trunk:
        // #[cfg(target_arch = "wasm32")]
        // #[wasm_bindgen]
        // fn main() {
        // }

        // #[wasm_bindgen]
        // pub async fn start_app() {
        fn main() {
            use wasm_bindgen::prelude::*;

            // Redirect `log` message to `console.log` and friends:
            eframe::WebLogger::init(log::LevelFilter::Debug).ok();
            let web_options = eframe::WebOptions::default();

            let settings = Settings::unwrap_or_else(|err| {
                log_error!("Unable to load settings: {err}");
                Settings::default()
            });

            wasm_bindgen_futures::spawn_local(async {
                use workflow_log::*;
                log_info!("starting");
                eframe::WebRunner::new()
                    .start(
                        "kaspa-wallet",
                        web_options,
                        Box::new(move |cc| {
                            let interop = interop::Interop::new(&cc.egui_ctx, &settings);
                            interop.start();

                            let adaptor = kaspa_egui::adaptor::Adaptor::new(interop.clone());
                            let window = web_sys::window().expect("no global `window` exists");
                            js_sys::Reflect::set(
                                &window,
                                &JsValue::from_str("adaptor"),
                                &JsValue::from(adaptor),
                            ).expect("failed to set adaptor");

                            Box::new(kaspa_egui::Wallet::new(cc, interop, settings))
                        }),
                    )
                    .await
                    .expect("failed to start eframe");

                // log_info!("shutting down...");
            });

            // wasm_bindgen_futures::spawn_local(async {
            //     // interop.join();

            // });
        }
    }
}
