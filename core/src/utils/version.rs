use crate::imports::*;
use workflow_http::Request;

pub fn kaspa_version() -> String {
    kaspa_utils::git::version()
}

#[derive(Debug, Clone)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
}

#[derive(Debug, Clone)]
pub struct Release {
    pub version: String,
    pub timestamp: Option<String>,
    pub html_url: Option<String>,
    pub assets: Vec<Asset>,
}

pub async fn check_version() -> Result<()> {
    let current_version = crate::app::VERSION;

    let url = "https://api.github.com/repos/aspectron/kaspa-ng/releases/latest";
    let response = Request::new(url)
        .with_user_agent(format!("kaspa-ng {current_version} software update check"))
        .get_json::<serde_json::Value>()
        .await;
    match response {
        Ok(data) => {
            let latest_version = data["tag_name"]
                .as_str()
                .ok_or(Error::custom("No tag_name found"))?;
            if latest_version != current_version {
                let timestamp = data["published_at"].as_str();
                let html_url = data["html_url"].as_str();
                let assets = data["assets"]
                    .as_array()
                    .ok_or(Error::custom("No assets found"))?;
                let mut assets = assets
                    .iter()
                    .filter_map(|asset| {
                        if let (Some(name), Some(browser_download_url)) = (
                            asset["name"].as_str(),
                            asset["browser_download_url"].as_str(),
                        ) {
                            Some(Asset {
                                name: name.to_string(),
                                browser_download_url: browser_download_url.to_string(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<Asset>>();

                assets.sort_by_key(|asset| !platform_match(&asset.name));

                let release = Release {
                    version: latest_version.to_string(),
                    timestamp: timestamp.map(String::from),
                    html_url: html_url.map(String::from),
                    assets,
                };

                runtime()
                    .application_events()
                    .send(Events::VersionUpdate(release))
                    .await
                    .unwrap();
            }
        }
        Err(err) => {
            println!("Unable to check for software update with GitHub: {:?}", err);
        }
    }
    Ok(())
}

pub fn platform_match(name: impl Into<String>) -> bool {
    let name: String = name.into();

    Path::new(name.as_str())
        .file_stem()
        .and_then(|file_stem| file_stem.to_str())
        .and_then(|file_stem| file_stem.split('-').collect::<Vec<&str>>().last().cloned())
        .map(|suffix| {
            cfg_if! {
                if #[cfg(target_os = "macos")] {
                    suffix.contains("osx") || suffix.contains("macos")
                } else if #[cfg(target_os = "windows")] {
                    suffix.contains("win")
                } else if #[cfg(target_os = "linux")] {
                    suffix.contains("linux")
                } else {
                    suffix.contains("wasm")
                }
            }
        })
        .unwrap_or_default()
}

pub fn release_link_name(name: impl Into<String>) -> impl Into<WidgetText> {
    let name: String = name.into();

    let matches = Path::new(name.as_str())
        .file_stem()
        .and_then(|file_stem| file_stem.to_str())
        .and_then(|file_stem| file_stem.split('-').collect::<Vec<&str>>().last().cloned())
        .map(|_suffix| {
            cfg_if! {
                if #[cfg(target_os = "macos")] {
                    _suffix.contains("osx") || _suffix.contains("macos")
                } else if #[cfg(target_os = "windows")] {
                    _suffix.contains("win")
                } else if #[cfg(target_os = "linux")] {
                    _suffix.contains("linux")
                } else {
                    false
                }
            }
        })
        .unwrap_or_default();

    if matches {
        RichText::new(format!("• {name}")).strong()
    } else {
        RichText::new(format!("• {name}"))
    }
}

pub fn is_version_greater(current: &str, update: &str) -> Result<bool> {
    let current = current
        .split('.')
        .map(|part| part.parse().map_err(Into::into))
        .collect::<Result<Vec<u64>>>()?;
    let update = update
        .split('.')
        .map(|part| part.parse().map_err(Into::into))
        .collect::<Result<Vec<u64>>>()?;

    let current = current.iter().fold(0, |acc, &x| acc * 1000 + x);
    let update = update.iter().fold(0, |acc, &x| acc * 1000 + x);

    Ok(current < update)
}
