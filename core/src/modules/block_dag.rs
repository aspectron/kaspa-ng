use crate::imports::*;
use egui_plot::{
    Legend,
    LineStyle,
    Plot,
    PlotPoints, Polygon, uniform_grid_spacer, Line,
};

pub struct BlockDag {
    #[allow(dead_code)]
    runtime: Runtime,
    daa_cursor: f64,
    last_daa_score : u64,
    running : bool,
    
}

impl BlockDag {
    pub fn new(runtime: Runtime) -> Self {
        Self { runtime, daa_cursor : 0.0, last_daa_score : 0, running : false }
    }
}

impl ModuleT for BlockDag {

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
        let _theme = theme();

        ui.heading("BlockDAG");
        ui.separator();

        let current_daa_score = core.state().current_daa_score().unwrap_or_default();
        if self.last_daa_score != current_daa_score {

            if !self.running {
                self.running = true;
                self.daa_cursor = current_daa_score as f64;
            }

            self.last_daa_score = current_daa_score;
        }

        let delta = 0.005;
        let diff = current_daa_score as f64 - self.daa_cursor;
        let step = diff * delta;
        self.daa_cursor += step;
        if diff > 0.01 {
            crate::runtime::try_runtime().unwrap().request_repaint();
        }
        
        let graph_width = ui.available_width();
        let graph_height = ui.available_height();

        let plot = Plot::new("block_dag")
            .legend(Legend::default())
            .width(graph_width)
            .height(graph_height)
            .include_x(self.daa_cursor + 8.)
            .include_x(self.daa_cursor - 20.)
            // .include_x(self.daa_cursor + 30.)
            // .include_x(self.daa_cursor - 150.)
            .include_y(15.)
            .include_y(-15.)
            // .include_y(100.)
            // .include_y(-100.)
            // .auto_bounds_x()
            // .auto_bounds_y()
            .data_aspect(0.3)
            .y_axis_width(4)
            .show_axes(true)
            .show_grid(true)
            .allow_drag([true, false])
            .allow_scroll(true)
            .allow_double_click_reset(true)
            // .y_axis_formatter(move |y,_size,_range|{
            //     metric.format(y, true, true)
            // })
            .x_axis_formatter(move |x, _size, _range| {
                format!("{} DAA", x.trunc().separated_string())
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
            .label_formatter(move |_name, point| {
                let PlotPoint { x, y: _ } = point;
                format!("{} DAA", x.trunc().separated_string())
            })                                                    
            ;

        let fill_color = Color32::LIGHT_BLUE;
        let stroke_color = Color32::BLUE;

        let mut lines = Vec::new();

        let blocks = if let Ok(block_map) = self.runtime.block_dag_monitor_service().blocks_by_hash.read() {

            block_map.iter().map(|(hash,block)| {
                let x = block.header.daa_score as f64;
                let y = hash_to_y(hash);

                for parent_hash in block.header.direct_parents() {
                    if let Some(parent) = block_map.get(parent_hash) {
                        let parent_x = parent.header.daa_score as f64;
                        let parent_y = hash_to_y(parent_hash);

                        let points: Vec<PlotPoint> = [
                            [x,y],
                            [parent_x, parent_y],
                        ].into_iter().map(|pt|pt.into()).collect::<Vec<_>>();
        
                        lines.push(Line::new(PlotPoints::Owned(points)).color(fill_color).style(LineStyle::Solid));
                    }
                }

                let d = 1.5;
                let points: PlotPoints = [
                    [x+d*0.3, y+d],
                    [x-d*0.3, y+d],
                    [x-d*0.3, y-d],
                    [x+d*0.3, y-d],
                ].to_vec().into();
            
                Polygon::new(points)
                    .fill_color(fill_color)
                    .stroke(Stroke::new(1.0, stroke_color))
                    .style(LineStyle::Solid)

            }).collect::<Vec<_>>()
        } else {
            return;
        };

        plot.show(ui, |plot_ui| {
            blocks.into_iter().for_each(|polygon| {
                plot_ui.polygon(polygon);
            });
            lines.into_iter().for_each(|line| {
                plot_ui.line(line);
            });
        });
    }
}

pub fn hash_to_y(hash: &kaspa_consensus_core::Hash) -> f64 {
    let bytes = hash.as_bytes().iter().take(1).cloned().collect::<Vec<_>>();
    (i8::from_le_bytes(bytes.as_slice().try_into().unwrap())-127) as f64 * (10.0 / 127.0)
}