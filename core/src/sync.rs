use crate::imports::*;
use kaspa_wallet_core::SyncState;

#[derive(Default)]
pub struct SyncStatus {
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
                        caption: "Syncing Proof...".to_string(),
                        ..Default::default()
                    }
                } else {
                    SyncStatus {
                        caption: format!("Syncing Proof {}", level.separated_string()),
                        ..Default::default()
                    }
                }
            }
            SyncState::Headers { headers, progress } => SyncStatus {
                caption: "Syncing Headers...".to_string(),
                progress_bar_percentage: Some(progress as f32 / 100_f32),
                progress_bar_text: Some(headers.separated_string()),
                ..Default::default()
            },
            SyncState::Blocks { blocks, progress } => SyncStatus {
                caption: "Syncing DAG Blocks...".to_string(),
                progress_bar_percentage: Some(progress as f32 / 100_f32),
                progress_bar_text: Some(blocks.separated_string()),
                ..Default::default()
            },
            SyncState::TrustSync { processed, total } => {
                let progress = processed * 100 / total;

                SyncStatus {
                    caption: "Syncing DAG Trust...".to_string(),
                    progress_bar_percentage: Some(progress as f32 / 100_f32),
                    progress_bar_text: Some(processed.separated_string()),
                    ..Default::default()
                }
            }
            SyncState::UtxoSync { total, .. } => SyncStatus {
                caption: "Syncing UTXO entries...".to_string(),
                progress_bar_text: Some(total.separated_string()),
                ..Default::default()
            },
            SyncState::UtxoResync => SyncStatus {
                caption: "Syncing...".to_string(),
                ..Default::default()
            },
            SyncState::NotSynced => SyncStatus {
                caption: "Syncing...".to_string(),
                ..Default::default()
            },
            SyncState::Synced { .. } => SyncStatus {
                caption: "Ready...".to_string(),
                synced: true,
                ..Default::default()
            },
        }
    }

    pub fn progress_bar(&self) -> Option<egui::ProgressBar> {
        if let Some(progress_bar_percentage) = self.progress_bar_percentage {
            if let Some(progress_bar_text) = &self.progress_bar_text {
                Some(egui::ProgressBar::new(progress_bar_percentage).text(progress_bar_text))
            } else {
                Some(egui::ProgressBar::new(progress_bar_percentage))
            }
        } else {
            None
        }
    }

    pub fn render_text_state(&self, ui: &mut egui::Ui) {
        ui.label(self.caption.as_str());
        if let Some(text_status) = &self.text_status {
            ui.separator();
            ui.label(text_status);
        }
    }
}
