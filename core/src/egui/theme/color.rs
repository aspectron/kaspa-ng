use crate::imports::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ThemeColor {
    pub name: String,
    pub dark_mode: bool,

    pub kaspa_color: Color32,
    pub hyperlink_color: Color32,
    pub node_data_color: Color32,
    pub balance_color: Color32,
    pub balance_syncing_color: Color32,
    pub error_color: Color32,
    pub alert_color: Color32,
    pub warning_color: Color32,
    pub info_color: Color32,
    pub icon_syncing_color: Color32,
    pub icon_connected_color: Color32,
    pub icon_color_default: Color32,
    pub ack_color: Color32,
    pub nack_color: Color32,
    pub metrics_text_color: Color32,
    pub market_default_color: Color32,
    pub market_up_color: Color32,
    pub market_down_color: Color32,

    pub raised_text_color: Color32,
    pub raised_text_shadow: Color32,

    pub qr_background: Color32,
    pub qr_foreground: Color32,
    pub selection_background_color: Color32,
    pub selection_text_color: Color32,
    pub progress_color: Color32,

    pub default_color: Color32,
    pub strong_color: Color32,
    pub transaction_incoming: Color32,
    pub transaction_outgoing: Color32,
    pub transaction_external: Color32,
    pub transaction_reorg: Color32,
    pub transaction_batch: Color32,
    pub transaction_stasis: Color32,
    pub transaction_transfer_incoming: Color32,
    pub transaction_transfer_outgoing: Color32,
    pub transaction_change: Color32,

    pub logs_info_color: Color32,
    pub logs_error_color: Color32,
    pub logs_warning_color: Color32,
    pub logs_debug_color: Color32,
    pub logs_trace_color: Color32,
    pub logs_processed_color: Color32,

    pub graph_frame_color: Color32,
    pub performance_graph_color: Color32,
    pub storage_graph_color: Color32,
    pub connections_graph_color: Color32,
    pub bandwidth_graph_color: Color32,
    pub network_graph_color: Color32,

    pub block_dag_separator_color: Color32,
    pub block_dag_new_block_fill_color: Color32,
    pub block_dag_block_fill_color: Color32,
    pub block_dag_block_stroke_color: Color32,
    pub block_dag_vspc_connect_color: Color32,
    pub block_dag_parent_connect_color: Color32,
}

impl ThemeColor {
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            dark_mode: true,
            kaspa_color: Color32::from_rgb(58, 221, 190),
            hyperlink_color: Color32::from_rgb(141, 184, 178),

            default_color: Color32::LIGHT_GRAY,
            strong_color: Color32::WHITE,

            node_data_color: Color32::WHITE,
            balance_color: Color32::WHITE,
            balance_syncing_color: Color32::DARK_GRAY,
            error_color: Color32::from_rgb(255, 136, 136),
            alert_color: Color32::from_rgb(255, 136, 136),
            warning_color: egui::Color32::from_rgb(255, 255, 136),
            info_color: egui::Color32::from_rgb(66, 178, 252),
            icon_syncing_color: egui::Color32::from_rgb(255, 255, 136),
            icon_connected_color: egui::Color32::from_rgb(85, 233, 136),
            icon_color_default: Color32::from_rgb(240, 240, 240),
            ack_color: Color32::from_rgb(100, 200, 100),
            nack_color: Color32::from_rgb(200, 100, 100),
            metrics_text_color: Color32::from_rgb(230, 230, 230),
            market_default_color: Color32::from_rgb(240, 240, 240),
            market_up_color: Color32::from_rgb(136, 255, 136),
            market_down_color: Color32::from_rgb(255, 136, 136),

            raised_text_color: Color32::from_rgb(255, 255, 255),
            raised_text_shadow: Color32::from_rgba(0, 0, 0, 96),

            qr_background: Color32::from_rgba(0, 0, 0, 0),
            qr_foreground: Color32::WHITE,
            selection_background_color: Color32::from_rgb(50, 50, 50),
            selection_text_color: Color32::from_rgb(255, 255, 255),
            progress_color: Color32::from_rgb(71, 105, 97),

            transaction_incoming: Color32::from_rgb(162, 245, 187),
            transaction_outgoing: Color32::from_rgb(245, 162, 162),
            transaction_transfer_incoming: Color32::from_rgb(162, 245, 187),
            transaction_transfer_outgoing: Color32::from_rgb(245, 162, 162),
            transaction_external: Color32::from_rgb(162, 245, 187),
            transaction_reorg: Color32::from_rgb(79, 64, 64),
            transaction_batch: Color32::GRAY,
            transaction_stasis: Color32::GRAY,
            transaction_change: Color32::GRAY,

            logs_info_color: Color32::WHITE,
            logs_error_color: Color32::LIGHT_RED,
            logs_warning_color: Color32::LIGHT_YELLOW,
            logs_debug_color: Color32::LIGHT_BLUE,
            logs_trace_color: Color32::LIGHT_GRAY,
            logs_processed_color: Color32::LIGHT_GREEN,

            graph_frame_color: Color32::GRAY,
            performance_graph_color: Color32::from_rgb(186, 238, 255),
            storage_graph_color: Color32::from_rgb(255, 231, 186),
            connections_graph_color: Color32::from_rgb(241, 255, 186),
            bandwidth_graph_color: Color32::from_rgb(196, 255, 199),
            network_graph_color: Color32::from_rgb(186, 255, 241),

            block_dag_separator_color: Color32::from_rgb(220, 220, 220),
            block_dag_new_block_fill_color: Color32::from_rgb(220, 220, 220),
            block_dag_block_fill_color: Color32::from_rgb(173, 216, 230),
            block_dag_block_stroke_color: Color32::from_rgb(15, 84, 77),
            block_dag_vspc_connect_color: Color32::from_rgb(23, 150, 137),
            block_dag_parent_connect_color: Color32::from_rgba_premultiplied(173, 216, 230, 220),
        }
    }

    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            dark_mode: false,
            kaspa_color: Color32::from_rgb(58, 221, 190),
            hyperlink_color: Color32::from_rgb(15, 84, 73),

            default_color: Color32::DARK_GRAY,
            strong_color: Color32::BLACK,

            node_data_color: Color32::BLACK,
            balance_color: Color32::BLACK,
            balance_syncing_color: Color32::LIGHT_GRAY,
            error_color: Color32::from_rgb(77, 41, 41),
            alert_color: Color32::from_rgb(77, 41, 41),
            warning_color: egui::Color32::from_rgb(77, 77, 41),
            info_color: egui::Color32::from_rgb(41, 56, 77),
            icon_syncing_color: egui::Color32::from_rgb(117, 117, 4),
            icon_connected_color: egui::Color32::from_rgb(8, 110, 65),
            icon_color_default: Color32::from_rgb(32, 32, 32),
            ack_color: Color32::from_rgb(100, 200, 100),
            nack_color: Color32::from_rgb(200, 100, 100),
            metrics_text_color: Color32::from_rgb(20, 20, 20),
            market_default_color: Color32::from_rgb(20, 20, 20),
            market_up_color: Color32::from_rgb(41, 77, 41),
            market_down_color: Color32::from_rgb(77, 41, 41),

            raised_text_color: Color32::from_rgb(0, 0, 0),
            raised_text_shadow: Color32::from_rgba(255, 255, 255, 64),

            qr_background: Color32::from_rgba(255, 255, 255, 0),
            qr_foreground: Color32::BLACK,
            selection_background_color: Color32::from_rgb(165, 201, 197),
            selection_text_color: Color32::from_rgb(20, 20, 20),
            progress_color: Color32::from_rgb(165, 201, 197),

            transaction_incoming: Color32::from_rgb(15, 77, 35),
            transaction_outgoing: Color32::from_rgb(77, 15, 15),
            transaction_transfer_incoming: Color32::from_rgb(15, 77, 35),
            transaction_transfer_outgoing: Color32::from_rgb(77, 15, 15),
            transaction_external: Color32::from_rgb(15, 77, 35),
            transaction_change: Color32::GRAY,
            transaction_reorg: Color32::from_rgb(38, 31, 31),
            transaction_batch: Color32::GRAY,
            transaction_stasis: Color32::GRAY,

            logs_info_color: Color32::BLACK,
            logs_error_color: Color32::DARK_RED,
            logs_warning_color: Color32::BROWN,
            logs_debug_color: Color32::DARK_BLUE,
            logs_trace_color: Color32::DARK_GRAY,
            logs_processed_color: Color32::DARK_GREEN,

            graph_frame_color: Color32::GRAY,
            performance_graph_color: Color32::from_rgb(56, 71, 77),
            storage_graph_color: Color32::from_rgb(77, 69, 56),
            connections_graph_color: Color32::from_rgb(72, 77, 56),
            bandwidth_graph_color: Color32::from_rgb(59, 77, 60),
            network_graph_color: Color32::from_rgb(56, 77, 72),

            block_dag_separator_color: Color32::from_rgb(120, 120, 120),
            block_dag_new_block_fill_color: Color32::from_rgb(220, 220, 220),
            block_dag_block_fill_color: Color32::from_rgb(201, 230, 240),
            block_dag_block_stroke_color: Color32::from_rgb(42, 51, 50),
            block_dag_vspc_connect_color: Color32::from_rgb(11, 77, 70),
            block_dag_parent_connect_color: Color32::from_rgba_premultiplied(0, 0, 0, 72),
        }
    }
}

impl Default for ThemeColor {
    fn default() -> Self {
        Self::dark()
    }
}

impl ThemeColor {
    pub fn name(&self) -> &str {
        &self.name
    }
}

static THEME_COLOR_LIST: Mutex<Option<Arc<HashMap<String, ThemeColor>>>> = Mutex::new(None);

#[inline(always)]
pub fn theme_colors() -> Arc<HashMap<String, ThemeColor>> {
    let mut colors_lock = THEME_COLOR_LIST.lock().unwrap();
    colors_lock
        .get_or_insert_with(|| {
            let mut themes = HashMap::new();
            [ThemeColor::dark(), ThemeColor::light()]
                .into_iter()
                .for_each(|theme| {
                    themes.insert(theme.name.clone(), theme.clone());
                });
            Arc::new(themes)
        })
        .clone()
}
