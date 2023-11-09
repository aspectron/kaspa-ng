use chrono::DateTime;
use egui::*;
use egui_plot::{
    // Arrows, AxisBools, AxisHints, Bar, BarChart, BoxElem, BoxPlot, BoxSpread, CoordinatesFormatter,
    // Corner, GridInput, GridMark, HLine,
    // MarkerShape,
    // PlotImage,
    // PlotResponse, Points, Polygon, Text, VLine,
    Legend,
    Line,
    LineStyle,
    Plot,
    PlotPoint,
    PlotPoints,
};
// use workflow_core::time::unixtime_as_millis_f64;

#[must_use = "You should put this widget in an ui with `ui.add(widget);`"]
pub struct CompositeGraph<'a> {
    id: String,
    show_axes: bool,
    show_grid: bool,
    graph_data: &'a Vec<PlotPoint>,
}

impl<'a> CompositeGraph<'a> {
    pub fn new(id: impl Into<String>, graph_data: &'a Vec<PlotPoint>) -> Self {
        Self {
            id: id.into(),
            show_axes: true,
            show_grid: true,
            graph_data,
        }
    }

    pub fn show_axes(mut self, show_axes: bool) -> Self {
        self.show_axes = show_axes;
        self
    }
    pub fn show_grid(mut self, show_grid: bool) -> Self {
        self.show_grid = show_grid;
        self
    }

    fn render(&mut self, ui: &mut Ui) -> Response {
        //self.options_ui(ui);

        // if self.animate {
        //     ui.ctx().request_repaint();
        //     self.time += ui.input(|i| i.unstable_dt).at_most(1.0 / 30.0) as f64;
        // };
        let first_point = self.graph_data[0];
        let last_point = self.graph_data[self.graph_data.len() - 1];
        let plot = Plot::new(&self.id)
            .legend(Legend::default())
            .height(96.)
            .auto_bounds_x()
            .auto_bounds_y()
            .y_axis_width(4)
            .show_axes(self.show_axes)
            .show_grid(self.show_grid)
            .allow_drag([true, false])
            .allow_scroll(false)
            .x_axis_label("Time")
            // .x_grid_spacer(move |input|{
            //     let mut list = vec![];
            //     //let value = first_point.x;//input.bounds.0.ceil();
            //     list.push(GridMark { value: first_point.x, step_size: 100.0 });
            //     list.push(GridMark { value: last_point.x, step_size: 100.0 });
            //     list
            //     // vec![
            //     //     // 100s
            //     //     GridMark { value: 100.0, step_size: 100.0 },
            //     //     GridMark { value: 200.0, step_size: 100.0 },
            //     //     // 25s
            //     //     GridMark { value: 125.0, step_size: 25.0 },
            //     //     GridMark { value: 150.0, step_size: 25.0 },
            //     //     GridMark { value: 175.0, step_size: 25.0 },
            //     //     GridMark { value: 225.0, step_size: 25.0 },
            //     // ]
            // })
            .x_axis_formatter(move |x, size, range| {
                workflow_log::log_info!("x:{x}, size:{size}, range:{range:?}");
                if x <= first_point.x || x >= last_point.x {
                    DateTime::<chrono::Utc>::from_timestamp((x / 1000.0) as i64, 0)
                        .expect("could not parse timestamp")
                        .with_timezone(&chrono::Local)
                        .format("%H:%M:%S")
                        .to_string()
                } else {
                    "".to_string()
                }
                //"1".to_string()
            });
        // if self.square {
        //     plot = plot.view_aspect(1.0);
        // }
        // if self.proportional {
        //     plot = plot.data_aspect(1.0);
        // }
        // if self.coordinates {
        //     plot = plot.coordinates_formatter(Corner::LeftBottom, CoordinatesFormatter::default());
        // }

        plot.show(ui, |plot_ui| {
            //plot_ui.line(self.circle());
            plot_ui.line(self.line());
            //plot_ui.line(self.thingy());
        })
        .response
    }

    fn line(&self) -> Line {
        // Line::new(PlotPoints::from_explicit_callback(
        //     move |x| 0.5 * x,
        //     0.0..100.0,
        //     20,
        // ))

        Line::new(PlotPoints::Owned(self.graph_data.clone()))
            .color(crate::egui::theme::theme().graph_color)
            // .color(Color32::from_rgb(200, 100, 100))
            .style(LineStyle::Solid)
            .fill(0.0)
        //.name("wave")
    }
}

impl<'a> Widget for CompositeGraph<'a> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        self.render(ui)
    }
}
