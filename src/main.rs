#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use kaspa_egui::interop;


// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // use std::sync::Arc;

    use std::sync::{Arc,Mutex};

    // use egui::mutex::Mutex;

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    // let interop = interop::Interop::new();
    // interop::signals::Signals::bind(&interop);

    let interop : Arc<Mutex<Option<interop::Interop>>> = Arc::new(Mutex::new(None));
    // let delegate = interop.clone();
    let delegate = interop.clone();
    println!("spawn done");
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "DAG Wallet",
        native_options,
        Box::new(move |cc| {

            let interop = interop::Interop::new(&cc.egui_ctx);
            delegate.lock().unwrap().replace(interop.clone());
            interop::signals::Signals::bind(&interop);
            interop.spawn();

            Box::new(kaspa_egui::Wallet::new(cc, interop))
        }),
    )?;

    let interop = interop.lock().unwrap().take().unwrap();
    println!("wallet shutdown");
    interop.shutdown();
    println!("worker join");
    interop.join();
    println!("exit");

    Ok(())
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    
    wasm_bindgen_futures::spawn_local(async {
        use workflow_log::*;    
        log_info!("starting");
        eframe::WebRunner::new()
            .start(
                "kaspa-wallet",
                web_options,
                Box::new(move |cc| {
                
                    let interop = interop::Interop::new(&cc.egui_ctx);
                    interop.spawn();
                
                    Box::new(kaspa_egui::Wallet::new(cc, interop))
                
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
