use crate::imports::*;

#[derive(Clone)]
pub struct ThemeStyle {
    pub name: String,
    pub widget_rounding: CornerRadius,
    pub widget_spacing: f32,
    pub panel_icon_size: IconSize,
    pub panel_margin_size: f32,
    pub error_icon_size: IconSize,
    pub medium_button_size: Vec2,
    pub large_button_size: Vec2,
    pub panel_footer_height: f32,
    pub panel_editor_size: Vec2,
    pub icon_size_large: f32,
    pub icon_size_medium: f32,
    pub status_icon_size: f32,
    pub node_log_font_size: f32,
    pub composite_icon_size: f32,
}

impl ThemeStyle {
    pub fn rounded() -> ThemeStyle {
        Self {
            name: "Rounded".to_string(),
            widget_rounding: CornerRadius::from(6.),
            widget_spacing: 6_f32,
            panel_icon_size: IconSize::new(Vec2::splat(26.)).with_padding(Vec2::new(6., 0.)),
            error_icon_size: IconSize::new(Vec2::splat(64.)).with_padding(Vec2::new(6., 6.)),
            medium_button_size: Vec2::new(100_f32, 30_f32),
            large_button_size: Vec2::new(200_f32, 40_f32),
            panel_footer_height: 72_f32,
            panel_margin_size: 24_f32,
            panel_editor_size: Vec2::new(200_f32, 40_f32),
            icon_size_large: 96_f32,
            icon_size_medium: 48_f32,
            status_icon_size: 18_f32,
            node_log_font_size: 15_f32,
            composite_icon_size: 32_f32,
        }
    }

    pub fn sharp() -> ThemeStyle {
        Self {
            name: "Sharp".to_string(),
            widget_rounding: CornerRadius::from(0.),
            widget_spacing: 6_f32,
            panel_icon_size: IconSize::new(Vec2::splat(26.)).with_padding(Vec2::new(6., 0.)),
            error_icon_size: IconSize::new(Vec2::splat(64.)).with_padding(Vec2::new(6., 6.)),
            medium_button_size: Vec2::new(100_f32, 30_f32),
            large_button_size: Vec2::new(200_f32, 40_f32),
            panel_footer_height: 72_f32,
            panel_margin_size: 24_f32,
            panel_editor_size: Vec2::new(200_f32, 40_f32),
            icon_size_large: 96_f32,
            icon_size_medium: 48_f32,
            status_icon_size: 18_f32,
            node_log_font_size: 15_f32,
            composite_icon_size: 32_f32,
        }
    }
}

impl Default for ThemeStyle {
    fn default() -> Self {
        ThemeStyle::rounded()
    }
}

impl ThemeStyle {
    pub fn name(&self) -> &str {
        &self.name
    }

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
}

static THEME_STYLE_LIST: Mutex<Option<Arc<HashMap<String, ThemeStyle>>>> = Mutex::new(None);

#[inline(always)]
pub fn theme_styles() -> Arc<HashMap<String, ThemeStyle>> {
    let mut theme_styles_lock = THEME_STYLE_LIST.lock().unwrap();
    theme_styles_lock
        .get_or_insert_with(|| {
            let mut themes = HashMap::new();
            [ThemeStyle::rounded(), ThemeStyle::sharp()]
                .into_iter()
                .for_each(|theme| {
                    themes.insert(theme.name.clone(), theme.clone());
                });
            Arc::new(themes)
        })
        .clone()
}
