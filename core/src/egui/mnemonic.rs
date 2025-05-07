use crate::imports::*;
use egui_extras::{Size, StripBuilder};
use egui_phosphor::light::CLIPBOARD_TEXT;

const MNEMONIC_FONT: &str = "noto_sans_mono_light";

#[derive(Default, Clone)]
pub struct MnemonicPresenterContext {
    allow_clipboard: bool,
}

impl Zeroize for MnemonicPresenterContext {
    fn zeroize(&mut self) {
        self.allow_clipboard.zeroize();
    }
}

pub struct MnemonicPresenter<'render> {
    phrase: &'render str,
    context: &'render mut MnemonicPresenterContext,
}

impl<'render> MnemonicPresenter<'render> {
    pub fn new(phrase: &'render str, context: &'render mut MnemonicPresenterContext) -> Self {
        Self { phrase, context }
    }

    pub fn notice(&self) -> &'static str {
        "Your mnemonic phrase allows you to re-create your private key. \
        The person who has access to this mnemonic will have full control of \
        the Kaspa stored in it. Keep your mnemonic safe. Write it down and \
        store it in a safe, preferably in a fire-resistant location. Do not \
        store your mnemonic on this computer or a mobile device. This wallet \
        will never ask you for this mnemonic phrase unless you manually \
        initiate a private key recovery."
    }

    pub fn warning(&self) -> &'static str {
        "This wallet will never ask you for this mnemonic phrase unless you manually \
        initiate a private key recovery."
    }

    pub fn render(&mut self, ui: &mut Ui, caption: Option<impl Into<String>>) {
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new(i18n("Never share your mnemonic with anyone!"))
                    .color(theme_color().alert_color),
            );
            // ui.separator();
            ui.label(" ");

            if let Some(caption) = caption {
                ui.label(caption.into());
            }

            ui.label(" ");
        });

        let mut words = self.phrase.split(' ').collect::<VecDeque<&str>>();
        let available_width = ui.available_width();
        // println!("available_width: {}", available_width);
        let (font_size, relative_size, width_per_col) = if available_width < 390.0 {
            (12., 0.95, 120.)
        } else if available_width < 600.0 {
            (12., 0.95, 150.)
        } else if available_width < 850.0 {
            (14., 0.95, 165.)
        } else if available_width < 1000.0 {
            (14., 0.6, 165.)
        } else {
            (16., 0.6, 175.)
        };

        ui.horizontal(|ui| {
            let mut seq: usize = 0;
            StripBuilder::new(ui)
                .cell_layout(Layout::top_down(Align::Center))
                .size(Size::remainder())
                .size(Size::relative(relative_size))
                .size(Size::remainder())
                .horizontal(|mut strip| {
                    strip.empty();
                    strip.cell(|ui| {
                        ui.vertical(|ui| {
                            Frame::new()
                                .stroke(Stroke::new(1.0, Color32::from_black_alpha(48)))
                                .fill(Color32::from_black_alpha(32))
                                .inner_margin(16.)
                                .outer_margin(0.)
                                .corner_radius(8.)
                                .show(ui, |ui| {
                                    let frame_width = ui.available_width();
                                    let num_cols = (frame_width / width_per_col).max(1.0) as usize;
                                    let num_rows = words.len().div_ceil(num_cols);

                                    ui.set_max_height(num_rows as f32 * 32.);

                                    for _ in 0..num_rows {
                                        ui.columns(num_cols, |cols| {
                                            for col in cols.iter_mut().take(num_cols) {
                                                if let Some(word) = words.pop_front() {
                                                    col.horizontal(|ui| {
                                                        seq += 1;
                                                        Self::render_word(ui, word, seq, font_size);
                                                    });
                                                }
                                            }
                                        });
                                    }
                                });
                        });
                    });
                    strip.empty();
                });
        });

        ui.vertical_centered(|ui| {
            ui.label("");
            ui.checkbox(&mut self.context.allow_clipboard, i18n("Allow clipboard copy"));
            if self.context.allow_clipboard {
                ui.label("");
                ui.label(i18n("Please note, copying to clipboard carries a risk of exposing your mnemonic to malware."));
                ui.label("");

                if ui.medium_button(format!("{CLIPBOARD_TEXT} Copy to clipboard")).clicked() {
                    //ui.output_mut(|o| o.copied_text = self.phrase.to_string());
                    ui.ctx().copy_text(self.phrase.to_string());
                    runtime().notify_clipboard(i18n("Copied to clipboard"));
                }
            }
        });
    }

    fn render_word(ui: &mut Ui, word: &str, seq: usize, font_size: f32) {
        Frame::new()
            .stroke(Stroke::new(1.0, Color32::DARK_GRAY))
            .fill(Color32::from_black_alpha(32))
            .inner_margin(Margin {
                left: 4,
                right: 4,
                top: 10,
                bottom: 10,
            })
            .outer_margin(4.)
            .corner_radius(8.)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(format!("{seq:>2}."))
                        .size(font_size)
                        .family(FontFamily::Name(MNEMONIC_FONT.into()))
                        .color(egui::Color32::GRAY),
                );
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new(word)
                            .size(font_size)
                            .family(FontFamily::Name(MNEMONIC_FONT.into()))
                            .color(egui::Color32::WHITE),
                    );
                });
            });
    }
}
