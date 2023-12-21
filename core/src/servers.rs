use crate::imports::*;
use serde_json::Value;

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

// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct Record {
//     pub name : Option<String>,
//     pub location : Option<String>,
//     pub protocol : String,
//     pub network : Network,
//     pub port : Option<u16>,
// }

// impl From<(String, Record)> for Server {
//     fn from((address, record) : (String, Record)) -> Self {
//         Self {
//             name : record.name,
//             location : record.location,
//             protocol : record.protocol,
//             network : record.network,
//             port : record.port,
//             address,
//             // uri : record.uri,
//         }
//     }
// }

// impl Server {
//     pub fn new(name : &str, location : &str, protocol : &str, network : &Network, uri : &str) -> Self {
//         Self {
//             name : Some(name.to_string()),
//             location : Some(location.to_string()),
//             protocol : protocol.to_string(),
//             network : network.clone(),
//             // uri : uri.to_string(),
//         }
//     }
// }

fn parse_servers_impl(toml : &str) -> Result<Arc<Vec<Server>>> {
    // let table: Value = toml::from_str(&toml)?;

    // println!("table: {:?}", table);
    // panic!();

    Ok(toml::from_str::<ServerConfig>(toml)?.server.into())
    
    
}


fn parse_servers(toml : &str) -> Arc<Vec<Server>> {
    match parse_servers_impl(toml) {
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
    let servers = parse_default_servers();
println!("servers: {:?}", servers);
    Ok(servers.clone())
}
