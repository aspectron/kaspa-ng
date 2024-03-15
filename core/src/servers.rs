use crate::imports::*;

type ServerCollection = Arc<Mutex<Arc<HashMap<Network, Vec<Server>>>>>;

pub fn public_server_config() -> &'static ServerCollection {
    static SERVERS: OnceLock<ServerCollection> = OnceLock::new();
    SERVERS.get_or_init(|| Arc::new(Mutex::new(parse_default_servers().clone())))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Server {
    pub name: Option<String>,
    pub ident: Option<String>,
    pub location: Option<String>,
    pub protocol: WrpcEncoding,
    pub address: String,
    pub enable: Option<bool>,
    pub link: Option<String>,
    pub network: Network,
    pub bias: Option<f32>,
    pub manual: Option<bool>,
    pub version: Option<String>,
}

impl Eq for Server {}

impl PartialEq for Server {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}

impl std::fmt::Display for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut title = self.name.clone().unwrap_or(self.address.to_string());
        if let Some(ident) = self.ident.as_ref() {
            title += format!(" ({ident})").as_str();
        } else if let Some(location) = self.location.as_ref() {
            title += format!(" ({location})").as_str();
        }

        write!(f, "{}", title)
    }
}

impl Server {
    pub fn address(&self) -> String {
        self.address.clone()
    }

    pub fn wrpc_encoding(&self) -> WrpcEncoding {
        self.protocol
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    server: Vec<Server>,
}

fn try_parse_servers(toml: &str) -> Result<Arc<HashMap<Network, Vec<Server>>>> {
    let servers: Vec<Server> = toml::from_str::<ServerConfig>(toml)?
        .server
        .into_iter()
        .filter(|server| server.enable.unwrap_or(true))
        .collect::<Vec<_>>();

    let servers = HashMap::group_from(servers.into_iter().map(|server| (server.network, server)));

    Ok(servers.into())
}

// fn parse_servers(toml: &str) -> Arc<Vec<Server>> {
fn parse_servers(toml: &str) -> Arc<HashMap<Network, Vec<Server>>> {
    match try_parse_servers(toml) {
        Ok(servers) => servers,
        Err(e) => {
            cfg_if! {
                if #[cfg(not(target_arch = "wasm32"))] {
                    println!();
                    panic!("Error parsing Servers.toml: {}", e);
                } else {
                    log_error!("Error parsing Servers.toml: {}", e);
                    HashMap::default().into()
                }
            }
        }
    }
}

pub fn parse_default_servers() -> &'static Arc<HashMap<Network, Vec<Server>>> {
    static EMBEDDED_SERVERS: OnceLock<Arc<HashMap<Network, Vec<Server>>>> = OnceLock::new();
    EMBEDDED_SERVERS.get_or_init(|| parse_servers(include_str!("../resources/Servers.toml")))
}

pub fn update_public_servers() {
    spawn(async move {
        let servers = fetch_public_servers().await?;
        *public_server_config().lock().unwrap() = servers;
        Ok(())
    });
}

pub fn load_public_servers() {
    parse_default_servers();
    update_public_servers();
}

async fn fetch_public_servers() -> Result<Arc<HashMap<Network, Vec<Server>>>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let href = location()?.href()?;
            let location = if let Some(index) = href.find('#') {
                let (location, _) = href.split_at(index);
                location.to_string()
            } else {
                href
            };
            let url = format!("{}/Servers.toml", location.trim_end_matches('/'));
            let servers_toml = http::get(url).await?;
            try_parse_servers(&servers_toml)
        } else {
            // TODO - parse local Servers.toml file
            Ok(parse_default_servers().clone())
        }
    }
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
            server.enable.unwrap_or(true)
                && !(tls()
                    && !(server.address.starts_with("wss://")
                        || server.address.starts_with("wrpcs://")))
        })
        .cloned()
        .collect::<Vec<_>>()
}

pub fn random_public_server(network: &Network, options: Option<RpcOptions>) -> Option<Server> {
    let servers = public_server_config().lock().unwrap().clone();

    let RpcOptions {
        force_server,
        blacklist_servers,
    } = options.unwrap_or_default();

    if let Some(server) = force_server.clone() {
        return Some(server);
    }

    if let Some(servers) = servers.get(network) {
        #[allow(clippy::nonminimal_bool)]
        let mut servers = servers
            .iter()
            .filter(|server| {
                server.enable.unwrap_or(true)
                    && !server.manual.unwrap_or(false)
                    && !server.address.contains("localhost")
                    && !server.address.contains("127.0.0.1")
                    && !(tls()
                        && !(server.address.starts_with("wss://")
                            || server.address.starts_with("wrpcs://")))
                    && !blacklist_servers.contains(&server.address)
            })
            .cloned()
            .collect::<Vec<_>>();

        let max = servers
            .iter()
            .map(|server| server.bias.unwrap_or(1.))
            .max_by(|a, b| a.total_cmp(b))
            .unwrap_or(1.);
        servers.iter_mut().for_each(|server| {
            server.bias = Some(server.bias.unwrap_or(1.) / max);
        });

        if servers.is_empty() {
            log_error!("Unable to select random public server: no servers available");
            None
        } else {
            let mut server = None;
            while server.is_none() {
                let selected = &servers[rand::thread_rng().gen::<usize>() % servers.len()];
                let f = rand::thread_rng().gen_range(0.0..1.0);
                if f < selected.bias.unwrap_or(1.) {
                    server = Some(selected.clone());
                }
            }
            server
        }
    } else {
        log_error!("Unable to select random public server: no servers available for this network");
        None
    }
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
                .id_source("server_selector_popup_scroll")
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

                        if let Some(link) = server.link.as_ref() {
                            ui.add_space(4.);
                            ui.hyperlink_url_to_tab(link);
                            ui.add_space(4.);
                        }
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
