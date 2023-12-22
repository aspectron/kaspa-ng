use crate::imports::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Server {
    pub name : Option<String>,
    pub location : Option<String>,
    pub protocol : String,
    pub network : Vec<Network>,
    pub port : Option<u16>,
    pub address : String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    server : Vec<Server>,
}

fn try_parse_servers(toml : &str) -> Result<Arc<Vec<Server>>> {
    Ok(toml::from_str::<ServerConfig>(toml)?.server.into())
}

fn parse_servers(toml : &str) -> Arc<Vec<Server>> {
    match try_parse_servers(toml) {
        Ok(servers) => servers,
        Err(e) => {
            cfg_if! {
                if #[cfg(not(target_arch = "wasm32"))] {
                    println!();
                    panic!("Error parsing Servers.toml: {}", e);
                } else {
                    log_error!("Error parsing Servers.toml: {}", e);
                    vec![].into()
                }   
            }         
        }
    }
}

pub fn parse_default_servers() -> &'static Arc<Vec<Server>> {
    static EMBEDDED_SERVERS: OnceLock<Arc<Vec<Server>>> = OnceLock::new();
    EMBEDDED_SERVERS.get_or_init(|| {
        parse_servers(include_str!("../../Servers.toml"))
    })
}

pub async fn load_servers() -> Result<Arc<Vec<Server>>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            use workflow_dom::utils::*;
            let href = window().location().href()?;
            let location = if let Some(index) = href.find('#') {
                let (location, _) = href.split_at(index);
                location.to_string()
            } else {
                href
            };
            let url = format!("{}/Servers.toml", location.trim_end_matches("/"));
            let servers_toml = http::get(url).await?;
            Ok(try_parse_servers(&servers_toml)?)
        } else {
            // TODO - parse local Servers.toml file
            Ok(parse_default_servers().clone())
        }
    }
}
