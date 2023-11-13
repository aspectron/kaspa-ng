use crate::imports::*;
use kaspa_metrics::Metric;
// use kaspa_metrics::{Metric, MetricsSnapshot};
use chrono::DateTime;
use egui_plot::{
    // GridInput,
    Legend,
    Line,
    LineStyle,
    Plot,
    // PlotPoint,
    PlotPoints, uniform_grid_spacer,
};


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
        core: &mut Core,
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

                if let Some(metrics) = core.metrics.as_ref() {

                    for metric in Metric::list().into_iter() {
                        
                        ui.vertical(|ui| {
                            // let value = metrics.get(&metric);
                            // let caption = metrics.format(&metric, true);
                            // ui.rig
                            ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {

                                ui.label(metrics.format(&metric, true));
                            });

                            // ---
                            
                            let metrics_data = self.interop.kaspa_service().metrics_data();
                            let data = metrics_data.get(&metric).unwrap();
                            let duration = 60 * 10; // 15 min (testing)
                            let samples = if data.len() < duration { data.len() } else { duration };

                            let graph_data = data[data.len()-samples..].to_vec();
                            // let graph_data = &data[data.len()-samples..];//.to_vec();
                            // let start_time = graph_data[0].x;

                            // let graph_data = if graph_data.len() < samples {
                            //     // vec![PlotPoint::new(0.,0.),samples-graph_data.len()]//.concat(graph_data)
                            //     let len = samples-graph_data.len();
                            //     let mut vec = Vec::with_capacity(len);
                            //     for _ in 0..len {
                            //         vec.push(PlotPoint::new(0.,0.));
                            //     }
                            //     vec
                            // } else {
                            //     graph_data.to_vec()
                            // };

                            // let graph = CompositeGraph::new(metric.as_str(), &period_data);
                            // ui.add(graph);





                            // let first_point = graph_data[0];
                            // let last_point = graph_data[graph_data.len() - 1];
                            let plot = Plot::new(metric.as_str())
                                .legend(Legend::default())
                                .height(96.)
                                .auto_bounds_x()
                                .auto_bounds_y()
                                .set_margin_fraction(vec2(0.0,0.0) )
                                .y_axis_width(4)
                                .show_axes(true)
                                .show_grid(true)
                                .allow_drag([false, false])
                                .allow_scroll(false)
                                .y_axis_formatter(move |y,_size,_range|{
                                    // "".to_string()
                                    metric.format(y, true)
                                })
                                .x_axis_formatter(move |x, _size, _range| {
                                    // workflow_log::log_info!("x:{x}, size:{size}, range:{range:?}");
                                    // if x <= first_point.x || x >= last_point.x {
                                        DateTime::<chrono::Utc>::from_timestamp((x / 1000.0) as i64, 0)
                                            .expect("could not parse timestamp")
                                            .with_timezone(&chrono::Local)
                                            .format("%H:%M:%S")
                                            .to_string()
                                    // } else {
                                    //     "".to_string()
                                    // }
                                })
                                .x_grid_spacer(
                                    uniform_grid_spacer(|_input| {
                                        // let GridInput { bounds, base_step_size } = input;
                                        // println!("bounds: {:?}, base_step_size: {}", bounds, base_step_size);
                                        // [300.,60.,15.]
                                        [300.*1000.,60.*1000.,15.*1000.]
                                    })
                                )
                                .label_formatter(move |_name, point| {
                                    let PlotPoint { x, y } = point;

                                    format!("{} @ {}", metric.format(*y, true), DateTime::<chrono::Utc>::from_timestamp((*x / 1000.0) as i64, 0)
                                        .expect("could not parse timestamp")
                                        .with_timezone(&chrono::Local)
                                        .format("%H:%M:%S")
                                    )
                                })
                                ;
                    
                            let line = Line::new(PlotPoints::Owned(graph_data))
                                .color(theme().graph_color)
                                .style(LineStyle::Solid)
                                .fill(0.0);
                    
                            plot.show(ui, |plot_ui| {
                                plot_ui.line(line);
                            });

                            ui.add_space(12.);

























                        });
                    }    
                }
            });

                // CollapsingHeader::new("Kaspa Node")
                //     .default_open(true)
                //     .show(ui, |ui| {
                //         // ui.label("This is the settings page");

                //         if let Some(metrics) = core.metrics.as_ref() {

                //             ui.vertical(|ui| {

                //                 for metric in Metric::list().into_iter() {
                                    
                //                     let value = metrics.get(&metric);
                //                     let caption = metrics.format(&metric, true);
                                    
                //                     ui.horizontal(|ui| {
                //                         ui.label(caption);
                //                         ui.label(format!(" ... ({})", value));
                //                     });

                //                     // mutex!
                //                     let metrics_data = self.interop.kaspa_service().metrics_data();
                //                     let data = metrics_data.get(&metric).unwrap();
                //                     // test code
                //                     let len = 5;
                //                     let last = data.len();
                //                     let first = if last < len { 0 } else { last - len };
                //                     let samples = &data[first..last];
                //                     let text = samples.iter().map(|sample| format!("{:?}", sample)).collect::<Vec<_>>().join(", ");
                //                     ui.label(format!("[{text}]"));
                //                     ui.label(" ");
                //                 }
                //             });
                //         }



                //     });
                // });
            

        // CollapsingHeader::new("RPC Protocol")
        //     .default_open(false)
        //     .show(ui, |ui| {
        //         ui.label("This is the settings page");
        //     });
    }
}
