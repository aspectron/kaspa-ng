use crate::imports::*;
use std::fmt::Display;

type ActionFn<'panel, Context> = Box<dyn FnOnce(&mut Context) + 'panel>;
type RenderFn<'panel, Context> = Box<dyn FnOnce(&mut Context, &mut Ui) + 'panel>;

pub struct Panel<'panel, Context> {
    pub this: &'panel mut Context,
    caption: Option<String>,
    close: Option<ActionFn<'panel, Context>>,
    close_enabled: bool,
    close_active: bool,
    back: Option<ActionFn<'panel, Context>>,
    back_enabled: bool,
    back_active: bool,
    header: Option<RenderFn<'panel, Context>>,
    body: Option<RenderFn<'panel, Context>>,
    footer: Option<RenderFn<'panel, Context>>,
    handler: Option<ActionFn<'panel, Context>>,
    handler_text: Option<String>,
    handler_enabled: bool,
}

impl<'panel, Context> Panel<'panel, Context> {
    pub fn new(this: &'panel mut Context) -> Self {
        Self {
            this,
            close: None,
            close_enabled: true,
            close_active: true,
            back: None,
            back_enabled: true,
            back_active: true,
            caption: None,
            header: None,
            body: None,
            footer: None,
            handler: None,
            handler_text: None,
            handler_enabled: true,
        }
    }

    pub fn with_close(mut self, close: impl FnOnce(&mut Context) + 'panel) -> Self {
        self.close = Some(Box::new(close));
        self
    }

    pub fn with_close_enabled(
        mut self,
        enabled: bool,
        close: impl FnOnce(&mut Context) + 'panel,
    ) -> Self {
        self.close = Some(Box::new(close));
        self.close_enabled = enabled;
        self
    }

    pub fn with_back(mut self, back: impl FnOnce(&mut Context) + 'panel) -> Self {
        self.back = Some(Box::new(back));
        self
    }

    pub fn with_back_enabled(
        mut self,
        enabled: bool,
        back: impl FnOnce(&mut Context) + 'panel,
    ) -> Self {
        self.back = Some(Box::new(back));
        self.back_enabled = enabled;
        self
    }

    pub fn with_caption<S: Display>(mut self, caption: S) -> Self {
        self.caption = Some(caption.to_string());
        self
    }

    pub fn with_header(mut self, header: impl FnOnce(&mut Context, &mut Ui) + 'panel) -> Self {
        self.header = Some(Box::new(header));
        self
    }

    pub fn with_body(mut self, body: impl FnOnce(&mut Context, &mut Ui) + 'panel) -> Self {
        self.body = Some(Box::new(body));
        self
    }

    pub fn with_footer(mut self, footer: impl FnOnce(&mut Context, &mut Ui) + 'panel) -> Self {
        self.footer = Some(Box::new(footer));
        self
    }

    pub fn with_handler(mut self, handler: impl FnOnce(&mut Context) + 'panel) -> Self {
        self.handler = Some(Box::new(handler));
        self
    }

    pub fn with_handler_enabled(
        mut self,
        enabled: bool,
        handler: impl FnOnce(&mut Context) + 'panel,
    ) -> Self {
        self.handler = Some(Box::new(handler));
        self.handler_enabled = enabled;
        self
    }

    pub fn with_custom_handler(
        mut self,
        handler_text: impl Display,
        handler: impl FnOnce(&mut Context) + 'panel,
    ) -> Self {
        self.handler = Some(Box::new(handler));
        self.handler_text = Some(handler_text.to_string());
        self
    }

    pub fn render(self, ui: &mut Ui) {
        let theme_style = theme_style();
        let icon_size = theme_style.panel_icon_size();
        let icon_padding = (icon_size.outer - icon_size.inner) / 2.0;
        let panel_margin_size = theme_style.panel_margin_size();
        let panel_width = ui.available_width();
        let inner_panel_width = panel_width - panel_margin_size * 2.;

        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                match self.back {
                    Some(back) if self.back_enabled => {
                        let icon = CompositeIcon::new(egui_phosphor::bold::ARROW_BEND_UP_LEFT)
                            .icon_size(icon_size.inner.x)
                            .padding(Some(icon_padding));
                        if ui.add_enabled(self.back_active, icon).clicked() {
                            back(self.this);
                        }
                    }
                    _ => {
                        ui.add_space(icon_size.outer_width());
                    }
                }

                if let Some(caption) = self.caption {
                    let max_size = Vec2::new(
                        panel_width - (icon_size.outer_width() + ui.spacing().item_spacing.x) * 2.,
                        icon_size.outer_height(),
                    );

                    ui.add_sized(max_size, Label::new(WidgetText::from(caption).heading()));
                }

                match self.close {
                    Some(close) if self.close_enabled => {
                        let icon = CompositeIcon::new(egui_phosphor::bold::X)
                            .icon_size(icon_size.inner.x)
                            .padding(Some(icon_padding));
                        if ui.add_enabled(self.close_active, icon).clicked() {
                            close(self.this);
                        }
                    }
                    _ => {
                        ui.add_space(icon_size.outer_width());
                    }
                }
            });

            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    if let Some(header) = self.header {
                        ui.add_space(24.);

                        ui.vertical_centered(|ui| {
                            ui.set_width(inner_panel_width);
                            header(self.this, ui);
                        });
                    }

                    ui.add_space(24.);

                    // egui::ScrollArea::vertical()
                    //     .auto_shrink([false, true])
                    //     .show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    if let Some(body) = self.body {
                        ui.vertical_centered(|ui| {
                            ui.set_width(inner_panel_width);

                            body(self.this, ui);
                        });
                    }

                    let padding = ui.available_height() - theme_style.panel_footer_height;
                    if padding > 0. {
                        ui.add_space(padding);
                    }
                    //});

                    if let Some(footer) = self.footer {
                        footer(self.this, ui);
                    }

                    if let Some(handler) = self.handler {
                        let text = self.handler_text.as_deref();
                        if ui
                            .large_button_enabled(
                                self.handler_enabled,
                                text.unwrap_or(i18n("Continue")),
                            )
                            .clicked()
                        {
                            handler(self.this);
                        }
                    }
                });
        });
    }
}
