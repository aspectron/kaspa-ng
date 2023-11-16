use crate::result::Result;
use cfg_if::cfg_if;
use kaspa_ng_core::interop;
use kaspa_ng_core::settings::Settings;
use kaspa_wallet_core::api::WalletApi;
use std::sync::Arc;
use workflow_i18n::*;
use workflow_log::*;

#[allow(unused)]
pub const KASPA_NG_ICON_256X256: &[u8] = include_bytes!("../../resources/icons/icon-256.png");
pub const I18N_EMBEDDED: &str = include_str!("../../resources/i18n/i18n.json");

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        use kaspad_lib::daemon::create_core;
        use kaspad_lib::args::Args as NodeArgs;
        use kaspad_lib::args::parse_args as parse_kaspad_args;
        use kaspa_utils::fd_budget;
        use kaspa_core::signals::Signals;
        use clap::ArgAction;
        use eframe::IconData;
        use crate::utils::*;
        use std::fs;

        #[derive(Debug)]
        enum I18n {

            Import,
            Export,
        }

        enum Args {
            I18n { op : I18n },
            Cli,
            Kng {
                reset_settings : bool,
                disable : bool,
            },
            Kaspad { args : Box<NodeArgs> },
        }

        fn parse_args() -> Args {
            #[allow(unused)]
            use clap::{arg, command, Arg, Command};

            if std::env::var("KASPA_NG_NODE").is_ok() {
                Args::Kaspad { args : Box::new(parse_kaspad_args()) }
            } else {
                let cmd = Command::new("kaspa-ng")

                    .about(format!("kaspa-ng v{} (rusty-kaspa v{})", env!("CARGO_PKG_VERSION"), kaspa_wallet_core::version()))
                    // .version(env!("CARGO_PKG_VERSION"))
                    .arg(arg!(--disable "Disable node services when starting"))
                    .arg(
                        Arg::new("reset-settings")
                        .long("reset-settings")
                        .action(ArgAction::SetTrue)
                        .help("Reset kaspa-ng settings")
                    )

                    .subcommand(
                        Command::new("i18n").hide(true)
                        .about("kaspa-ng i18n user interface translation")
                        .subcommand(
                            Command::new("import")
                            // .help("import i18n data")
                            .about("import JSON files suffixed with language codes (*_en.json, *_de.json, etc.)")
                        )
                        .subcommand(
                            Command::new("export")
                            .about("export default 'en' translations as JSON")
                        )
                    )

                    .subcommand(
                        Command::new("cli")
                            .about("Run kaspa-ng as rusty-kaspa cli wallet")
                    )
                    ;

                    let matches = cmd.get_matches();
                    // println!("matches: {:#?}", matches);

                    if matches.subcommand_matches("cli").is_some() {
                        Args::Cli
                    } else if let Some(matches) = matches.subcommand_matches("i18n") {
                        if let Some(_matches) = matches.subcommand_matches("import") {
                            Args::I18n { op : I18n::Import }
                        } else if let Some(_matches) = matches.subcommand_matches("export") {
                            Args::I18n { op : I18n::Export }
                        } else {
                            panic!("unknown i18n subcommand")
                        }
                    } else {
                        let reset_settings = matches.get_one::<bool>("reset-settings").cloned().unwrap_or(false);
                        let disable = matches.get_one::<bool>("disable").cloned().unwrap_or(false);

                        Args::Kng { reset_settings, disable }
                    }
            }
        }

        pub async fn kaspa_ng_main(_wallet_api : Option<Arc<dyn WalletApi>>) -> Result<()> {

            use std::sync::Mutex;

            interop::panic::init_panic_handler();

            match parse_args() {
                Args::Cli => {
                    use kaspa_cli_lib::*;
                    let result = kaspa_cli(TerminalOptions::new().with_prompt("$ "), None).await;
                    if let Err(err) = result {
                        println!("{err}");
                    }
                }
                Args::Kaspad{ args } => {
                    let fd_total_budget = fd_budget::limit() - args.rpc_max_clients as i32 - args.inbound_limit as i32 - args.outbound_target as i32;
                    let (core, _) = create_core(*args, fd_total_budget);
                    Arc::new(Signals::new(&core)).init();
                    core.run();
                }

                Args::I18n {
                    op
                } => {
                    let i18n_json_file = i18n_storage_file()?;
                    let i18n_json_file_store = i18n_storage_file()?;
                    i18n::Builder::new("en", "en")
                        .with_static_json_data(I18N_EMBEDDED)
                        .with_string_json_data(i18n_json_file.exists().then(move ||{
                            fs::read_to_string(i18n_json_file)
                        }).transpose()?)
                        .with_store(move |json_data: &str| {
                            Ok(fs::write(&i18n_json_file_store, json_data)?)
                        })
                        .try_init()?;

                    match op {
                        I18n::Import => {
                            let source_folder = i18n_storage_folder()?;
                            println!("importing translation files from: '{}'", source_folder.display());
                            i18n::import_translation_files(source_folder,false)?;
                        }
                        I18n::Export => {
                            let mut target_folder = if let Some(cwd) = try_cwd_repo_root()? {
                                cwd.join("resources").join("i18n")
                            } else {
                                std::env::current_dir()?
                            };
                            target_folder.push("kaspa-ng_en.json");
                            println!("exporting default language to: '{}'", target_folder.display());
                            i18n::export_default_language(move |json_data: &str| {
                                Ok(fs::write(&target_folder, json_data)?)
                            })?;
                        }
                    }
                }

                Args::Kng { reset_settings, disable } => {

                    println!("kaspa-ng v{} (rusty-kaspa v{})", env!("CARGO_PKG_VERSION"), kaspa_wallet_core::version());

                    // Log to stderr (if you run with `RUST_LOG=debug`).
                    env_logger::init();

                    let mut settings = if reset_settings {
                        println!("Resetting kaspa-ng settings on user request...");
                        Settings::default().store_sync()?.clone()
                    } else {
                        Settings::load().await.unwrap_or_else(|err| {
                            log_error!("Unable to load settings: {err}");
                            Settings::default()
                        })
                    };

                    println!("settings: {:#?}", settings);

                    let i18n_json_file = i18n_storage_file()?;
                    let i18n_json_file_load = i18n_json_file.clone();
                    let i18n_json_file_store = i18n_json_file.clone();
                    i18n::Builder::new(settings.language_code.as_str(), "en")
                        .with_static_json_data(I18N_EMBEDDED)
                        .with_string_json_data(i18n_json_file.exists().then(move ||{
                            fs::read_to_string(i18n_json_file_load)
                        }).transpose()?)
                        .with_store(move |json_data: &str| {
                            Ok(fs::write(&i18n_json_file_store, json_data)?)
                        })
                        .try_init()?;

                    if disable {
                        settings.node.node_kind = kaspa_ng_core::settings::KaspadNodeKind::Disable;
                    }

                    let interop: Arc<Mutex<Option<interop::Interop>>> = Arc::new(Mutex::new(None));
                    let delegate = interop.clone();
                    // println!("spawn done");
                    let native_options = eframe::NativeOptions {
                        icon_data : IconData::try_from_png_bytes(KASPA_NG_ICON_256X256).ok(),
                        persist_window : true,
                        // initial_window_size : Some(egui::Vec2 { x : 1000.0, y : 600.0 }),
                        // min_window_size : Some(egui::Vec2 { x : 1000.0, y : 600.0 }),
                        ..Default::default()
                    };
                    eframe::run_native(
                        "Kaspa NG",
                        native_options,
                        Box::new(move |cc| {
                            let interop = interop::Interop::new(&cc.egui_ctx, &settings);
                            delegate.lock().unwrap().replace(interop.clone());
                            interop::signals::Signals::bind(&interop);
                            interop.start();

                            Box::new(kaspa_ng_core::Core::new(cc, interop, settings))
                        }),
                    )?;

                    let interop = interop.lock().unwrap().take().unwrap();
                    interop.shutdown().await;
                }
            }

            Ok(())
        }
    } else {

        // use crate::result::Result;

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

            // init_i18n(settings.language_code.as_str()).expect("failed to init i18n");

            i18n::Builder::new(settings.language_code.as_str(), "en")
                .with_static_json_data(I18N_EMBEDDED)
                // .with_store(move |json_data: &str| {
                // })
                .try_init()?;

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

                        Box::new(kaspa_ng_core::Core::new(cc, interop, settings))
                    }),
                )
                .await
                .expect("failed to start eframe");

                // log_info!("shutting down...");
            // });

            Ok(())
        }
    }
}
