use crate::imports::*;

#[derive(PartialEq, Eq)]
pub struct StorageFolder {
    pub path: PathBuf,
    pub network: Network,
    pub name: String,
    pub folder_size: u64,
    pub folder_size_string: String,
    pub confirm_deletion: bool,
}

impl Ord for StorageFolder {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for StorageFolder {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Default, Debug, Clone)]
pub struct StorageUpdateOptions {
    pub update_if_not_present: bool,
    pub network: Option<Network>,
    pub delay: Option<Duration>,
}

impl StorageUpdateOptions {
    pub fn if_not_present(mut self) -> Self {
        self.update_if_not_present = true;
        self
    }

    pub fn with_network(mut self, network: Network) -> Self {
        self.network = Some(network);
        self
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = Some(delay);
        self
    }
}

#[derive(Default, Clone)]
pub struct Storage {
    pub folders: Arc<Mutex<Vec<StorageFolder>>>,
    pub storage_root: Arc<Mutex<Option<PathBuf>>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Storage {
    pub fn track_storage_root(&self, storage_root: Option<&str>) {
        *self.storage_root.lock().unwrap() = storage_root.map(PathBuf::from);
        self.update(None);
    }

    pub fn storage_root(&self) -> PathBuf {
        self.storage_root
            .lock()
            .unwrap()
            .clone()
            .or_else(|| Some(kaspad_lib::daemon::get_app_dir()))
            .unwrap()
    }

    pub fn update(&self, options: Option<StorageUpdateOptions>) {
        let options = options.unwrap_or_default();

        if options.update_if_not_present {
            if let Some(network) = options.network {
                if self.has_network(network) {
                    return;
                }
            }
        }

        let rusty_kaspa_app_dir = self.storage_root();
        if !rusty_kaspa_app_dir.exists() {
            return;
        }

        let this = self.clone();
        spawn(async move {
            if let Some(delay) = options.delay {
                task::sleep(delay).await;
            }

            let paths = std::fs::read_dir(rusty_kaspa_app_dir).unwrap();
            for path in paths {
                let path = path?.path();
                if std::fs::metadata(&path)?.is_dir() {
                    if let Some(folder) = path.clone().file_name().and_then(|path| path.to_str()) {
                        if !folder.starts_with('.') {
                            if let Some(network) = folder
                                .strip_prefix("kaspa-")
                                .and_then(|folder| folder.parse::<Network>().ok())
                            {
                                let mut folder_size = 0;
                                for entry in walkdir::WalkDir::new(&path).into_iter().flatten() {
                                    folder_size += entry
                                        .metadata()
                                        .map(|metadata| metadata.len())
                                        .unwrap_or_default();
                                }

                                this.update_folder_size(network, folder_size, path);
                            }
                        }
                    }
                }
            }

            runtime().request_repaint();

            Ok(())
        });
    }

    fn update_folder_size(&self, network: Network, folder_size: u64, path: PathBuf) {
        use kaspa_metrics_core::data::as_data_size;
        let folder_size_string = as_data_size(folder_size as f64, true);

        let mut folders = self.folders.lock().unwrap();
        if let Some(folder) = folders.iter_mut().find(|folder| folder.network == network) {
            folder.folder_size = folder_size;
            folder.folder_size_string = folder_size_string;
        } else {
            folders.push(StorageFolder {
                name: network.to_string().to_uppercase(),
                path,
                network,
                folder_size,
                folder_size_string,
                confirm_deletion: false,
            });

            folders.sort();
        }
    }

    pub fn has_network(&self, network: Network) -> bool {
        self.folders
            .lock()
            .unwrap()
            .iter()
            .any(|folder| folder.network == network)
    }

    pub fn folder(&self, network: Network) -> Option<PathBuf> {
        self.folders
            .lock()
            .unwrap()
            .iter()
            .find(|folder| folder.network == network)
            .map(|folder| folder.path.clone())
    }

    pub fn remove(&self, network: Network) {
        let this = self.clone();
        spawn(async move {
            if let Some(path) = this.folder(network) {
                if path.exists() {
                    println!("Removing storage folder: {:?}", path.display());
                    if let Err(e) = std::fs::remove_dir_all(&path) {
                        println!("Error removing storage folder: {:?}", e);
                        runtime().error(format!("Error removing storage folder: {:?}", e));
                    }
                    println!("Storage folder removed: {:?}", path.display());
                    this.update(None);
                } else {
                    runtime().error(format!("Folder not found: {}", path.display()));
                }
            }
            Ok(())
        });
    }

    pub fn render(&self, ui: &mut Ui) {
        let folders = self.folders.lock().unwrap();
        if !folders.is_empty() {
            ui.vertical_centered(|ui| {
                CollapsingHeader::new(i18n("Storage"))
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            for folder in folders.iter() {
                                let StorageFolder {
                                    network,
                                    folder_size_string,
                                    ..
                                } = folder;
                                ui.label(format!(
                                    "{}: {folder_size_string}",
                                    network.to_string().to_uppercase()
                                ));
                            }
                        });
                    });
            });
        }
    }

    pub fn clear_settings(&self) {
        let mut folders = self.folders.lock().unwrap();
        for folder in folders.iter_mut() {
            folder.confirm_deletion = false;
        }
    }

    pub fn render_settings(&self, core: &mut Core, ui: &mut Ui) {
        let mut folders = self.folders.lock().unwrap();
        if !folders.is_empty() {
            ui.vertical_centered(|ui| {
                CollapsingHeader::new(i18n("Storage"))
                .default_open(false)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        for folder in folders.iter_mut() {
                            let StorageFolder { network, folder_size_string, path, confirm_deletion, .. } = folder;

                            CollapsingHeader::new(format!("{}: {folder_size_string}", network.to_string().to_uppercase()))
                            .default_open(false)
                            .show(ui, |ui| {
                                let is_running = core.settings.node.network == *network && core.settings.node.node_kind.is_local();

                                ui.horizontal(|ui|{
                                    if ui.medium_button(i18n("Open Data Folder")).clicked() {
                                        if let Err(err) = open::that(path) {
                                            runtime().error(format!("Error opening folder: {:?}", err));
                                        }
                                    }
                                    if ui.medium_button_enabled(!is_running && !*confirm_deletion, i18n("Delete Data Folder")).clicked() {
                                        *confirm_deletion = true;
                                    }
                                });

                                if is_running {
                                    ui.label(i18n("Cannot delete data folder while the node is running"));
                                    ui.label(i18n("Please set node to 'Disabled' to delete the data folder"));
                                }

                                if *confirm_deletion {
                                    ui.add_sized(vec2(260.,4.), Separator::default());
                                    ui.label(i18n("This action will erase Kaspa database and logs"));
                                    ui.label("");
                                    ui.colored_label(theme_color().alert_color, i18n("Please Confirm Deletion"));
                                    if let Some(response) = ui.confirm_medium_apply_cancel(Align::Min) {
                                        match response {
                                            Confirm::Ack => {
                                                *confirm_deletion = false;
                                                self.remove(*network);
                                            },
                                            Confirm::Nack => {
                                                *confirm_deletion = false;
                                            }
                                        }
                                    }
                                    ui.add_sized(vec2(260.,4.), Separator::default());
                                }
                            });
                        }
                    });
                });
            });
        }
    }
}
