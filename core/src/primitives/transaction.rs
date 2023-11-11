use crate::imports::*;
use kaspa_consensus_core::tx::{TransactionInput, TransactionOutpoint};
use kaspa_wallet_core::storage::transaction::TransactionData;
use pad::*;
// pub struct Context {
//     // qr: Arc<RetainedImage>,
//     // qr: Arc<String>,
//     qr: load::Bytes,
//     receive_address: String,
// }

// impl Context {
//     pub fn new(descriptor: &AccountDescriptor) -> Option<Arc<Self>> {
//         // if account.wallet().network_id().is_ok() {
//         if let Some(receive_address) = descriptor.receive_address().map(String::from) {
//             // let receive_address = account.receive_address().unwrap().to_string();
//             let qr = render_qrcode(&receive_address, 200, 200);
//             Some(Arc::new(Self {
//                 qr: qr.as_bytes().to_vec().into(),
//                 receive_address,
//             }))
//         } else {
//             None
//         }
//     }

//     pub fn address(&self) -> &str {
//         self.receive_address.as_str()
//     }

//     pub fn qr(&self) -> load::Bytes {
//         self.qr.clone()
//     }
// }

struct Inner {
    // runtime: Arc<dyn runtime::Account>,
    id: TransactionId,
    // balance: Mutex<Option<Balance>>,
    // utxo_sizes: Mutex<Option<(usize, usize)>>,
    record: Mutex<Arc<TransactionRecord>>,
    // context: Mutex<Option<Arc<Context>>>,
}

impl Inner {
    // fn new(record: TransactionRecord) -> Self {
    //     // let context = Context::new(&record);
    //     Self {
    //         id: *record.id(),
    //         // balance: Mutex::new(None),
    //         // utxo_sizes: Mutex::new(None),
    //         record: Mutex::new(record),
    //         // context: Mutex::new(context),
    //     }
    // }

    // fn record(&self) -> MutexGuard<'_, TransactionRecord> {
    //     self.record.lock().unwrap()
    // }
}

#[derive(Clone)]
pub struct Transaction {
    inner: Arc<Inner>,
}

// impl From<AccountDescriptor> for Account {
//     fn from(descriptor: AccountDescriptor) -> Self {
//         Self {
//             inner: Arc::new(Inner::new(descriptor)),
//         }
//     }
// }

impl Transaction {
    // pub fn runtime(&self) -> Arc<dyn runtime::Account> {
    //     self.inner.runtime.clone()
    // }

    // pub fn record(&self) -> MutexGuard<'_, TransactionRecord> {
    //     self.inner.record()
    // }

    pub fn record(&self) -> Arc<TransactionRecord> {
        self.inner.record.lock().unwrap().clone()
    }

    pub fn id(&self) -> TransactionId {
        self.inner.id
    }

    // pub fn name_or_id(&self) -> String {
    //     self.descriptor().name_or_id()
    // }

    // pub fn balance(&self) -> Option<Balance> {
    //     self.inner.balance.lock().unwrap().clone()
    // }

    // pub fn utxo_sizes(&self) -> Option<(usize,usize)> {
    //     self.inner.utxo_sizes.lock().unwrap().clone()
    // }

    // pub fn address(&self) -> Result<String> {
    //     self.inner.context.lock().unwrap().receive_address
    //     Ok(self.inner.runtime.receive_address()?.into())
    // }

    // pub fn context(&self) -> Option<Arc<Context>> {
    //     self.inner.context.lock().unwrap().clone()
    // }

    pub fn update(&self, record: Arc<TransactionRecord>) -> Result<()> {
        // *self.inner.context.lock().unwrap() = Context::new(&descriptor);
        *self.inner.record.lock().unwrap() = record;

        Ok(())
    }

    // pub fn update_balance(&self, balance : Option<Balance>, mature_utxo_size : usize, pending_utxo_size : usize) -> Result<()> {
    //     *self.inner.balance.lock().unwrap() = balance;
    //     *self.inner.utxo_sizes.lock().unwrap() = Some((mature_utxo_size,pending_utxo_size));

    //     Ok(())

    // }
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
        // core : &Core,
        // wallet: &Arc<Wallet>,
        // state: Option<&str>,
        network_type: NetworkType,
        current_daa_score: Option<u64>,
        include_utxos: bool,
        // history: bool,
        // account: Option<Arc<dyn runtime::Account>>,
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
                    "{:>4}UTXOs: {}  Total: {}",
                    "",
                    utxo_entries.len(),
                    aggregate_input_value
                ));
                if include_utxos {
                    for utxo_entry in utxo_entries {
                        let address = style(
                            utxo_entry
                                .address
                                .as_ref()
                                .map(|addr| addr.to_string())
                                .unwrap_or_else(|| "n/a".to_string()),
                        )
                        .blue();
                        let index = utxo_entry.index;
                        let is_coinbase = if utxo_entry.is_coinbase {
                            style(format!("coinbase utxo [{index}]")).dim()
                        } else {
                            style(format!("standard utxo [{index}]")).dim()
                        };
                        let amount = sompi_to_kaspa_string(utxo_entry.amount);

                        lines.push(format!("{:>4}{address}", ""));
                        lines.push(format!("{:>4}{amount} {suffix} {is_coinbase}", ""));
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
                        "{:>4}Payment: {}  Used: {}  Fees: {}  Change: {}  UTXOs: [{}↠{}]",
                        "",
                        style(sompi_to_kaspa_string(*payment_value)).red(),
                        style(sompi_to_kaspa_string(*aggregate_input_value)).blue(),
                        style(sompi_to_kaspa_string(*fees)).red(),
                        style(sompi_to_kaspa_string(*change_value)).green(),
                        transaction.inputs.len(),
                        transaction.outputs.len(),
                    ));
                } else {
                    lines.push(format!(
                        "{:>4}Sweep: {}  Fees: {}  Change: {}  UTXOs: [{}↠{}]",
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
                            "{:>4}{sequence:>2}: {transaction_id}:{index} SigOps: {sig_op_count}",
                            ""
                        ));
                        // lines.push(format!("{:>4}{:>2}  Sig Ops: {sig_op_count}", "", ""));
                        // lines.push(format!("{:>4}{:>2}   Script: {}", "", "", signature_script.to_hex()));
                    }
                }
            }
        }

        for line in lines.iter() {
            ui.label(line);
        }
    }
}
