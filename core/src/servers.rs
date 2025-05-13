use kaspa_wrpc_client::Resolver;

use crate::imports::*;

type ServerCollection = Arc<Mutex<Arc<HashMap<Network, Vec<Server>>>>>;

pub fn public_server_config() -> &'static ServerCollection {
    static SERVERS: OnceLock<ServerCollection> = OnceLock::new();
    SERVERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new().into())))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Server {
    pub id: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_url: Option<String>,
    pub encoding: WrpcEncoding,
    pub network: Network,
    pub online: bool,
    pub status: String,
}

impl Eq for Server {}

impl PartialEq for Server {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

impl std::fmt::Display for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.id, self.url)
    }
}

impl Server {
    pub fn address(&self) -> String {
        self.url.clone()
    }

    pub fn wrpc_encoding(&self) -> WrpcEncoding {
        self.encoding
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    server: Vec<Server>,
}

pub fn update_public_servers() {
    spawn(async move {
        let servers = fetch_public_servers().await?;
        *public_server_config().lock().unwrap() = servers;
        Ok(())
    });
}

pub fn load_public_servers() {
    update_public_servers();
}

async fn get_server_list() -> Result<Vec<Server>> {
    // Get all resolver urls
    if let Some(resolvers) = Resolver::default().urls() {
        // Try to connect to each resolver
        for resolver in resolvers {
            // Retrieve server list
            let server_list =
                workflow_http::get_json::<Vec<Server>>(format!("{}/status", resolver)).await;
            if server_list.is_ok() {
                return Ok(server_list?);
            }
        }
    }

    // If no resolver was able to connect, return an error
    Err(Error::custom("Unable to connect to any resolver"))
}

async fn fetch_public_servers() -> Result<Arc<HashMap<Network, Vec<Server>>>> {
    // Get server list
    let servers = get_server_list().await?;
    // Group servers by network
    let servers = HashMap::group_from(servers.into_iter().map(|server| (server.network, server)));
    Ok(servers.into())
}

pub fn tls() -> bool {
    static TLS: OnceLock<bool> = OnceLock::new();
    *TLS.get_or_init(|| {
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                location().expect("expecting location").protocol().expect("expecting protocol").as_str() == "https:"
            } else {
                false
            }
        }
    })
}

pub fn public_servers(network: &Network) -> Vec<Server> {
    let servers = public_server_config().lock().unwrap().clone();
    let servers = servers.get(network).unwrap();
    servers
        .iter()
        .filter(|server| {
            server.online
                && !(tls()
                    && !(server.url.starts_with("wss://") || server.url.starts_with("wrpcs://")))
        })
        .cloned()
        .collect::<Vec<_>>()
}

pub fn render_public_server_selector(
    core: &mut Core,
    ui: &mut egui::Ui,
    settings: &mut NodeSettings,
) -> Option<&'static str> {
    let mut node_settings_error = None;

    let servers = public_servers(&settings.network);

    ui.add_space(4.);

    let (text, _secondary) = if let Some(server) = settings.public_servers.get(&settings.network) {
        (server.to_string(), Option::<String>::None)
    } else {
        node_settings_error = Some(i18n(
            "No public node selected - please select a public node",
        ));
        (i18n("Select Public Node").to_string(), None)
    };

    let response = ui.add_sized(
        theme_style().large_button_size,
        CompositeButton::opt_image_and_text(None, Some(text.into()), None)
            .with_pulldown_selector(true),
    );

    PopupPanel::new(
        PopupPanel::id(ui, "server_selector_popup"),
        |_ui| response,
        |ui, close| {
            egui::ScrollArea::vertical()
                .id_salt("server_selector_popup_scroll")
                .auto_shrink([true; 2])
                .show(ui, |ui| {
                    let mut first = true;
                    for server in servers {
                        if !first {
                            ui.separator();
                        } else {
                            first = false;
                        }
                        if ui
                            .add_sized(
                                theme_style().large_button_size,
                                CompositeButton::opt_image_and_text(
                                    None,
                                    Some(server.to_string().into()),
                                    None,
                                ),
                            )
                            .clicked()
                        {
                            settings
                                .public_servers
                                .insert(settings.network, server.clone());
                            *close = true;
                        }

                        ui.add_space(4.);
                        ui.hyperlink_url_to_tab(server.url);
                        ui.add_space(4.);
                    }
                });
        },
    )
    .with_min_width(240.)
    .with_max_height(core.device().screen_size.y * 0.5)
    .with_close_button(true)
    .with_padding(false)
    .build(ui);

    ui.add_space(4.);

    node_settings_error
}
