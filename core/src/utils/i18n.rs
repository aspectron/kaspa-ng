#[allow(unused_imports)]
use crate::imports::*;

#[cfg(not(target_arch = "wasm32"))]
pub fn try_cwd_repo_root() -> Result<Option<PathBuf>> {
    let cwd = std::env::current_dir()?;
    let cargo_toml = cwd.join("Cargo.toml");
    let resources = cwd.join("core").join("resources").join("i18n");
    Ok((cargo_toml.exists() && resources.exists()).then_some(cwd))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn i18n_storage_folder() -> Result<PathBuf> {
    // check if we are in the repository, then use /resources/i18n/i18n.json
    let mut path = std::env::current_exe()?;
    path.pop();
    if path.ends_with("debug") || path.ends_with("release") {
        path.pop();
        if path.ends_with("target") {
            path.pop();
        }
        path.push("core");
        path.push("resources");
        path.push("i18n");
        path.push("i18n.json");
        if !path.exists() {
            panic!("Expecting i18n.json in the repository at '/core/resources/i18n/i18n.json'")
        } else {
            path.pop();
        }
        Ok(path)
    } else {
        // check if we can find i18n.json in the same folder as the executable
        path.push("i18n.json");
        if path.exists() {
            path.pop();
            Ok(path)
        } else {
            // check for i18n.json in the current working directory
            let mut local = std::env::current_dir()?.join("i18n.json");
            if local.exists() {
                local.pop();
                Ok(local)
            } else {
                // fallback to the default storage folder, which is the
                // same as kaspa-ng settings storage folder: `~/.kaspa-ng/`
                let default_storage_folder =
                    kaspa_wallet_core::storage::local::default_storage_folder();
                let storage_folder = workflow_store::fs::resolve_path(default_storage_folder)?;
                if !storage_folder.exists() {
                    std::fs::create_dir_all(&storage_folder)?;
                }
                Ok(storage_folder.to_path_buf())
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn i18n_storage_file() -> Result<PathBuf> {
    // check if we are in the repository, then use /resources/i18n/i18n.json
    let mut path = std::env::current_exe()?;
    path.pop();
    if path.ends_with("debug") || path.ends_with("release") {
        path.pop();
        if path.ends_with("target") {
            path.pop();
        }
        path.push("core");
        path.push("resources");
        path.push("i18n");
        path.push("i18n.json");
        Ok(path)
    } else {
        // check if we can find i18n.json in the same folder as the executable
        let in_same_folder = path.join("i18n.json");
        if in_same_folder.exists() {
            Ok(in_same_folder)
        } else {
            // check for i18n.json in the current working directory
            let local = std::env::current_dir()?.join("i18n.json");
            if local.exists() {
                Ok(local)
            } else {
                // fallback to the default storage folder, which is the
                // same as kaspa-ng settings storage folder: `~/.kaspa-ng/`
                let default_storage_folder =
                    kaspa_wallet_core::storage::local::default_storage_folder();
                let storage_folder = workflow_store::fs::resolve_path(default_storage_folder)?;
                if !storage_folder.exists() {
                    std::fs::create_dir_all(&storage_folder)?;
                }
                Ok(storage_folder.join("kaspa-ng.i18n.json"))
            }
        }
    }
}
