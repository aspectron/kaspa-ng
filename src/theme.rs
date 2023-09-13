use crate::imports::*;

// #[derive(Default)]
pub struct Theme {
    pub panel_icon_size: IconSize,
    pub panel_margin_size: f32,
    pub error_icon_size: IconSize,
    pub large_button_size: Vec2,
    pub panel_footer_height: f32, //72_f32,
    // pub panel_alert_icon_size : IconSize,
    // pub panel_icon_size : IconSize,
    pub error_color: Color32,
    pub warning_color: Color32,
    // pub panel_icon_size : f32,
    // pub panel_icon_padding : f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            // panel_icon_size : IconSize::new(26.,36.),
            panel_icon_size: IconSize::new(Vec2::splat(26.)).with_padding(Vec2::new(6., 0.)),
            error_icon_size: IconSize::new(Vec2::splat(64.)).with_padding(Vec2::new(6., 6.)),
            large_button_size: Vec2::new(200_f32, 40_f32),
            panel_footer_height: 72_f32,
            panel_margin_size: 24_f32,
            // panel_error_icon_size : IconSize::new(Vec2::splat(26.)).with_padding(Vec2::new(6.,0.)),
            error_color: Color32::from_rgb(255, 136, 136), //Color32::from_rgb(255, 0, 0),
            warning_color: Color32::from_rgb(255, 255, 0),
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

    // pub fn panel_icon_padding(&self) -> f32 {
    //     self.panel_icon_padding
    // }
}

static mut THEME: Option<Theme> = None;
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
