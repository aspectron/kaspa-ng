use crate::imports::*;
use egui_plot::{
    LineStyle,
    Plot,
    PlotPoints, Polygon, uniform_grid_spacer, Line, PlotBounds,
    // Legend,
    // Corner
};

#[derive(Clone)]
pub struct Preset {
    name : &'static str,
    daa_range : f64,
    daa_offset : f64,
    spread : f64,
    noise : f64,
    block_scale : f64,
}

const PRESETS: &[Preset] = &[
    Preset {
        name : "Large (1 BPS)",
        daa_range : 36.0,
        daa_offset : 10.0,
        spread : 10.0,
        noise : 0.0,
        block_scale : 1.0,
    },
    Preset {
        name : "Medium Wide",
        daa_range : 100.0,
        daa_offset : 16.0,
        spread : 36.0,
        noise : 0.0,
        block_scale : 1.0,
    },
    Preset {
        name : "Medium Narrow",
        daa_range : 80.0,
        daa_offset : 16.0,
        spread : 22.0,
        noise : 0.0,
        block_scale : 1.2,
    },
    Preset {
        name : "Small (10 BPS)",
        daa_range : 180.0,
        daa_offset : 32.0,
        spread : 36.0,
        noise : 1.0,
        block_scale : 1.4,
    },
];

impl From<Network> for Preset {
    fn from(network: Network) -> Self {
        match network {
            Network::Mainnet => PRESETS[0].clone(),
            Network::Testnet10 => PRESETS[3].clone(),
        }
    }
}

pub struct BlockDag {
    #[allow(dead_code)]
    runtime: Runtime,
    daa_cursor: f64,
    last_daa_score : u64,
    running : bool,
    plot_bounds : PlotBounds,
    bezier : bool,
    parent_levels : usize,
    parent_threshold : usize,
    daa_offset : f64,
    daa_range : f64,
    block_scale : f64,
    last_repaint : Instant,
    settings: BlockDagGraphSettings,
    background : Arc<AtomicBool>,
    network : Network,
}

impl BlockDag {
    pub fn new(runtime: Runtime) -> Self {

        let preset = Preset::from(Network::Mainnet);
        let settings = BlockDagGraphSettings::new(preset.spread);
        runtime.block_dag_monitor_service().update_settings(settings.clone());

        Self {
            runtime,
            daa_cursor : 0.0,
            last_daa_score : 0,
            running : false,
            plot_bounds : PlotBounds::NOTHING,
            bezier : true,
            daa_offset : preset.daa_offset,
            daa_range : preset.daa_range,
            block_scale : preset.block_scale,
            last_repaint : Instant::now(),
            parent_levels : 1,
            parent_threshold : 200,
            settings,
            background : Arc::new(AtomicBool::new(false)),
            network : Network::Mainnet,
        }
    }

    pub fn background(&self) -> bool {
        self.background.load(Ordering::SeqCst)
    }

    pub fn background_state(&self) -> Arc<AtomicBool> {
        self.background.clone()
    }

    pub fn load_preset(&mut self, preset : &Preset) {
        self.daa_range = preset.daa_range;
        self.daa_offset = preset.daa_offset;
        self.block_scale = preset.block_scale;
        self.settings.y_dist = preset.spread;
        self.settings.noise = preset.noise;
    }

    fn reset_state(&mut self) {
        self.running = false;
        self.daa_cursor = 0.0; 
        self.last_daa_score = 0; 
    }

}

impl ModuleT for BlockDag {

    fn style(&self) -> ModuleStyle {
        ModuleStyle::Default
    }

    fn status_bar(&self, core: &mut Core, ui : &mut Ui) {
        ui.separator();
        if !core.state().is_connected() {
            ui.label(RichText::new(i18n("You must be connected to a node...")).color(theme_color().error_color));
        } else if !core.state().is_synced() {
            ui.label(RichText::new(i18n("Please wait for the node to sync...")).color(theme_color().warning_color));
        } else {
            ui.label(i18n("Double click on the graph to re-center..."));
        }
    }

    fn render(
        &mut self,
        core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        let theme_color = theme_color();

        let y_dist = self.settings.y_dist;
        let noise = self.settings.noise;
        let vspc_center = self.settings.center_vspc;

        if core.settings.node.network != self.network {
            self.network = core.settings.node.network;
            self.load_preset(&self.network.into());
            runtime().block_dag_monitor_service().update_settings(self.settings.clone());
        }

        ui.horizontal(|ui| {
            ui.heading(i18n("Block DAG"));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                PopupPanel::new(PopupPanel::id(ui,"block_dag_settings"),|ui|{ ui.add(Label::new(format!("{} ⏷", i18n("Settings"))).sense(Sense::click())) }, |ui, _| {

                    CollapsingHeader::new(i18n("Dimensions"))
                        .open(Some(true))
                        // .default_open(true)
                        .show(ui, |ui| {
                            ui.space();

                            ui.space();
                            ui.add(
                                Slider::new(&mut self.daa_range, 1.0..=self.settings.graph_length_daa as f64)
                                    .clamping(SliderClamping::Always)
                                    .logarithmic(true)
                                    .text(i18n("DAA Range"))
                            );
                            ui.space();
                            ui.add(
                                Slider::new(&mut self.daa_offset, 1.0..=50.0)
                                    .clamping(SliderClamping::Always)
                                    .text(i18n("DAA Offset"))
                                    // .step_by(1.0)
                            );
                            ui.space();
                            ui.add(
                                Slider::new(&mut self.settings.y_dist, 1.0..=100.0)
                                    .clamping(SliderClamping::Always)
                                    .text(i18n("Spread"))
                            );
                            ui.space();
                            ui.add(
                                Slider::new(&mut self.settings.noise, 0.0..=10.0)
                                    .clamping(SliderClamping::Always)
                                    .text(i18n("Noise"))
                            );
                            ui.space();
                            ui.add(
                                Slider::new(&mut self.block_scale, 0.1..=2.5)
                                    .clamping(SliderClamping::Always)
                                    .logarithmic(true)
                                    .text(i18n("Block Scale"))
                            );
                            ui.space();
                        });

                    CollapsingHeader::new(i18n("Parents"))
                        .open(Some(true))
                        // .default_open(true)
                        .show(ui, |ui| {

                            ui.space();
                            
                            ui.add(
                                Slider::new(&mut self.parent_levels, 1..=50)
                                    .clamping(SliderClamping::Always)
                                    .text(i18n("Levels"))
                                    .step_by(1.0)
                            );
                            ui.space();
                            ui.add(
                                Slider::new(&mut self.parent_threshold, 200..=1000)
                                    .clamping(SliderClamping::Always)
                                    .logarithmic(true)
                                    .text(i18n("Threshold"))
                            );
                            ui.space();
                        });

                    ui.separator();
                    ui.space();
                    ui.horizontal_wrapped(|ui| {
                        ui.checkbox(&mut self.settings.center_vspc, i18n("Center VSPC"));
                        ui.space();
                        ui.checkbox(&mut self.settings.show_vspc, i18n("Show VSPC"));
                        ui.space();
                        // ui.checkbox(&mut self.settings.reset_vspc, i18n("Reset VSPC"));
                        // ui.space();
                        ui.checkbox(&mut self.settings.show_grid, i18n("Show Grid"));
                        ui.space();
                        ui.checkbox(&mut self.settings.show_daa, i18n("Show DAA"));
                        ui.space();
                        ui.checkbox(&mut self.bezier, i18n("Bezier Curves"));
                        ui.space();

                        if core.settings.node.node_kind.is_local() {
                            let background_flag = self.background.load(Ordering::SeqCst);
                            let mut background_state = background_flag;
                            ui.checkbox(&mut background_state, i18n("Track in the background"));
                            if background_state != background_flag {
                                self.background.store(background_state, Ordering::SeqCst);
                            }
                        }
                    });
                })
                .with_min_width(240.)
                .with_caption(i18n("Settings"))
                .with_close_button(true)
                .build(ui);

                let response = ui
                        .add(Label::new(RichText::new(format!("{} ⏷", i18n("Presets")))).sense(Sense::click()));
                PopupPanel::new(
                    PopupPanel::id(ui,"network_selector_popup"),
                    |_ui| response,
                    |ui, close| {
                        set_menu_style(ui.style_mut());
                        for preset in PRESETS {
                            if ui.button(i18n(preset.name)).clicked() {
                                self.load_preset(preset);
                                *close = true;
                            }
                        }
                })
                .with_min_width(100.0)
                .build(ui);

            });
        });
        ui.separator();

        if y_dist != self.settings.y_dist || noise != self.settings.noise || vspc_center != self.settings.center_vspc {
            runtime().block_dag_monitor_service().update_settings(self.settings.clone());
        }

        let mut reset_plot = false;
        let current_daa_score = core.state().current_daa_score().unwrap_or_default();
        if self.last_daa_score != current_daa_score {

            if !self.running {
                self.running = true;
                reset_plot = true;
                self.daa_cursor = current_daa_score as f64 - 1.0;
            }

            self.last_daa_score = current_daa_score;
        }

        // let mut reset_plot = false;
        // let current_daa_score = core.state().current_daa_score().unwrap_or_default();
        // if self.last_daa_score != current_daa_score || current_daa_score==0 {
        //    if !self.running || current_daa_score==0{
        //         if current_daa_score > 0{
        //             self.running = true;
        //         }else{
        //             self.running = false;
        //         }
        //         reset_plot = true;
        //         self.daa_cursor = current_daa_score as f64 - 1.0;
        //     }

        //     self.last_daa_score = current_daa_score;
        // }

        let delta = 0.025;
        let daa_diff = current_daa_score as f64 - self.daa_cursor;
        let step = daa_diff * delta;
        let step = (1.0 + step).powf(2.0) - 1.0;
        self.daa_cursor += step;
        
        let graph_width = ui.available_width();
        let graph_height = ui.available_height();
        let default_daa_max = self.daa_cursor + self.daa_offset;
        let default_daa_min = default_daa_max - self.daa_range;
        let default_daa_range = default_daa_max - default_daa_min;
        let pixels_per_daa = graph_width as f64 / default_daa_range;

        let mut plot = Plot::new("block_dag")
            .width(graph_width)
            .height(graph_height)
            .include_x(default_daa_max)
            .include_x(default_daa_min)
            .include_y(15.)
            .include_y(-15.)
            .data_aspect(0.2)
            .y_axis_min_width(0.0)
            .show_axes([self.settings.show_daa, false])
            .show_grid(self.settings.show_grid)
            .allow_drag([true, true])
            .allow_scroll(true)
            .allow_double_click_reset(true)
            .x_axis_formatter(move |x, _range| {
                format!("{} DAA", x.value.trunc().separated_string())
            })
            .x_grid_spacer(
                uniform_grid_spacer(move |input| {
                    let (start,stop) = input.bounds;
                    let d = (stop - start) / 5.;
                    let mut v = 10.;
                    while v < d {
                        v *= 2.;
                    }
                    [v,v*10.,v*100.]
                })
            )
            .label_formatter(move |name, point| {
                let PlotPoint { x, y: _ } = point;
                format!("{name}\n{} DAA", x.trunc().separated_string())
            })                        
            ;

        // kick it into gear when starting up
        if reset_plot {
            plot = plot.auto_bounds([true, true]);
            plot = plot.reset();
        } else {
            plot = plot.auto_bounds([false, false]);
        }

        let mut graph_settled = true;
        let mut lines_parent = Vec::new();
        let mut lines_vspc = Vec::new();

        let daa_range = self.plot_bounds.max()[0] - self.plot_bounds.min()[0];
        let daa_margin = daa_range.min(128.0).max(32.0);
        let daa_min = (self.plot_bounds.min()[0] - daa_margin).max(0.0) as u64;
        let daa_max = (self.plot_bounds.max()[0] + daa_margin).max(0.0) as u64;
        
        let blocks = if let Ok(mut daa_buckets) = self.runtime.block_dag_monitor_service().chain.lock() {
            daa_buckets.iter_mut().filter_map(|(daa_score,bucket)| {
                (*daa_score > daa_min && *daa_score < daa_max).then_some(bucket)
            }).flat_map(DaaBucket::render).collect::<Vec<_>>()
        } else {
            return;
        };

        // let separators = if let Ok(separators) = self.runtime.block_dag_monitor_service().separators.lock() {
        //     separators.iter().filter_map(|daa_score| {
        //         (*daa_score > daa_min || *daa_score < daa_max).then_some(*daa_score)
        //     }).collect::<Vec<_>>()
        // } else {
        //     return;
        // };

        let parent_levels = self.parent_levels.max(1);
        let block_map : AHashMap<KaspaHash,(PlotPoint,bool)> = blocks.clone().into_iter().map(|(block, plot_point,vspc, _)|(block.header.hash,(plot_point,vspc))).collect();
        let new_blocks = self.runtime.block_dag_monitor_service().new_blocks().clone();
        let polygons = blocks.iter().map(|(block, point, current_vspc, block_settled)| {
            if !block_settled {
                graph_settled = false;
            }

            let PlotPoint { x, y } = *point;

            for (level,parent_level) in block.header.parents_by_level.iter().enumerate() {
                if level >= parent_levels {
                    break;
                }
                // for parent_hash in block.header.parents_by_level.iter().flatten() {
                for parent_hash in parent_level.iter() {
                    if let Some(parent_point) = block_map.get(parent_hash) {
                        let (PlotPoint { x: parent_x, y: parent_y }, parent_vspc) = *parent_point;
                        let x_len = (x - parent_x).abs();
                        
                        if x_len > self.parent_threshold as f64 {
                            continue;
                        }

                        let points = if self.bezier {
                            // x dist is sufficient... (let's save some cycles)
                            let line_steps = (x_len * pixels_per_daa * 0.3) as usize;
                            bezier(x,y,parent_x,parent_y,line_steps,0.6) 
                        } else {
                            [
                                [x,y],
                                [parent_x, parent_y],
                            ].into_iter().map(|pt|pt.into()).collect::<Vec<_>>()
                        };
                        if self.settings.show_vspc && level == 0 && *current_vspc && parent_vspc {
                            lines_vspc.push(Line::new("", PlotPoints::Owned(points)).color(theme_color.block_dag_vspc_connect_color).style(LineStyle::Solid).width(3.0));
                        } else {
                            lines_parent.push(Line::new("", PlotPoints::Owned(points)).color(theme_color.block_dag_parent_connect_color).style(LineStyle::Solid));
                        }
                    }
                }
            }

            let d = 1.5 * self.block_scale;
            let points: PlotPoints<'_> = [
                [x+d*0.2, y+d],
                [x-d*0.2, y+d],
                [x-d*0.2, y-d],
                [x+d*0.2, y-d],
            ].to_vec().into();
        
            let fill_color = if new_blocks.contains(&block.header.hash) {
                theme_color.block_dag_new_block_fill_color
            } else {
                theme_color.block_dag_block_fill_color
            };

            Polygon::new("polygon1", points)
                .name(block.header.hash.to_string())
                .fill_color(fill_color)
                .stroke(Stroke::new(1.0, theme_color.block_dag_block_stroke_color))
                .style(LineStyle::Solid)

            
        }).collect::<Vec<_>>();

        // let lines_separators = separators.iter().map(|daa_score| {
        //     let x = *daa_score as f64;
        //     let points: PlotPoints = [
        //         [x, 0.0 - y_dist],
        //         [x, 0.0 + y_dist],
        //     ].to_vec().into();
        //     Line::new("", points).color(theme_color.block_dag_separator_color).style(LineStyle::Dotted { spacing: 0.75 })
        // }).collect::<Vec<_>>();

        let plot_response = plot.show(ui, |plot_ui| {
            // lines_separators.into_iter().for_each(|line| {
            //     plot_ui.line(line);
            // });
            lines_parent.into_iter().for_each(|line| {
                plot_ui.line(line);
            });
            lines_vspc.into_iter().for_each(|line| {
                plot_ui.line(line);
            });
            polygons.into_iter().for_each(|polygon| {
                plot_ui.polygon(polygon);
            });
        });

        if daa_diff > 0.001 || !graph_settled {
            runtime().request_repaint();
        } 

        self.plot_bounds = *plot_response.transform.bounds();
        self.last_repaint = Instant::now();

    }

    fn activate(&mut self, core: &mut Core) {
        let block_dag_monitor_service = crate::runtime::runtime().block_dag_monitor_service().clone();
        block_dag_monitor_service.enable(core.state().current_daa_score().map(|score|score - 2));
        block_dag_monitor_service.activate(true);
    }

    fn deactivate(&mut self, core: &mut Core) {
        if !self.background() {
            let block_dag_monitor_service = crate::runtime::runtime().block_dag_monitor_service().clone();
            self.running = false;
            block_dag_monitor_service.disable(core.state().current_daa_score());
            block_dag_monitor_service.activate(false);
        }
    }

    fn disconnect(&mut self, _core: &mut Core) {
        self.reset_state();
    }

    fn show(&mut self, _core: &mut Core) {
        self.reset_state();
    }

}

