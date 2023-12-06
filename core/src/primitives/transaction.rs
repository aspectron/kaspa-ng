use crate::imports::*;
use kaspa_consensus_core::tx::{TransactionInput, TransactionOutpoint, TransactionOutput};
use kaspa_wallet_core::storage::{
    transaction::{TransactionData, UtxoRecord},
    TransactionType,
};

pub trait AsColor {
    fn as_color(&self) -> Color32;
}

impl AsColor for TransactionType {
    fn as_color(&self) -> Color32 {
        match self {
            TransactionType::Incoming => theme_color().transaction_incoming,
            TransactionType::Outgoing => theme_color().transaction_outgoing,
            TransactionType::External => theme_color().transaction_external,
            TransactionType::Reorg => theme_color().transaction_reorg,
            TransactionType::Batch => theme_color().transaction_batch,
            TransactionType::Stasis => theme_color().transaction_stasis,
        }
    }
}

#[derive(Debug)]
struct Context {
    record: Arc<TransactionRecord>,
    maturity: Option<bool>,
}

impl Context {
    pub fn new(record: Arc<TransactionRecord>, maturity: Option<bool>) -> Self {
        Self { record, maturity }
    }
}

struct Inner {
    id: TransactionId,
    context: Mutex<Arc<Context>>,
}

impl Inner {
    fn new(record: Arc<TransactionRecord>, maturity: Option<bool>) -> Self {
        Self {
            id: *record.id(),
            context: Mutex::new(Arc::new(Context::new(record, maturity))),
        }
    }
}

#[derive(Clone)]
pub struct Transaction {
    inner: Arc<Inner>,
}

impl Transaction {
    pub fn new_confirmed(record: Arc<TransactionRecord>) -> Self {
        Self {
            inner: Arc::new(Inner::new(record, Some(true))),
        }
    }

    pub fn new_processing(record: Arc<TransactionRecord>) -> Self {
        Self {
            inner: Arc::new(Inner::new(record, Some(false))),
        }
    }

    pub fn new(record: Arc<TransactionRecord>) -> Self {
        Self {
            inner: Arc::new(Inner::new(record, None)),
        }
    }

    fn context(&self) -> Arc<Context> {
        self.inner.context.lock().unwrap().clone()
    }

    pub fn id(&self) -> TransactionId {
        self.inner.id
    }

    pub fn aggregate_input_value(&self) -> u64 {
        self.context().record.aggregate_input_value()
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
        std::fmt::Debug::fmt(&self.context(), f)
    }
}

pub type TransactionCollection = Collection<TransactionId, Transaction>;

impl From<TransactionRecord> for Transaction {
    fn from(record: TransactionRecord) -> Self {
        Self {
            inner: Arc::new(Inner::new(Arc::new(record), None)),
        }
    }
}

impl From<Arc<TransactionRecord>> for Transaction {
    fn from(record: Arc<TransactionRecord>) -> Self {
        Self {
            inner: Arc::new(Inner::new(record, None)),
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
        largest: Option<u64>,
    ) {
        let ppp = ui.ctx().pixels_per_point();
        let width = ui.available_width() / ppp;

        let Context { record, maturity } = &*self.context();

        let padding = 9 + largest
            .map(|largest| sompi_to_kaspa(largest).trunc().separated_string().len())
            .unwrap_or_default();

        let ps2k = |sompi| padded_sompi_to_kaspa_string_with_suffix(sompi, &network_type, padding);
        let s2k = |sompi| sompi_to_kaspa_string_with_suffix(sompi, &network_type);

        let timestamp = record
            .unixtime_as_locale_string()
            .unwrap_or_else(|| format!("@{} DAA", record.block_daa_score().separated_string()));
        let block_daa_score = record.block_daa_score().separated_string();
        let transaction_id = record.id().to_string();

        let default_color = theme_color().default_color;
        let strong_color = theme_color().strong_color;

        let font_id_header = FontId::monospace(15.0);
        let font_id_content = FontId::monospace(15.0);
        let icon_font_id = FontId::proportional(18.0);

        let header = LayoutJobBuilderSettings::new(width, 8.0, Some(font_id_content.clone()));
        let content = LayoutJobBuilderSettings::new(width, 8.0, Some(font_id_content.clone()));

        match record.transaction_data() {
            TransactionData::Reorg {
                utxo_entries,
                aggregate_input_value,
            }
            | TransactionData::Stasis {
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
                let aggregate_input_value = ps2k(*aggregate_input_value);
                let mut job = LayoutJobBuilder::new(width, 8.0, Some(font_id_header.clone()))
                    .with_icon_font(icon_font_id);
                job = job.icon(
                    egui_phosphor::light::ARROW_SQUARE_RIGHT,
                    TransactionType::Incoming.as_color(),
                );

                if maturity.unwrap_or(false) {
                    let maturity_progress = current_daa_score.and_then(|current_daa_score| {
                        record
                            .maturity_progress(current_daa_score)
                            .map(|progress| format!("{}% - ", (progress * 100.) as usize))
                    });

                    if let Some(maturity_progress) = maturity_progress {
                        job = job.text(maturity_progress.as_str(), strong_color);
                    }
                }

                job = job
                    .text(timestamp.as_str(), default_color)
                    .text(&aggregate_input_value, TransactionType::Incoming.as_color());

                // ui.LayoutJobBuilder::new(width,8.0(&transaction_id, false, |ui,state| {
                //     ui.horizontal( |ui| {

                //         let icon = RichText::new(egui_phosphor::light::ARROW_SQUARE_RIGHT).color(TransactionType::Incoming.as_color());
                //         if ui.add(Label::new(icon).sense(Sense::click())).clicked() {
                //             *state = !*state;
                //         }

                //         let mut text = LayoutJob::default();
                //         text.append(&block_daa_score.as_str(), 0., TextFormat {
                //             color: default_color,
                //             font_id: font_id_header.clone(),
                //             ..Default::default()
                //         });
                //         text.halign = Align::Min;
                //         text.break_on_newline = false;
                //         // if ui.add_sized(vec2(200.,12.), Label::new(text).sense(Sense::click())).clicked() {
                //         if ui.add(Label::new(text).sense(Sense::click())).clicked() {
                //             *state = !*state;
                //         }

                //         ui.with_layout(Layout::top_down(Align::Min).with_cross_align(Align::Max), |ui|{
                //             let mut text = LayoutJob::default();
                //             text.append(&aggregate_input_value, 0., TextFormat {
                //                 color : transaction_type.as_color(),
                //                 font_id: font_id_header.clone(),
                //                 ..Default::default()
                //             });
                //             if ui.add_sized(vec2(200.,12.), Label::new(text).sense(Sense::click())).clicked() {
                //                 *state = !*state;
                //             }
                //         });
                //     });
                // }, |ui| {

                CollapsingHeader::new(job)
                    .id_source(&transaction_id)
                    .icon(paint_header_icon)
                    .default_open(false)
                    .show(ui, |ui| {
                        ljb(&content)
                            .padded(15, "Transaction id:", default_color)
                            .text(&transaction_id, default_color)
                            .label(ui);

                        ljb(&content)
                            .padded(15, "Received at:", default_color)
                            .text(&format!("{} DAA", block_daa_score), default_color)
                            .label(ui);

                        utxo_entries.iter().for_each(|utxo_entry| {
                            let UtxoRecord {
                                index: _,
                                address,
                                amount,
                                script_public_key,
                                is_coinbase,
                            } = utxo_entry;
                            let address = address
                                .as_ref()
                                .map(|addr| addr.to_string())
                                .unwrap_or_else(|| "n/a".to_string());

                            ljb(&content).text(&address, default_color).label(ui);

                            if *is_coinbase {
                                ljb(&content)
                                    .text(
                                        &format!("{} - Coinbase UTXO", s2k(*amount)),
                                        default_color,
                                    )
                                    .label(ui);
                            } else {
                                ljb(&content)
                                    .text(
                                        &format!("{} - Standard UTXO", s2k(*amount)),
                                        default_color,
                                    )
                                    .label(ui);
                            }

                            ljb(&content)
                                .text(
                                    &format!("Script: {}", script_public_key.script_as_hex()),
                                    default_color,
                                )
                                .label(ui);
                        });
                    });
            }
            TransactionData::Outgoing {
                fees,
                aggregate_input_value,
                transaction,
                payment_value,
                change_value,
                accepted_daa_score,
                ..
            } => {
                let job = if let Some(payment_value) = payment_value {
                    let mut job = ljb(&header) 
                        .with_icon_font(icon_font_id);

                    job = job
                        .icon(
                            egui_phosphor::light::ARROW_SQUARE_LEFT,
                            TransactionType::Outgoing.as_color(),
                        )
                        .text(timestamp.as_str(), default_color)
                        .text(
                            &ps2k(*payment_value + *fees),
                            TransactionType::Outgoing.as_color(),
                        );

                    if !maturity.unwrap_or(true) {
                        job = job.text("Submitting...", strong_color);
                    }

                    job
                } else {
                    ljb(&header)
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

                // ui.collapsable(&transaction_id, false, |ui,state| {
                //     ui.horizontal( |ui| {

                //         let icon = RichText::new(egui_phosphor::light::ARROW_SQUARE_LEFT).color(TransactionType::Outgoing.as_color());
                //         if ui.add(Label::new(icon).sense(Sense::click())).clicked() {
                //             *state = !*state;
                //         }

                //         let mut text = LayoutJob::default();
                //         text.append(&block_daa_score.as_str(), 0., TextFormat {
                //             color: default_color,
                //             font_id: font_id_header.clone(),
                //             ..Default::default()
                //         });
                //         text.halign = Align::Min;
                //         // if ui.add_sized(vec2(200.,12.),Label::new(text).sense(Sense::click())).clicked() {
                //         if ui.add(Label::new(text).sense(Sense::click())).clicked() {
                //             *state = !*state;
                //         }

                //         if let Some(payment_value) = payment_value {
                //             let mut text = LayoutJob::default();
                //             text.append(&ps2k(*payment_value + *fees), 0., TextFormat {
                //                 color : transaction_type.as_color(),
                //                 font_id: font_id_header.clone(),
                //                 ..Default::default()
                //             });
                //             text.halign = Align::Max;
                //             if ui.add_sized(vec2(200.,12.), Label::new(text).sense(Sense::click())).clicked() {
                //             // if ui.add(Label::new(text).sense(Sense::click())).clicked() {
                //                 *state = !*state;
                //             }
                //         } else {
                //             // LayoutJobBuilder::new(16.0, Some(font_id_header.clone()))
                //             // .text("Sweep:", default_color)
                //             // .text(&sompi_to_kaspa_string(*aggregate_input_value), strong_color)
                //             // .text("Fees:", default_color)
                //             // .text(
                //             //     &sompi_to_kaspa_string(*fees),
                //             //     TransactionType::Outgoing.as_color(),
                //             // )
                //             // .text("Change:", default_color)
                //             // .text(&sompi_to_kaspa_string(*change_value), strong_color)
                //         }

                //         if !maturity.unwrap_or(true) {
                //             ui.spinner();
                //         }

                //         // Spinner::
                //         // Spinner::new().paint_at(ui,response.rect);
                //     });
                // }, |ui| {

                // })
                let mut collapsing_header = CollapsingHeader::new(job)
                    .id_source(&transaction_id)
                    .icon(paint_header_icon)
                    .default_open(false);
                if !maturity.unwrap_or(true) {
                    collapsing_header = collapsing_header.icon(|ui, _rect, response| {
                        Spinner::new().paint_at(ui, response.rect.expand(4.));
                    });
                }
                collapsing_header.show(ui, |ui| {
                    let width = ui.available_width() - 64.0;

                    ljb(&content)
                        .text(
                            &format!("{}: {}", "Transaction id", transaction_id),
                            default_color,
                        )
                        .label(ui);

                    ljb(&content)
                        .padded(15, "Submitted at:", default_color)
                        .text(&format!("{} DAA", block_daa_score), default_color)
                        .label(ui);

                    if let Some(accepted_daa_score) = accepted_daa_score {
                        ljb(&content)
                            .padded(15, "Accepted at:", default_color)
                            .text(
                                &format!("{} DAA", accepted_daa_score.separated_string()),
                                default_color,
                            )
                            .label(ui);
                    }

                    if let Some(payment_value) = payment_value {
                        ljb(&content)
                            .padded(15, "Amount:", default_color)
                            .text(&ps2k(*payment_value), TransactionType::Outgoing.as_color())
                            .label(ui);
                    }

                    ljb(&content)
                        .padded(14, "Fees:", default_color)
                        .text(&ps2k(*fees), TransactionType::Outgoing.as_color())
                        .label(ui);

                    ljb(&content)
                        .padded(15, "Inputs:", default_color)
                        .text(&ps2k(*aggregate_input_value), strong_color)
                        .label(ui);

                    ljb(&content)
                        .padded(15, "Change:", default_color)
                        .text(&ps2k(*change_value), TransactionType::Incoming.as_color())
                        .label(ui);

                    ljb(&content)
                        .text(
                            &format!("{} UTXO inputs", transaction.inputs.len()),
                            default_color,
                        )
                        .label(ui);

                    for input in transaction.inputs.iter() {
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

                        ljb(&content)
                            .text(
                                &format!(
                                    "  {sequence:>2}: {}:{index} SigOps: {sig_op_count}",
                                    transaction_id.to_string()
                                ),
                                default_color,
                            )
                            .label(ui);
                    }

                    let text = LayoutJobBuilder::new(width, 16.0, Some(font_id_content.clone()))
                        .text(
                            &format!("{} UTXO outputs:", transaction.outputs.len()),
                            default_color,
                        );
                    ui.label(text);

                    for output in transaction.outputs.iter() {
                        let TransactionOutput {
                            value,
                            script_public_key,
                        } = output;

                        ljb(&content)
                            .text(
                                &format!(
                                    "  {} {}",
                                    ps2k(*value),
                                    script_public_key.script_as_hex()
                                ),
                                default_color,
                            )
                            .label(ui);
                    }
                });
            }
        }
    }
}

#[inline]
pub fn sompi_to_kaspa(sompi: u64) -> f64 {
    sompi as f64 / SOMPI_PER_KASPA as f64
}

#[inline]
pub fn kaspa_to_sompi(kaspa: f64) -> u64 {
    (kaspa * SOMPI_PER_KASPA as f64) as u64
}

#[inline]
pub fn sompi_to_kaspa_string(sompi: u64) -> String {
    separated_float!(format!("{:.8}", sompi_to_kaspa(sompi)))
}
#[inline]
pub fn padded_sompi_to_kaspa_string(sompi: u64, padding: usize) -> String {
    separated_float!(format!("{:.8}", sompi_to_kaspa(sompi)))
        .pad_to_width_with_alignment(padding, Alignment::Right)
}

pub fn kaspa_suffix(network_type: &NetworkType) -> &'static str {
    match network_type {
        NetworkType::Mainnet => "KAS",
        NetworkType::Testnet => "TKAS",
        NetworkType::Simnet => "SKAS",
        NetworkType::Devnet => "DKAS",
    }
}

#[inline]
pub fn sompi_to_kaspa_string_with_suffix(sompi: u64, network_type: &NetworkType) -> String {
    let kas = sompi_to_kaspa(sompi).separated_string();
    let suffix = kaspa_suffix(network_type);
    format!("{kas} {suffix}")
}

#[inline]
pub fn padded_sompi_to_kaspa_string_with_suffix(
    sompi: u64,
    network_type: &NetworkType,
    padding: usize,
) -> String {
    let kas = padded_sompi_to_kaspa_string(sompi, padding);
    let suffix = kaspa_suffix(network_type);
    format!("{kas} {suffix}")
}

// pub fn shorten(s: impl Into<String>) -> String {
//     s.into()
//     // let s: String = s.into();
//     // s.chars().take(10).collect::<String>() + "..."
// }

pub fn paint_header_icon(ui: &mut Ui, openness: f32, response: &Response) {
    let visuals = ui.style().interact(response);

    let rect = response.rect;
    let rect = Rect::from_center_size(rect.center(), vec2(rect.width(), rect.height()) * 0.75);
    let rect = rect.expand(visuals.expansion);
    let mut points = vec![rect.left_top(), rect.right_top(), rect.center_bottom()];
    use std::f32::consts::TAU;
    let rotation = emath::Rot2::from_angle(remap(openness, 0.0..=1.0, -TAU / 4.0..=0.0));
    for p in &mut points {
        *p = rect.center() + rotation * (*p - rect.center());
    }

    ui.painter().add(Shape::convex_polygon(
        points,
        visuals.fg_stroke.color,
        Stroke::NONE,
    ));
}
