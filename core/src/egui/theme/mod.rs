use kaspa_metrics_core::MetricGroup;

mod color;
pub use color::*;
mod style;
use crate::imports::*;
pub use style::*;

#[derive(Clone)]
pub struct AppTheme {
    pub color: Arc<ThemeColor>,
    pub style: Arc<ThemeStyle>,
}

impl AppTheme {
    pub fn new(color: Arc<ThemeColor>, style: Arc<ThemeStyle>) -> Self {
        Self { color, style }
    }

    #[inline(always)]
    pub fn color(&self) -> &ThemeColor {
        &self.color
    }

    #[inline(always)]
    pub fn style(&self) -> &ThemeStyle {
        &self.style
    }
}

impl Default for AppTheme {
    fn default() -> Self {
        Self {
            color: Arc::new(ThemeColor::dark()),
            style: Arc::new(ThemeStyle::rounded()),
        }
    }
}

impl From<&AppTheme> for Visuals {
    fn from(theme: &AppTheme) -> Self {
        let mut visuals = if theme.color.dark_mode {
            Visuals::dark()
        } else {
            Visuals::light()
        };

        visuals.widgets.active.corner_radius = theme.style.widget_rounding;
        visuals.widgets.inactive.corner_radius = theme.style.widget_rounding;
        visuals.widgets.hovered.corner_radius = theme.style.widget_rounding;
        visuals.widgets.noninteractive.corner_radius = theme.style.widget_rounding;
        visuals.widgets.open.corner_radius = theme.style.widget_rounding;

        visuals.hyperlink_color = theme.color.hyperlink_color;
        visuals.selection.bg_fill = theme.color.selection_background_color;
        visuals.selection.stroke.color = theme.color.selection_text_color;
        visuals.warn_fg_color = theme.color.warning_color;
        visuals.error_fg_color = theme.color.error_color;

        visuals
    }
}

impl AsRef<AppTheme> for AppTheme {
    fn as_ref(&self) -> &Self {
        self
    }
}

static THEME: Mutex<Option<Arc<AppTheme>>> = Mutex::new(None);

#[inline(always)]
pub fn theme() -> Arc<AppTheme> {
    let mut theme_lock = THEME.lock().unwrap();
    theme_lock
        .get_or_insert_with(|| Arc::new(AppTheme::default()))
        .clone()
}

#[inline(always)]
pub fn theme_color() -> Arc<ThemeColor> {
    Arc::clone(&theme().color)
}

#[inline(always)]
pub fn theme_style() -> Arc<ThemeStyle> {
    Arc::clone(&theme().style)
}

pub fn apply_theme_by_name(
    ctx: &Context,
    theme_color_name: impl Into<String>,
    theme_style_name: impl Into<String>,
) {
    let theme_color_name = theme_color_name.into();
    let theme_color = theme_colors()
        .get(&theme_color_name)
        .cloned()
        .unwrap_or_else(|| {
            log_error!("Theme color not found: {}", theme_color_name);
            ThemeColor::default()
        });

    let theme_style_name = theme_style_name.into();
    let theme_style = theme_styles()
        .get(&theme_style_name)
        .cloned()
        .unwrap_or_else(|| {
            log_error!("Theme style not found: {}", theme_style_name);
            ThemeStyle::default()
        });

    apply_theme(ctx, AppTheme::new(theme_color.into(), theme_style.into()));
}

pub fn apply_theme_color_by_name(ctx: &Context, theme_color_name: impl Into<String>) {
    let theme_color_name = theme_color_name.into();
    let theme_color = theme_colors()
        .get(&theme_color_name)
        .cloned()
        .unwrap_or_else(|| {
            log_error!("Theme not found: {}", theme_color_name);
            ThemeColor::default()
        });

    apply_theme(
        ctx,
        AppTheme::new(theme_color.into(), theme_style().clone()),
    );
}

pub fn apply_theme_style_by_name(ctx: &Context, theme_style_name: impl Into<String>) {
    let theme_style_name = theme_style_name.into();
    let theme_style = theme_styles()
        .get(&theme_style_name)
        .cloned()
        .unwrap_or_else(|| {
            log_error!("Theme not found: {}", theme_style_name);
            ThemeStyle::default()
        });

    apply_theme(
        ctx,
        AppTheme::new(theme_color().clone(), theme_style.into()),
    );
}

pub fn apply_theme(ctx: &Context, theme: AppTheme) {
    let _ = THEME.lock().unwrap().insert(Arc::new(theme.clone()));
    ctx.set_visuals(theme.as_ref().into());
    runtime()
        .application_events()
        .try_send(Events::ThemeChange)
        .unwrap();
}

// ~

#[inline(always)]
pub fn error_color() -> Color32 {
    theme_color().error_color
}

#[inline(always)]
pub fn warning_color() -> Color32 {
    theme_color().warning_color
}

#[inline(always)]
pub fn info_color() -> Color32 {
    theme_color().info_color
}

#[inline(always)]
pub fn strong_color() -> Color32 {
    theme_color().strong_color
}

// ~

pub trait MetricGroupExtension {
    fn to_color(&self) -> Color32;
}

impl MetricGroupExtension for MetricGroup {
    fn to_color(&self) -> Color32 {
        match self {
            MetricGroup::System => theme_color().performance_graph_color,
            MetricGroup::Storage => theme_color().storage_graph_color,
            MetricGroup::Connections => theme_color().connections_graph_color,
            MetricGroup::Bandwidth => theme_color().bandwidth_graph_color,
            MetricGroup::Network => theme_color().network_graph_color,
        }
    }
}
