use kaspa_metrics::Metric;

use crate::imports::*;

pub struct Metrics {
    #[allow(dead_code)]
    interop: Interop,
}

impl Metrics {
    pub fn new(interop: Interop) -> Self {
        Self { interop }
    }
}

impl ModuleT for Metrics {

    fn style(&self) -> ModuleStyle {
        ModuleStyle::Default
    }

    fn render(
        &mut self,
        wallet: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {

        ui.heading("Metrics");
        ui.separator();

        egui::ScrollArea::vertical()
            .id_source("node_metrics")
            .auto_shrink([false; 2])
            .show(ui, |ui| {

                CollapsingHeader::new("Kaspa Node")
                    .default_open(true)
                    .show(ui, |ui| {
                        // ui.label("This is the settings page");

                        if let Some(metrics) = wallet.metrics.as_ref() {

                            ui.vertical(|ui| {

                                for metric in Metric::list().into_iter() {
                                    
                                    let value = metrics.get(&metric);
                                    let caption = metrics.format(&metric, true);
                                    
                                    ui.horizontal(|ui| {
                                        ui.label(caption);
                                        ui.label(format!(" ... ({})", value));
                                    });

                                    // mutex!
                                    let metrics_data = self.interop.kaspa_service().metrics_data();
                                    let data = metrics_data.get(&metric).unwrap();
                                    // test code
                                    let len = 5;
                                    let last = data.len();
                                    let first = if last < len { 0 } else { last - len };
                                    let samples = &data[first..last];
                                    let text = samples.iter().map(|sample| format!("{}", sample)).collect::<Vec<_>>().join(", ");
                                    ui.label(format!("[{text}]"));
                                    ui.label(" ");
                                }
                            });
                        }



                    });
                });
            

        // CollapsingHeader::new("RPC Protocol")
        //     .default_open(false)
        //     .show(ui, |ui| {
        //         ui.label("This is the settings page");
        //     });
    }
}
