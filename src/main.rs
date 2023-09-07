#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use kaspa_egui::interop;


// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // use std::sync::Arc;

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let interop = interop::Interop::new();
    let delegate = interop.clone();
    println!("spawn done");
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Kaspa Flow",
        native_options,
        Box::new(move |cc| Box::new(kaspa_egui::Wallet::new(cc, delegate))),
    )?;

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
        let interop = interop::Interop::new();
        interop.spawn();
        let delegate = interop.clone();
log_info!("starting");
        eframe::WebRunner::new()
            .start(
                "kaspa-wallet",
                web_options,
                Box::new(move |cc| Box::new(kaspa_egui::Wallet::new(cc, delegate))),
            )
            .await
            .expect("failed to start eframe");

        // log_info!("shutting down...");
    });

    // wasm_bindgen_futures::spawn_local(async {
    //     // interop.join();

    // });

}
