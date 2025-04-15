use crate::imports::*;
use kaspa_wallet_core::events::SyncState;

const SYNC_STAGES: usize = 5;

#[derive(Default)]
pub struct SyncStatus {
    pub stage: Option<usize>,
    pub caption: String,
    pub text_status: Option<String>,
    pub progress_bar_percentage: Option<f32>,
    pub progress_bar_text: Option<String>,
    pub synced: bool,
}

impl SyncStatus {
    pub fn try_from(state: &SyncState) -> Self {
        match state.clone() {
            SyncState::Proof { level } => {
                if level == 0 {
                    SyncStatus {
                        stage: Some(1),
                        caption: i18n("Syncing Cryptographic Proof...").to_string(),
                        ..Default::default()
                    }
                } else {
                    SyncStatus {
                        stage: Some(1),
                        caption: format!(
                            "{} {}",
                            i18n("Syncing Cryptographic Proof..."),
                            level.separated_string()
                        ),
                        ..Default::default()
                    }
                }
            }
            SyncState::Headers { headers, progress } => SyncStatus {
                stage: Some(2),
                caption: format!(
                    "{} {}",
                    i18n("Syncing Headers..."),
                    headers.separated_string()
                ),
                progress_bar_percentage: Some(progress as f32 / 100_f32),
                progress_bar_text: Some(format!("{}%", progress)),
                ..Default::default()
            },
            SyncState::Blocks { blocks, progress } => SyncStatus {
                stage: Some(3),
                caption: format!(
                    "{} {}",
                    i18n("Syncing DAG Blocks..."),
                    blocks.separated_string()
                ),
                // caption: "Syncing DAG Blocks...".to_string(),
                progress_bar_percentage: Some(progress as f32 / 100_f32),
                progress_bar_text: Some(format!("{}%", progress)),
                ..Default::default()
            },
            SyncState::TrustSync { processed, total } => {
                let progress = processed * 100 / total;

                SyncStatus {
                    stage: Some(4),
                    caption: format!(
                        "{} {}",
                        i18n("Syncing DAG Trust..."),
                        processed.separated_string()
                    ),
                    // caption: "Syncing DAG Trust...".to_string(),
                    progress_bar_percentage: Some(progress as f32 / 100_f32),
                    progress_bar_text: Some(format!("{}%", progress)),
                    // progress_bar_text: Some(processed.separated_string()),
                    ..Default::default()
                }
            }
            SyncState::UtxoSync { total, .. } => SyncStatus {
                stage: Some(5),
                caption: format!(
                    "{} {}",
                    i18n("Syncing UTXO entries..."),
                    total.separated_string()
                ),
                // caption: "Syncing UTXO entries...".to_string(),
                // progress_bar_text: Some(total.separated_string()),
                ..Default::default()
            },
            SyncState::UtxoResync => SyncStatus {
                caption: i18n("Syncing...").to_string(),
                ..Default::default()
            },
            SyncState::NotSynced => SyncStatus {
                caption: i18n("Syncing...").to_string(),
                ..Default::default()
            },
            SyncState::Synced => SyncStatus {
                caption: i18n("Ready...").to_string(),
                synced: true,
                ..Default::default()
            },
        }
    }

    pub fn progress_bar(&self, ui: &mut egui::Ui) -> Option<egui::ProgressBar> {
        let progress_color = theme_color().progress_color;
        if let Some(progress_bar_percentage) = self.progress_bar_percentage {
            if let Some(progress_bar_text) = &self.progress_bar_text {
                ui.style_mut().visuals.override_text_color = Some(theme_color().raised_text_color);
                Some(
                    egui::ProgressBar::new(progress_bar_percentage)
                        .fill(progress_color)
                        .text(progress_bar_text),
                )
            } else {
                Some(egui::ProgressBar::new(progress_bar_percentage).fill(progress_color))
            }
        } else {
            None
        }
    }

    pub fn render_text_state(&self, ui: &mut egui::Ui) {
        if let Some(stage) = self.stage {
            ui.label(format!(
                "{} {stage} {} {SYNC_STAGES}",
                i18n("Stage"),
                i18n("of")
            ));
            ui.separator();
        }
        ui.label(self.caption.as_str());
        if let Some(text_status) = &self.text_status {
            ui.separator();
            ui.label(text_status);
        }
    }
}
