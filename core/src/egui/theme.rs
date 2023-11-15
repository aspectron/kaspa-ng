use crate::imports::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Theme {
    pub kaspa_color: Color32,
    pub hyperlink_color: Color32,
    pub node_data_color: Color32,
    pub panel_icon_size: IconSize,
    pub panel_margin_size: f32,
    pub error_icon_size: IconSize,
    pub medium_button_size: Vec2,
    pub large_button_size: Vec2,
    pub panel_footer_height: f32, //72_f32,
    // pub panel_alert_icon_size : IconSize,
    // pub panel_icon_size : IconSize,
    pub error_color: Color32,
    pub warning_color: Color32,
    pub ack_color: Color32,
    pub nack_color: Color32,

    pub status_icon_size: f32,
    pub progress_color: Color32,
    pub graph_frame_color: Color32,
    pub performance_graph_color: Color32,
    pub storage_graph_color: Color32,
    pub node_graph_color: Color32,
    pub network_graph_color: Color32,
    // pub panel_icon_size : f32,
    // pub panel_icon_padding : f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            kaspa_color: Color32::from_rgb(58, 221, 190),
            // hyperlink_color: Color32::from_rgb(58, 221, 190),
            hyperlink_color: Color32::from_rgb(38, 148, 128),
            // node_data_color : Color32::from_rgb(217, 233,230),
            node_data_color: Color32::WHITE,
            // node_data_color : Color32::from_rgb(151, 209, 198),
            // panel_icon_size : IconSize::new(26.,36.),
            panel_icon_size: IconSize::new(Vec2::splat(26.)).with_padding(Vec2::new(6., 0.)),
            error_icon_size: IconSize::new(Vec2::splat(64.)).with_padding(Vec2::new(6., 6.)),
            medium_button_size: Vec2::new(100_f32, 30_f32),
            large_button_size: Vec2::new(200_f32, 40_f32),
            panel_footer_height: 72_f32,
            panel_margin_size: 24_f32,
            // panel_error_icon_size : IconSize::new(Vec2::splat(26.)).with_padding(Vec2::new(6.,0.)),
            error_color: Color32::from_rgb(255, 136, 136), //Color32::from_rgb(255, 0, 0),
            warning_color: egui::Color32::from_rgb(255, 255, 136),
            ack_color: Color32::from_rgb(100, 200, 100),
            nack_color: Color32::from_rgb(200, 100, 100),

            status_icon_size: 18_f32,
            progress_color: Color32::from_rgb(21, 82, 71),

            graph_frame_color: Color32::GRAY,
            performance_graph_color: Color32::from_rgb(186, 238, 255),
            storage_graph_color: Color32::from_rgb(255, 231, 186),
            node_graph_color: Color32::from_rgb(241, 255, 186),
            network_graph_color: Color32::from_rgb(186, 255, 241),
            // network_graph_color: Color32::from_rgb(58, 221, 190),
            // graph_color: Color32::from_rgb(21, 82, 71),
            // panel_icon_size : IconSize::new(Vec2::splat(26.),Vec2::new(36.,26.)),
        }
    }
}

impl Theme {
    pub fn panel_icon_size(&self) -> &IconSize {
        &self.panel_icon_size
    }

    pub fn panel_margin_size(&self) -> f32 {
        self.panel_margin_size
    }

    pub fn medium_button_size(&self) -> Vec2 {
        self.medium_button_size
    }

    pub fn large_button_size(&self) -> Vec2 {
        self.large_button_size
    }

    // pub fn panel_icon_padding(&self) -> f32 {
    //     self.panel_icon_padding
    // }

    pub fn apply(&self, visuals: &mut Visuals) {
        // let visuals = ui.visuals_mut();
        visuals.hyperlink_color = self.hyperlink_color;
    }
}

static mut THEME: Option<Theme> = None;
#[inline(always)]
pub fn theme() -> &'static Theme {
    unsafe {
        THEME.get_or_insert_with(Theme::default)

        // if THEME.is_none() {
        //     THEME = Some(Rc::new(Theme {
        //         icon_size : 20_f32,
        //     }));
        // }
        // THEME.clone().unwrap()
    }
}

pub fn apply_theme(theme: Theme) {
    unsafe {
        THEME = Some(theme);
    }
}

// pub fn theme() -> Rc<Theme> {
//     static mut THEME : Option<Rc<Theme>> = None;
//     unsafe {
//         THEME.get_or_insert_with(||Rc::new(Theme::default())).clone()
//         // if THEME.is_none() {
//         //     THEME = Some(Rc::new(Theme {
//         //         icon_size : 20_f32,
//         //     }));
//         // }
//         // THEME.clone().unwrap()
//     }
// }
