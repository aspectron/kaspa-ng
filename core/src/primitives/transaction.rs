use crate::imports::*;
use kaspa_consensus_core::tx::{TransactionInput, TransactionOutpoint};
use kaspa_wallet_core::storage::{
    transaction::{TransactionData, UtxoRecord},
    TransactionType,
};
// use pad::*;

pub trait AsColor {
    fn as_color(&self) -> Color32;
}

impl AsColor for TransactionType {
    fn as_color(&self) -> Color32 {
        match self {
            TransactionType::Incoming => Color32::LIGHT_GREEN,
            TransactionType::Outgoing => Color32::LIGHT_RED,
            TransactionType::External => Color32::LIGHT_BLUE,
            TransactionType::Reorg => Color32::BLACK,
            TransactionType::Batch => Color32::GRAY,
        }
    }
}

pub trait RenderUtxo {
    fn render(&self, ui: &mut Ui, suffix: &str);
}

impl RenderUtxo for UtxoRecord {
    fn render(&self, ui: &mut Ui, suffix: &str) {
        let address = self
            .address
            .as_ref()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "n/a".to_string());
        // let index = self.index;
        let amount = sompi_to_kaspa_string(self.amount);

        ui.label(address);
        if self.is_coinbase {
            ui.label(format!("{} {amount} {suffix} COINBASE UTXO", ""));
        } else {
            ui.label(format!("{} {amount} {suffix} STANDARD UTXO", ""));
        }
    }
}

struct Inner {
    id: TransactionId,
    record: Mutex<Arc<TransactionRecord>>,
}

#[derive(Clone)]
pub struct Transaction {
    inner: Arc<Inner>,
}

impl Transaction {
    pub fn record(&self) -> Arc<TransactionRecord> {
        self.inner.record.lock().unwrap().clone()
    }

    pub fn id(&self) -> TransactionId {
        self.inner.id
    }

    pub fn update(&self, record: Arc<TransactionRecord>) -> Result<()> {
        *self.inner.record.lock().unwrap() = record;
        Ok(())
    }
}

impl IdT for Transaction {
    type Id = TransactionId;

    fn id(&self) -> &Self::Id {
        &self.inner.id
    }
}

impl std::fmt::Debug for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.record(), f)
    }
}

pub type TransactionCollection = Collection<TransactionId, Transaction>;

impl From<TransactionRecord> for Transaction {
    fn from(record: TransactionRecord) -> Self {
        Self {
            inner: Arc::new(Inner {
                id: *record.id(),
                record: Mutex::new(Arc::new(record)),
            }),
        }
    }
}

impl From<Arc<TransactionRecord>> for Transaction {
    fn from(record: Arc<TransactionRecord>) -> Self {
        Self {
            inner: Arc::new(Inner {
                id: *record.id(),
                record: Mutex::new(record),
            }),
        }
    }
}

impl Transaction {
    pub fn render(
        &self,
        ui: &mut Ui,
        network_type: NetworkType,
        current_daa_score: Option<u64>,
        _include_utxos: bool,
    ) {
        let record = self.record();
        let transaction_type = record.transaction_type();
        // let kind = transaction_type.style(&transaction_type.to_string());

        let maturity_progress = current_daa_score.and_then(|current_daa_score| {
            record
                .maturity_progress(current_daa_score)
                .map(|progress| format!("{}% - ", (progress * 100.) as usize))
        });

        let block_daa_score = record.block_daa_score().separated_string();
        let transaction_id = record.id().to_string();
        let short_id = transaction_id.chars().take(8).collect::<String>();

        let suffix = kaspa_suffix(&network_type);

        let (default_color, strong_color) = if ui.visuals().dark_mode {
            (Color32::LIGHT_GRAY, Color32::WHITE)
        } else {
            (Color32::DARK_GRAY, Color32::BLACK)
        };

        let font_id = FontId::monospace(12.0);

        match record.transaction_data() {
            TransactionData::Reorg {
                utxo_entries,
                aggregate_input_value,
            }
            | TransactionData::Incoming {
                utxo_entries,
                aggregate_input_value,
            }
            | TransactionData::External {
                utxo_entries,
                aggregate_input_value,
            } => {
                let aggregate_input_value = sompi_to_kaspa_string(*aggregate_input_value);

                let mut job = LayoutJobBuilder::new(8.0, Some(font_id));

                if let Some(maturity_progress) = maturity_progress {
                    job = job.text(maturity_progress.as_str(), strong_color);
                }

                let job = job
                    .text(&aggregate_input_value, transaction_type.as_color())
                    .text(&transaction_type.to_string().to_uppercase(), default_color)
                    .text(&short_id, default_color)
                    .text(format!("@{} DAA", block_daa_score).as_str(), default_color);

                CollapsingHeader::new(job)
                    .id_source(transaction_id)
                    .default_open(false)
                    .show(ui, |ui| {
                        utxo_entries.iter().for_each(|utxo_entry| {
                            utxo_entry.render(ui, suffix);
                        });
                    });
            }
            TransactionData::Outgoing {
                fees,
                aggregate_input_value,
                transaction,
                payment_value,
                change_value,
                ..
            } => {
                let job = if let Some(payment_value) = payment_value {
                    LayoutJobBuilder::new(8.0, Some(font_id.clone()))
                        .text("SEND", TransactionType::Outgoing.as_color())
                        .text(
                            &sompi_to_kaspa_string(*payment_value),
                            TransactionType::Outgoing.as_color(),
                        )
                        .text("Used:", default_color)
                        .text(&sompi_to_kaspa_string(*aggregate_input_value), strong_color)
                        .text("Fees:", default_color)
                        .text(
                            &sompi_to_kaspa_string(*fees),
                            TransactionType::Outgoing.as_color(),
                        )
                        .text("Change:", default_color)
                        .text(
                            &sompi_to_kaspa_string(*change_value),
                            TransactionType::Incoming.as_color(),
                        )
                    //     transaction.inputs.len(),
                    //     transaction.outputs.len(),
                } else {
                    LayoutJobBuilder::new(16.0, Some(font_id.clone()))
                        .text("Sweep:", default_color)
                        .text(&sompi_to_kaspa_string(*aggregate_input_value), strong_color)
                        .text("Fees:", default_color)
                        .text(
                            &sompi_to_kaspa_string(*fees),
                            TransactionType::Outgoing.as_color(),
                        )
                        .text("Change:", default_color)
                        .text(&sompi_to_kaspa_string(*change_value), strong_color)
                };

                CollapsingHeader::new(job)
                .id_source(transaction_id)
                .default_open(false)
                .show(ui, |ui| {

                    for (_n, input) in transaction.inputs.iter().enumerate() {
                        let TransactionInput {
                            previous_outpoint,
                            signature_script: _,
                            sequence,
                            sig_op_count,
                        } = input;
                        let TransactionOutpoint {
                            transaction_id,
                            index,
                        } = previous_outpoint;

                        let text = RichText::new(format!(
                            "{:>10}{sequence:>2}: {transaction_id}:{index} SigOps: {sig_op_count}",""
                        )).font(font_id.clone());

                        ui.label(text);
                    }

                });
            }
        }
    }
}
