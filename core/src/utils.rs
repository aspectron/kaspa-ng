use crate::imports::*;
use qrcode::render::svg;
use qrcode::*;

pub fn render_qrcode(text: &str, width: usize, height: usize) -> String {
    let code = QrCode::with_version(text, Version::Normal(4), EcLevel::L).unwrap();

    // let _theme = theme();

    code.render::<svg::Color<'_>>()
        .min_dimensions(width as u32, height as u32)
        .dark_color(svg::Color("#ffffff"))
        .light_color(svg::Color("#00000000"))
        .build()
        .to_string()
}
// #[macro_use]
#[macro_export]
macro_rules! spawn {
    ($args: expr) => {{
        let id = concat!(file!(), ":", line!());
        let payload = Payload::new(id);
        if !payload.is_pending() {
            spawn_with_result(&payload, $args);
        }
        payload.take()
    }};
}

pub use spawn;

/* */
pub fn icon_with_text(ui: &Ui, icon: &str, color: Color32, text: &str) -> LayoutJob {
    // ui.horizontal(|ui| {
    //     ui.add(crate::theme::icon(icon));
    //     ui.add(crate::theme::text(text));
    // });

    // let size = ui.ctx().style().text_styles.entry(TextStyle::Button).or_default().size;
    let text_color = ui.ctx().style().visuals.widgets.inactive.text_color(); //.text_color();
    let text_size = ui
        .ctx()
        .style()
        .text_styles
        .get(&TextStyle::Button)
        .unwrap()
        .size;

    let _theme = theme();

    let mut job = LayoutJob {
        halign: Align::Min,
        // justify: true,
        ..Default::default()
    };

    job.append(
        icon,
        0.0,
        TextFormat {
            // font_id: FontId::new(text_size + 4., FontFamily::Name("phosphor".into())),
            font_id: FontId::new(text_size + 4., FontFamily::Proportional),
            color,
            valign: Align::Center,
            ..Default::default()
        },
    );
    //  job.append(text, leading_space, format)
    job.append(
        text,
        2.0,
        TextFormat {
            font_id: FontId::new(text_size, FontFamily::Proportional),
            color: text_color,
            valign: Align::Center,
            ..Default::default()
        },
    );
    // job.append(
    //     wallet.filename.clone().as_str(),
    //     0.0,
    //     TextFormat {
    //         font_id: FontId::new(12.0, FontFamily::Proportional),
    //         color: ui.ctx().style().visuals.text_color(),
    //         ..Default::default()
    //     },
    // );

    job
}

pub fn format_duration(millis: u64) -> String {
    let seconds = millis / 1000;
    // let seconds = seconds_f64 as u64;
    let days = seconds / (24 * 60 * 60);
    let hours = (seconds / (60 * 60)) % 24;
    let minutes = (seconds / 60) % 60;
    let seconds = (seconds % 60) as f64 + (millis % 1000) as f64 / 1000.0;

    if days > 0 {
        format!("{} days {:02}:{:02}:{:02.4}", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{:02}:{:02}:{:02.4}", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{:02}:{:02.4}", minutes, seconds)
    } else if millis > 1000 {
        format!("{:2.4} sec", seconds)
    } else {
        format!("{} msec", millis)
    }
}

pub fn format_address(address: &Address, range: Option<usize>) -> String {
    let address = address.to_string();

    let parts = address.split(':').collect::<Vec<&str>>();
    let prefix = parts[0];
    let payload = parts[1];
    let range = range.unwrap_or(6);
    let start = range;
    let finish = payload.len() - range;

    let left = &payload[0..start];
    // let center = style(&payload[start..finish]).dim();
    let right = &payload[finish..];

    format!("{prefix}:{left}....{right}")
}

#[derive(Default)]
pub struct Arglist {
    pub args: Vec<String>,
}

impl Arglist {
    pub fn push(&mut self, arg: impl Into<String>) {
        self.args.push(arg.into());
    }
}

impl From<Arglist> for Vec<String> {
    fn from(arglist: Arglist) -> Self {
        arglist.args
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn try_cwd_repo_root() -> Result<Option<PathBuf>> {
    let cwd = std::env::current_dir()?;
    let cargo_toml = cwd.join("Cargo.toml");
    let resources = cwd.join("resources").join("i18n");
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
        path.push("resources");
        path.push("i18n");
        path.push("i18n.json");
        if !path.exists() {
            panic!("Expecting i18n.json in the repository at '/resources/i18n/i18n.json'")
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
                let storage_folder =
                    Path::new(kaspa_wallet_core::storage::local::DEFAULT_STORAGE_FOLDER);
                if !storage_folder.exists() {
                    std::fs::create_dir_all(storage_folder)?;
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
                let storage_folder =
                    Path::new(kaspa_wallet_core::storage::local::DEFAULT_STORAGE_FOLDER);
                if !storage_folder.exists() {
                    std::fs::create_dir_all(storage_folder)?;
                }
                Ok(storage_folder.join("kaspa-ng.i18n.json"))
            }
        }
    }
}

pub fn try_user_string_to_address(address: &str, network_type: &NetworkType) -> Result<Address> {
    let address = Address::try_from(address)?;
    let address_network_type = NetworkType::try_from(address.prefix)?;
    if &address_network_type != network_type {
        return Err(Error::custom(format!(
            "{} {} ({} {})",
            i18n("Invalid address network type:"),
            address_network_type,
            i18n("expected:"),
            network_type
        )));
    }
    Ok(address)
}

pub fn hash_to_y_coord(hash: &kaspa_consensus_core::Hash, scale: f64) -> f64 {
    let bytes = hash.as_bytes().iter().take(2).cloned().collect::<Vec<_>>();
    (i16::from_le_bytes(bytes.as_slice().try_into().unwrap()) - 127) as f64 * (scale / 32767.5)
}

pub fn bezier(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    steps: usize,
    offset_factor: f64,
) -> Vec<PlotPoint> {
    let mut points = vec![];

    let offset = (x2 - x1) * offset_factor;

    let control_point1_x = x1 + offset;
    let control_point1_y = y1;

    let control_point2_x = x2 - offset;
    let control_point2_y = y2;

    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let inv_t = 1.0 - t;

        let x = x1 * inv_t.powi(3)
            + 3.0 * control_point1_x * inv_t.powi(2) * t
            + 3.0 * control_point2_x * inv_t * t.powi(2)
            + x2 * t.powi(3);
        let y = y1 * inv_t.powi(3)
            + 3.0 * control_point1_y * inv_t.powi(2) * t
            + 3.0 * control_point2_y * inv_t * t.powi(2)
            + y2 * t.powi(3);
        points.push(PlotPoint::new(x, y));
    }

    points
}
