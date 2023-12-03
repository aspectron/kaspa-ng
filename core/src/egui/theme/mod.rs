// use egui::style::WidgetVisuals;
use kaspa_metrics::MetricGroup;

mod color;
pub use color::*;
mod style;
pub use style::*;

use crate::imports::*;

#[derive(Clone)]
pub struct Theme {
    pub color: ThemeColor,
    pub style: ThemeStyle,
}

impl Theme {
    pub fn new(color: ThemeColor, style: ThemeStyle) -> Self {
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

impl Default for Theme {
    fn default() -> Self {
        Self {
            color: ThemeColor::dark(),
            style: ThemeStyle::rounded(),
        }
    }
}

impl From<&Theme> for Visuals {
    fn from(theme: &Theme) -> Self {
        let mut visuals = if theme.color.dark_mode {
            Visuals::dark()
        } else {
            Visuals::light()
        };

        visuals.widgets.active.rounding = theme.style.widget_rounding;
        visuals.widgets.inactive.rounding = theme.style.widget_rounding;
        visuals.widgets.hovered.rounding = theme.style.widget_rounding;
        visuals.widgets.noninteractive.rounding = theme.style.widget_rounding;
        visuals.widgets.open.rounding = theme.style.widget_rounding;

        visuals.hyperlink_color = theme.color.hyperlink_color;
        visuals.selection.bg_fill = theme.color.selection_color;
        visuals.warn_fg_color = theme.color.warning_color;
        visuals.error_fg_color = theme.color.error_color;

        visuals
    }
}

impl AsRef<Theme> for Theme {
    fn as_ref(&self) -> &Self {
        self
    }
}

// impl AsMut<Theme> for Theme {
//     fn as_mut(&mut self) -> &mut Self {
//         self
//     }
// }

static mut THEME: Option<Theme> = None;
#[inline(always)]
pub fn theme() -> &'static Theme {
    unsafe { THEME.get_or_insert_with(Theme::default) }
}

#[inline(always)]
pub fn theme_color() -> &'static ThemeColor {
    &theme().color
}

#[inline(always)]
pub fn theme_style() -> &'static ThemeStyle {
    &theme().style
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

    apply_theme(ctx, Theme::new(theme_color, theme_style));
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

    apply_theme(ctx, Theme::new(theme_color, theme_style().clone()));
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

    apply_theme(ctx, Theme::new(theme_color().clone(), theme_style));
}

pub fn apply_theme(ctx: &Context, theme: Theme) {
    unsafe {
        THEME = Some(theme.clone());
    }
    ctx.set_visuals(theme.as_ref().into());
    runtime()
        .application_events()
        .try_send(Events::ThemeChange)
        .unwrap();
}

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
