use crate::imports::*;
use kaspa_consensus_core::tx::{TransactionInput, TransactionOutpoint};
use kaspa_wallet_core::storage::transaction::TransactionData;
use pad::*;

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

impl Transaction {
    pub fn render(
        &self,
        ui: &mut Ui,
        network_type: NetworkType,
        current_daa_score: Option<u64>,
        include_utxos: bool,
    ) {
        let record = self.record();
        let transaction_type = record.transaction_type();
        // let kind = transaction_type.style(&transaction_type.to_string());

        let maturity_progress = current_daa_score
            .and_then(|current_daa_score| {
                record
                    .maturity_progress(current_daa_score)
                    .map(|progress| format!("{}% - ", (progress * 100.) as usize))
            })
            .unwrap_or_default()
            .pad_to_width_with_alignment(7, Alignment::Right);
        // .flatten();

        let block_daa_score = record.block_daa_score().separated_string();
        // let state = state.unwrap_or(&maturity);
        let mut lines = vec![format!(
            "{}{} @{block_daa_score} DAA - {transaction_type}",
            maturity_progress,
            record.id()
        )];

        let suffix = kaspa_suffix(&network_type);

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
                // .as_str();
                lines.push(format!(
                    "{:>10}UTXOs: {}  Total: {}",
                    "",
                    utxo_entries.len(),
                    aggregate_input_value
                ));
                if include_utxos {
                    for utxo_entry in utxo_entries {
                        let address = utxo_entry
                            .address
                            .as_ref()
                            .map(|addr| addr.to_string())
                            .unwrap_or_else(|| "n/a".to_string());
                        let index = utxo_entry.index;
                        let is_coinbase = if utxo_entry.is_coinbase {
                            format!("coinbase utxo [{index}]")
                        } else {
                            format!("standard utxo [{index}]")
                        };
                        let amount = sompi_to_kaspa_string(utxo_entry.amount);

                        lines.push(format!("{:>10}{address}", ""));
                        lines.push(format!("{:>10}{amount} {suffix} {is_coinbase}", ""));
                    }
                }
            }
            TransactionData::Outgoing {
                fees,
                aggregate_input_value,
                transaction,
                payment_value,
                change_value,
                ..
            } => {
                if let Some(payment_value) = payment_value {
                    lines.push(format!(
                        "{:>10}Payment: {}  Used: {}  Fees: {}  Change: {}  UTXOs: [{}↠{}]",
                        "",
                        sompi_to_kaspa_string(*payment_value),
                        sompi_to_kaspa_string(*aggregate_input_value),
                        sompi_to_kaspa_string(*fees),
                        sompi_to_kaspa_string(*change_value),
                        transaction.inputs.len(),
                        transaction.outputs.len(),
                    ));
                } else {
                    lines.push(format!(
                        "{:>10}Sweep: {}  Fees: {}  Change: {}  UTXOs: [{}↠{}]",
                        "",
                        sompi_to_kaspa_string(*aggregate_input_value),
                        sompi_to_kaspa_string(*fees),
                        sompi_to_kaspa_string(*change_value),
                        transaction.inputs.len(),
                        transaction.outputs.len(),
                    ));
                }

                if include_utxos {
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

                        lines.push(format!(
                            "{:>10}{sequence:>2}: {transaction_id}:{index} SigOps: {sig_op_count}",
                            ""
                        ));
                        // lines.push(format!("{:>4}{:>2}  Sig Ops: {sig_op_count}", "", ""));
                        // lines.push(format!("{:>4}{:>2}   Script: {}", "", "", signature_script.to_hex()));
                    }
                }
            }
        }

        for line in lines.iter() {
            // ui.label(line);
            ui.label(egui::RichText::new(line).size(14.).raised());
        }
    }
}
