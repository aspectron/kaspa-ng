use cfg_if::cfg_if;
use kaspa_ng_core::interop;
use kaspa_ng_core::settings::Settings;
use kaspa_wallet_core::api::WalletApi;
use std::sync::Arc;
use workflow_i18n::*;
use workflow_log::*;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        use kaspad_lib::daemon::create_core;
        use kaspad_lib::args::Args as NodeArgs;
        use kaspad_lib::args::parse_args as parse_kaspad_args;
        use kaspa_utils::fd_budget;
        use kaspa_core::signals::Signals;
        // use log::info;
        use kaspa_ng_core::runtime;

        struct KngArgs {
            // is_node: bool,
        }

        enum Args {
            Kng(KngArgs),
            Kaspad(Box<NodeArgs>),
        }

        fn parse_args() -> Args {
            #[allow(unused)]
            use clap::{arg, command, Arg, Command};

            if std::env::var("KASPA_NG_NODE").is_ok() {
                Args::Kaspad(Box::new(parse_kaspad_args()))
            } else {
                let cmd = Command::new("kaspa-ng")
                    // .about(format!("{} (rusty-kaspa) v{}", env!("CARGO_PKG_DESCRIPTION"), version()))
                    .version(env!("CARGO_PKG_VERSION"));
                    // .arg(arg!(--node "Run as a kaspad p2p node."));

                    let _matches = cmd.get_matches();
                    // let is_node = matches.get_one::<bool>("node").cloned().unwrap_or(false);

                    Args::Kng(KngArgs { })
            }
        }

        pub async fn kaspa_ng_main(_wallet_api : Option<Arc<dyn WalletApi>>) -> eframe::Result<()> {

            use std::sync::Mutex;

            runtime::panic::init_panic_handler();

            match parse_args() {
                Args::Kaspad(args) => {
                    let fd_total_budget = fd_budget::limit() - args.rpc_max_clients as i32 - args.inbound_limit as i32 - args.outbound_target as i32;
                    let (core, _) = create_core(*args, fd_total_budget);
                    Arc::new(Signals::new(&core)).init();
                    core.run();
                }

                Args::Kng(_args) => {

                    // Log to stderr (if you run with `RUST_LOG=debug`).
                    env_logger::init();

                    let settings = Settings::load().await.unwrap_or_else(|err| {
                        log_error!("Unable to load settings: {err}");
                        Settings::default()
                    });

                    init_i18n(settings.language.as_str()).expect("failed to init i18n");

                    let interop: Arc<Mutex<Option<interop::Interop>>> = Arc::new(Mutex::new(None));
                    let delegate = interop.clone();
                    println!("spawn done");
                    let native_options = eframe::NativeOptions::default();
                    eframe::run_native(
                        "Kaspa NG",
                        native_options,
                        Box::new(move |cc| {
                            let interop = interop::Interop::new(&cc.egui_ctx, &settings);
                            delegate.lock().unwrap().replace(interop.clone());
                            interop::signals::Signals::bind(&interop);
                            interop.start();

                            Box::new(kaspa_ng_core::Wallet::new(cc, interop, settings))
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

                }
            }

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
        use crate::result::Result;

        pub async fn kaspa_ng_main(_wallet_api : Option<Arc<dyn WalletApi>>) -> Result<()> {
            use wasm_bindgen::prelude::*;

            // ------------------------------------------------------------
            // ------------------------------------------------------------
            // ------------------------------------------------------------
            // log_info!("Sending ping request...");
            // let wallet_api = wallet_api.expect("wallet_api is None");
            // let v = wallet_api.ping(1).await.expect("ping failed");
            // log_info!("Ping response received '{v}' (should be 2) ...");

            // ------------------------------------------------------------
            // ------------------------------------------------------------
            // ------------------------------------------------------------


            // Redirect `log` message to `console.log` and friends:
            eframe::WebLogger::init(log::LevelFilter::Debug).ok();
            let web_options = eframe::WebOptions::default();

            let settings = Settings::load().await.unwrap_or_else(|err| {
                log_error!("Unable to load settings: {err}");
                Settings::default()
            });

            init_i18n(settings.language.as_str()).expect("failed to init i18n");

            // wasm_bindgen_futures::spawn_local(async {
                use workflow_log::*;
                log_info!("starting");
                eframe::WebRunner::new()
                    .start(
                        "kaspa-ng",
                        web_options,
                        Box::new(move |cc| {
                            let interop = interop::Interop::new(&cc.egui_ctx, &settings);
                            interop.start();

                            let adaptor = kaspa_ng_core::adaptor::Adaptor::new(interop.clone());
                            let window = web_sys::window().expect("no global `window` exists");
                            js_sys::Reflect::set(
                                &window,
                                &JsValue::from_str("adaptor"),
                                &JsValue::from(adaptor),
                            ).expect("failed to set adaptor");

                            Box::new(kaspa_ng_core::Wallet::new(cc, interop, settings))
                        }),
                    )
                    .await
                    .expect("failed to start eframe");

                // log_info!("shutting down...");
            // });

            // wasm_bindgen_futures::spawn_local(async {
            //     // interop.join();

            // });

            Ok(())
        }
    }
}
