use crate::imports::*;
use crate::interop::services::kaspa::MAX_METRICS_SAMPLES;
use egui_extras::{StripBuilder, Size};
use kaspa_metrics::{Metric,MetricGroup, MetricsSnapshot};
use chrono::DateTime;
use egui_plot::{
    Legend,
    Line,
    LineStyle,
    Plot,
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
        let theme = theme();

        ui.heading("Node Metrics");
        ui.separator();

        let mut graph_columns = core.settings.ux.metrics.graph_columns;
        let mut graph_height = core.settings.ux.metrics.graph_height;
        let mut graph_duration = core.settings.ux.metrics.graph_duration;

        ui.horizontal(|ui| {
            ui.label("Columns:");
            ui.add(
                Slider::new(&mut graph_columns, 
                    1..=8)
                    .orientation(SliderOrientation::Horizontal)
                    .step_by(1.)
            );
            ui.label("Graph Height:");
            ui.add(
                Slider::new(&mut graph_height, 
                    1..=1200)
                    .orientation(SliderOrientation::Horizontal)
                    .suffix("px")
            );
            ui.label("Duration:");
            ui.add(
                Slider::new(&mut graph_duration, 
                    15..=MAX_METRICS_SAMPLES)
                    .logarithmic(true)
                    .orientation(SliderOrientation::Horizontal)
                    .custom_formatter(|v, _range| {
                        format_duration(v as u64)
                    })
            );
        });

        if graph_columns != core.settings.ux.metrics.graph_columns 
        || graph_height != core.settings.ux.metrics.graph_height 
        || graph_duration != core.settings.ux.metrics.graph_duration {
            core.settings.ux.metrics.graph_columns = graph_columns;
            core.settings.ux.metrics.graph_height = graph_height;
            core.settings.ux.metrics.graph_duration = graph_duration;
            // TODO - post an application loop to relay to interop
            // so that we can combine multiple saves into one
            core.settings.store_sync().ok();
        }



        egui::ScrollArea::vertical()
            .id_source("node_metrics")
            .auto_shrink([false; 2])
            .show(ui, |ui| {

                let view_width = ui.available_width() - 32.;
                let graph_height = core.settings.ux.metrics.graph_height as f32;
                let graph_width = view_width / core.settings.ux.metrics.graph_columns as f32;

                if let Some(metrics) = core.metrics.as_ref() {

                    let mut metric_iter = Metric::list().into_iter();

                    let mut draw = true;
                    while draw {
                        ui.horizontal(|ui| {
                            for _ in 0..core.settings.ux.metrics.graph_columns {
                                if let Some(metric) = metric_iter.next() {
                                    // let duration = 60 * 10; // 15 min (testing)
                                    let duration = core.settings.ux.metrics.graph_duration;
                                    self.render_metric(ui,metric,metrics,theme,duration,graph_width,graph_height);
                                } else {
                                    draw = false;
                                }
                            }
                        });
                    }
                }
            });

    }
}

impl Metrics {

    #[allow(clippy::too_many_arguments)]
    fn render_metric(
        &mut self, 
        ui : &mut Ui, 
        metric : Metric, 
        metrics : &MetricsSnapshot, 
        theme : &Theme, 
        duration : usize, 
        graph_width : f32, 
        graph_height : f32
    ) {

        let group = MetricGroup::from(metric);
        let graph_color = match group {
            MetricGroup::System => theme.performance_graph_color,
            MetricGroup::Storage => theme.storage_graph_color,
            MetricGroup::Node => theme.node_graph_color,
            MetricGroup::Network => theme.network_graph_color,
        };

        StripBuilder::new(ui)
            .size(Size::exact(graph_width))
            .horizontal(|mut strip| {

                strip.cell(|ui| {
                ui.vertical(|ui| {
                    ui.add_space(8.);
                    ui.horizontal(|ui|{
                        ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                            ui.label(format!("{}: {}", i18n(metric.title()), metric.format(metrics.get(&metric), true, false)));
                        });
                    });

                    // ---
                    let graph_data = {
                        let metrics_data = self.interop.kaspa_service().metrics_data();
                        let data = metrics_data.get(&metric).unwrap();
                        let samples = if data.len() < duration { data.len() } else { duration };
                        data[data.len()-samples..].to_vec()
                    };

                    let mut plot = Plot::new(metric.as_str())
                        .legend(Legend::default())
                        .width(graph_width)
                        .height(graph_height)
                        .auto_bounds_x()
                        .auto_bounds_y()
                        .set_margin_fraction(vec2(0.0,0.0) )
                        .y_axis_width(4)
                        .show_axes(true)
                        .show_grid(true)
                        .allow_drag([false, false])
                        .allow_scroll(false)
                        .y_axis_formatter(move |y,_size,_range|{
                            metric.format(y, true, true)
                        })
                        .x_axis_formatter(move |x, _size, _range| {
                            DateTime::<chrono::Utc>::from_timestamp((x / 1000.0) as i64, 0)
                                .expect("could not parse timestamp")
                                .with_timezone(&chrono::Local)
                                .format("%H:%M:%S")
                                .to_string()
                        })
                        .x_grid_spacer(
                            uniform_grid_spacer(move |input| {

                                let (start_time,stop_time) = input.bounds;
                                let range = stop_time - start_time;
                                let base_step_size = range / graph_width as f64 * 64.;
                                calculate_grid_lines(base_step_size)
                            })
                        )
                        .label_formatter(move |_name, point| {
                            let PlotPoint { x, y } = point;

                            format!("{} @ {}", metric.format(*y, true, true), DateTime::<chrono::Utc>::from_timestamp((*x / 1000.0) as i64, 0)
                                .expect("could not parse timestamp")
                                .with_timezone(&chrono::Local)
                                .format("%H:%M:%S")
                            )
                        })                                                    
                        ;

                    if [Metric::CpuUsage].contains(&metric) {
                        plot = plot.include_y(100.);
                    }
            
                    if [
                        Metric::ResidentSetSizeBytes, 
                        Metric::VirtualMemorySizeBytes,
                        Metric::FdNum,
                        // Metric::DiskIoReadBytes,
                        // Metric::DiskIoWriteBytes,
                        Metric::DiskIoReadPerSec,
                        Metric::DiskIoWritePerSec,
                        Metric::Tps,
                    ].contains(&metric) {
                        plot = plot.include_y(100.);
                    }
            
                    let line = Line::new(PlotPoints::Owned(graph_data))
                        .color(graph_color)
                        .style(LineStyle::Solid)
                        .fill(0.0);
            
                        plot.show(ui, |plot_ui| {
                            plot_ui.line(line);
                        });
                    });
                });
            });
    }
}

fn calculate_grid_lines(base_step_size : f64) -> [f64; 3] {
    let mut small_grid = 15.*1000_f64;
    let mut medium_grid = 30.*1000_f64;
    let mut large_grid = 60.*1000_f64;

    while small_grid < base_step_size {
        small_grid *= 2.;
    }

    while medium_grid < small_grid  {
    // while medium_grid < base_step_size  {
        medium_grid *= 2.;
    }

    while large_grid < medium_grid {
        large_grid *= 2.;
    }

    [small_grid, medium_grid, large_grid]
}

fn format_duration(seconds: u64) -> String {
    const SECONDS_IN_MINUTE: u64 = 60;
    const MINUTES_IN_HOUR: u64 = 60;
    const HOURS_IN_DAY: u64 = 24;

    if seconds < SECONDS_IN_MINUTE {
        format_duration_unit(seconds, "sec")
    } else if seconds < SECONDS_IN_MINUTE * MINUTES_IN_HOUR {
        format_duration_unit(seconds / SECONDS_IN_MINUTE, "min")
    } else if seconds < SECONDS_IN_MINUTE * MINUTES_IN_HOUR * HOURS_IN_DAY {
        format_duration_unit(seconds / (SECONDS_IN_MINUTE * MINUTES_IN_HOUR), "hr")
    } else {
        format_duration_unit(seconds / (SECONDS_IN_MINUTE * MINUTES_IN_HOUR * HOURS_IN_DAY), "day")
    }
}

fn format_duration_unit(value: u64, unit: &str) -> String {
    if value > 1 {
        format!("{} {}s", value, unit)
    } else {
        format!("{} {}", value, unit)
    }
}
