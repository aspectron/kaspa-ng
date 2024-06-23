use crate::imports::*;
use crate::runtime::services::metrics_monitor::MAX_METRICS_SAMPLES;
use egui_extras::{StripBuilder, Size};
use kaspa_metrics_core::{Metric,MetricGroup, MetricsSnapshot};
use chrono::DateTime;
use egui_plot::{
    Legend,
    Line,
    LineStyle,
    Plot,
    PlotPoints, uniform_grid_spacer, CoordinatesFormatter, Corner,
};

const METRICS_SAMPLES_START : isize = -(MAX_METRICS_SAMPLES as isize);
const MIN_RANGE : isize = 15;

pub struct Metrics {
    #[allow(dead_code)]
    runtime: Runtime,
}

impl Metrics {
    pub fn new(runtime: Runtime) -> Self {
        Self { runtime }
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
        let screen_rect_height = ui.ctx().screen_rect().height();

        let mut store_settings = false;
                            
        let mut graph_columns = core.settings.user_interface.metrics.graph_columns;
        let mut graph_height = core.settings.user_interface.metrics.graph_height;
        let mut graph_range_from = core.settings.user_interface.metrics.graph_range_from;
        let mut graph_range_to = core.settings.user_interface.metrics.graph_range_to;

        if graph_range_from < METRICS_SAMPLES_START {
            graph_range_from = METRICS_SAMPLES_START;
        }
        
        if graph_range_to > 0 {
            graph_range_to = 0;
        }

        if graph_range_to < METRICS_SAMPLES_START {
            graph_range_to = METRICS_SAMPLES_START;
        }

        if graph_range_to > 0 {
            graph_range_to = 0;
        }

        ui.horizontal(|ui|{
            ui.heading(i18n("Metrics"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                PopupPanel::new(PopupPanel::id(ui,"metrics_settings"),|ui|{ ui.add(Label::new("Settings ⏷").sense(Sense::click())) }, |ui, _| {
                    ui.add(
                        Slider::new(&mut graph_columns, 1..=8)
                            .text("Columns")
                            .orientation(SliderOrientation::Horizontal)
                            .step_by(1.)
                    );
                    ui.space();
                    ui.add(
                        Slider::new(&mut graph_height, 1..=1200)
                            .text("Height")
                            .logarithmic(true)
                            .orientation(SliderOrientation::Horizontal)
                            .suffix("px")
                    );

                    ui.space();
                    ui.separator();
                    ScrollArea::vertical()
                        .id_source("metrics_popup_selector")
                        .auto_shrink([false;2])
                        .show(ui, |ui| {

                            ui.horizontal(|ui| {
                                if ui.button(i18n("All")).clicked() {
                                    core.settings.user_interface.metrics.disabled.clear();
                                }

                                if ui.button(i18n("None")).clicked() {
                                    core.settings.user_interface.metrics.disabled = Metric::list().into_iter().collect::<AHashSet<_>>();
                                }
                                
                                if ui.button(i18n("Key Perf.")).clicked() {
                                    core.settings.user_interface.metrics.disabled = Metric::list().into_iter().filter(|metric|!metric.is_key_performance_metric()).collect::<AHashSet<_>>();
                                }

                            });

                            ui.separator();

                            for group in MetricGroup::list() {
                                CollapsingHeader::new(i18n(group.title()))
                                    .default_open(true)
                                    .show(ui, |ui| {
            
                                        for metric in group.metrics() {
                                            ui.space();
                                            let mut state = !core.settings.user_interface.metrics.disabled.contains(metric);
                                            if ui.checkbox(&mut state, i18n(metric.title().0)).changed() {
                                                if state {
                                                    core.settings.user_interface.metrics.disabled.remove(metric);
                                                } else {
                                                    core.settings.user_interface.metrics.disabled.insert(*metric);
                                                }
                                                // core.store_settings();
                                                store_settings = true;
                                            }
                                        }
                                    });
                            }
                        });

                })
                .with_min_width(240.)
                .with_max_height(screen_rect_height * 0.8)
                .with_caption(i18n("Settings"))
                .with_close_button(true)
                .build(ui);

                ui.separator();

                ui.add(
                    Slider::new(&mut graph_range_to, (METRICS_SAMPLES_START+MIN_RANGE)..=0)
                        .logarithmic(true)
                        .orientation(SliderOrientation::Horizontal)
                        .show_value(false)
                        // .custom_formatter(|v, _range| {
                        //     format_duration(-v as u64)
                        // })
                );
                ui.add(
                    Slider::new(&mut graph_range_from, METRICS_SAMPLES_START..=-MIN_RANGE)
                        .logarithmic(true)
                        .orientation(SliderOrientation::Horizontal)
                        .show_value(false)
                        // .custom_formatter(|v, _range| {
                        //     format_duration(-v as u64)
                        // })
                );
                ui.label(format!("{} ... {}", format_duration(-graph_range_from as u64), format_duration(-graph_range_to as u64)));
                if core.device().orientation() != Orientation::Portrait {
                    ui.label(i18n("Range:"));
                }

            });
        });
        
        if graph_range_from != core.settings.user_interface.metrics.graph_range_from && graph_range_from.abs_diff(graph_range_to) < 15{
            graph_range_to = graph_range_from + 15;
        }

        if graph_range_to != core.settings.user_interface.metrics.graph_range_to && graph_range_to.abs_diff(graph_range_from) < 15 {
            graph_range_from = graph_range_to - 15;
        }

        if graph_range_from.abs_diff(graph_range_to) < 15 {
            graph_range_to = graph_range_from + 15;
        }
    
        if graph_range_to.abs_diff(graph_range_from) < 15 {
            graph_range_from = graph_range_to - 15;
        }

        if graph_range_from > graph_range_to {
            graph_range_from = graph_range_to - 15;
        }

        if store_settings
        || graph_columns != core.settings.user_interface.metrics.graph_columns 
        || graph_height != core.settings.user_interface.metrics.graph_height 
        || graph_range_from != core.settings.user_interface.metrics.graph_range_from 
        || graph_range_to != core.settings.user_interface.metrics.graph_range_to 
        {
            core.settings.user_interface.metrics.graph_columns = graph_columns;
            core.settings.user_interface.metrics.graph_height = graph_height;
            core.settings.user_interface.metrics.graph_range_from = graph_range_from;
            core.settings.user_interface.metrics.graph_range_to = graph_range_to;
            
            core.store_settings();
        }

        ui.separator();

        if let Some(metrics) = core.metrics().as_ref() {

            egui::ScrollArea::vertical()
                .id_source("node_metrics")
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    let view_width = ui.available_width() - 32.;
                    let graph_height = core.settings.user_interface.metrics.graph_height as f32;

                    let (columns, graph_width) = if core.device().orientation() == Orientation::Portrait {
                        (1,view_width)
                    } else {
                        (core.settings.user_interface.metrics.graph_columns, view_width / core.settings.user_interface.metrics.graph_columns as f32)
                    };


                        let mut metric_iter = Metric::list().into_iter().filter(|metric| !core.settings.user_interface.metrics.disabled.contains(metric));
                        let mut draw = true;
                        while draw {
                            ui.horizontal(|ui| {
                                for _ in 0..columns {
                                    if let Some(metric) = metric_iter.next() {
                                        let range_from = core.settings.user_interface.metrics.graph_range_from;
                                        let range_to = core.settings.user_interface.metrics.graph_range_to;
                                        self.render_metric(ui,metric,metrics,range_from..range_to,graph_width,graph_height);
                                    } else {
                                        draw = false;
                                    }
                                }
                            });
                        }
                    // }
                });
        } else {
            ui.vertical_centered(|ui| {

                ui.style_mut().text_styles = core.mobile_style.text_styles.clone();

                ui.label(i18n("Metrics are not currently available"));
                ui.add_space(32.);
                
                if core.settings.node.node_kind != KaspadNodeKind::Disable {
                    ui.add_space(64.);
                    ui.add(egui::Spinner::new().size(92.));
                } else {
                    ui.label(i18n("Please connect to Kaspa p2p node"));
                }

            });
        }

    }
}

impl Metrics {

    #[allow(clippy::too_many_arguments)]
    fn render_metric(
        &mut self, 
        ui : &mut Ui, 
        metric : Metric, 
        metrics : &MetricsSnapshot, 
        range : std::ops::Range<isize>,
        graph_width : f32, 
        graph_height : f32
    ) {

        let group = MetricGroup::from(metric);
        let graph_color = group.to_color();

        StripBuilder::new(ui)
            .size(Size::exact(graph_width))
            .horizontal(|mut strip| {

                strip.cell(|ui| {
                    ui.vertical(|ui| {
                        ui.add_space(8.);
                        ui.horizontal(|ui|{
                            ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                                ui.colored_label(theme_color().metrics_text_color, format!("{}: {}", i18n(metric.title().0), metric.format(metrics.get(&metric), true, false)));
                            });
                        });

                        // ---
                        let graph_data = {
                            let metrics_data = self.runtime.metrics_service().metrics_data();
                            let data = metrics_data.get(&metric).unwrap();
                            let mut start = range.start.clamp(METRICS_SAMPLES_START, 0).unsigned_abs();
                            let mut end = range.end.clamp(METRICS_SAMPLES_START, 0).unsigned_abs();
                            if start > data.len() {
                                start = data.len();
                            }
                            if end > data.len() {
                                end = data.len();
                            }
                            data[data.len()-start..data.len()-end].to_vec()
                        };

                        let mut plot = Plot::new(metric.as_str())
                        // .link_axis(id, true, false)
                        // .allow_boxed_zoom(true)
                        // .allow_double_click_reset(true)
                            .legend(Legend::default())
                            .width(graph_width)
                            .height(graph_height)
                            .auto_bounds_x()
                            .auto_bounds_y()
                            .set_margin_fraction(vec2(0.0,0.0) )
                            .y_axis_width(4)
                            .show_axes(true)
                            .show_grid(true)
                            // .allow_drag([true, false])
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
                            .coordinates_formatter(Corner::LeftTop, CoordinatesFormatter::new(move |point,_| {
                                let PlotPoint { x: _, y } = point;
                                metric.format(*y, true, true)
                            }))
                            ;

                        if [Metric::NodeCpuUsage].contains(&metric) {
                            plot = plot.include_y(100.);
                        }
                
                        if [
                            Metric::NodeResidentSetSizeBytes, 
                            Metric::NodeVirtualMemorySizeBytes,
                            Metric::NodeFileHandlesCount,
                            Metric::NodeDiskIoReadPerSec,
                            Metric::NodeDiskIoWritePerSec,
                            Metric::NetworkTransactionsPerSecond,
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
