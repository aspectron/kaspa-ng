use std::borrow::Cow;

use egui::load::Bytes;
use kaspa_metrics::{Metric,MetricGroup};
use egui_plot::{
    Legend,
    Line,
    LineStyle,
    Plot,
    PlotPoints,
};

use crate::imports::*;

pub struct Overview {
    #[allow(dead_code)]
    runtime: Runtime,
}

impl Overview {
    pub fn new(runtime: Runtime) -> Self {
        Self { runtime }
    }
}

impl ModuleT for Overview {

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

        let width = ui.available_width();

        SidePanel::left("overview_left").exact_width(width/2.).resizable(false).show_separator_line(true).show_inside(ui, |ui| {
            // ui.label("Kaspa NG");
            egui::ScrollArea::vertical()
                .id_source("overview_metrics")
                .auto_shrink([false; 2])
                .show(ui, |ui| {

                CollapsingHeader::new(i18n("Kaspa p2p Node"))
                    .default_open(true)
                    .show(ui, |ui| {
                        // ui.label(format!("Kaspa NG v{}-{} + Rusty Kaspa v{}", env!("CARGO_PKG_VERSION"),crate::app::GIT_DESCRIBE, kaspa_wallet_core::version()));
                        self.render_graphs(core,ui);
                    });

                ui.add_space(48.);
            });
        });

        SidePanel::right("overview_right")
            .exact_width(width/2.)
            .resizable(false)
            .show_separator_line(false)
            .show_inside(ui, |ui| {
                // ui.label("Wallet Stuff");
                // let module = core.modules().get(&TypeId::of::<modules::AccountManager>()).unwrap().clone();
                // module.render_default(core,ctx,frame,ui);

                // ui.image(Image::try_from(crate::app::KASPA_NG_ICON_256X256));
                
                // ui.horizontal(|ui| {
                    // ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        // ui.add(Image::new(ImageSource::Bytes { uri : Cow::Borrowed("bytes://logo.png"), bytes : Bytes::Static(crate::app::KASPA_NG_ICON_256X256)}).maintain_aspect_ratio(true).max_size(vec2(128.,128.)));
                    // });
                // });
                let image_size = vec2(64.,64.);
                let cursor = ui.cursor().min;
                let width = ui.available_width();
                let left = width - image_size.x*2. + cursor.x;
                let top = cursor.y;// - 32.;//image_size.y/2.;
                // rect.se
                let rect = Rect::from_min_size(Pos2::new(left, top), image_size);

                // ui.put(
                //     rect,
                //     Image::new(ImageSource::Bytes { uri : Cow::Borrowed("bytes://logo.png"), bytes : Bytes::Static(crate::app::KASPA_NG_ICON_256X256)})
                //         .maintain_aspect_ratio(true)
                //         .max_size(image_size)
                //         .fit_to_exact_size(image_size)
                //         .shrink_to_fit()
                //         .texture_options(TextureOptions::LINEAR)
                // );

                CollapsingHeader::new(i18n("Kaspa NG"))
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.label(format!("Kaspa NG v{}-{} + Rusty Kaspa v{}", env!("CARGO_PKG_VERSION"),crate::app::GIT_DESCRIBE, kaspa_wallet_core::version()));
                        ui.label(format!("Build: {}", crate::app::BUILD_TIMESTAMP));
                        ui.label(format!("rustc {}-{} {} llvm {} {}", 
                            crate::app::RUSTC_SEMVER,
                            crate::app::RUSTC_COMMIT_HASH.chars().take(8).collect::<String>(),
                            crate::app::RUSTC_LLVM_VERSION,
                            crate::app::RUSTC_CHANNEL,
                            crate::app::RUSTC_HOST_TRIPLE
                        ));
                        ui.label("Codename: \"This is the way\"");
                    });

                if let Some(system) = runtime().system() {
                    system.render(ui);
                }

                CollapsingHeader::new(i18n("Market"))
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.label("TODO");
                    });

                CollapsingHeader::new(i18n("Resources"))
                    .default_open(true)
                    .show(ui, |ui| {
                        // use egui_phosphor::light::{DISCORD_LOGO,GITHUB_LOGO};
                        ui.hyperlink_to(
                            format!("• {}",i18n("Kaspa NextGen on GitHub")),
                            "https://github.com/aspectron/kaspa-ng",
                        );
                        ui.hyperlink_to(
                            format!("• {}",i18n("Rusty Kaspa on GitHub")),
                            "https://github.com/kaspanet/rusty-kaspa",
                        );
                        ui.hyperlink_to(
                            format!("• {}",i18n("WASM SDK for JavaScript and TypeScript")),
                            "https://github.com/kaspanet/rusty-kaspa/wasm",
                        );
                        ui.hyperlink_to(
                            format!("• {}",i18n("Rust Wallet SDK")),
                            "https://docs.rs/kaspa-wallet-core/0.0.4/kaspa_wallet_core/",
                        );
                        ui.hyperlink_to(
                            format!("• {}",i18n("NPM Modules for NodeJS")),
                            "https://www.npmjs.com/package/kaspa",
                        );
                        ui.hyperlink_to(
                            format!("• {}",i18n("Kaspa Discord")),
                            "https://discord.com/invite/kS3SK5F36R",
                        );
                    });

                CollapsingHeader::new(i18n("Music"))
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.label("TODO");
                    });

                CollapsingHeader::new(i18n("License Information"))
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.label("TODO");
                    });

                ui.put(
                    rect,
                    Image::new(ImageSource::Bytes { uri : Cow::Borrowed("bytes://logo.png"), bytes : Bytes::Static(crate::app::KASPA_NG_ICON_256X256)})
                        .maintain_aspect_ratio(true)
                        .max_size(image_size)
                        .fit_to_exact_size(image_size)
                        .shrink_to_fit()
                        .texture_options(TextureOptions::LINEAR)
                );

    


            });

    }
}

impl Overview {
    fn render_graphs(&mut self, core: &mut Core, ui : &mut Ui) {

        // let mut metric_iter = [Metric::CpuUsage,                 
        //     Metric::DiskIoReadPerSec,
        //     Metric::DiskIoWritePerSec,
        //     Metric::Tps,
        //     Metric::ChainBlockCounts,
        // ].into_iter();

        let mut metric_iter = METRICS.iter(); //Metric::list().into_iter();

        if let Some(snapshot) = core.metrics.as_ref() {
            let theme = theme();
            let view_width = ui.available_width();
            if view_width < 200. {
                return;
            }
            let graph_columns = ((view_width-48.) / 128.) as usize;

            let mut draw = true;
            while draw {
                ui.horizontal(|ui| {
                    for _ in 0..graph_columns {
                        if let Some(metric) = metric_iter.next() {
                            // let duration = 60 * 10; // 15 min (testing)
                            // let duration = core.settings.ux.metrics.graph_duration;
                            let value = snapshot.get(metric);
                            self.render_graph(ui,  *metric, value, theme);
                        } else {
                            draw = false;
                        }
                    }
                });
            }
        }

    }

    fn render_graph(&mut self, ui : &mut Ui, metric : Metric, value : f64, theme : &Theme) {

        let group = MetricGroup::from(metric);
        let graph_color = match group {
            MetricGroup::System => theme.performance_graph_color,
            MetricGroup::Storage => theme.storage_graph_color,
            MetricGroup::Network => theme.network_graph_color,
            MetricGroup::BlockDAG => theme.blockdag_graph_color,
        };    

        let graph_data = {
            let metrics_data = self.runtime.metrics_service().metrics_data();
            let data = metrics_data.get(&metric).unwrap();
            let mut duration = 2 * 60;
            let uptime = self.runtime.uptime().as_secs() as usize;
            if uptime < duration {
                duration = uptime;
            }
            let samples = if data.len() < duration { data.len() } else { duration };
            data[data.len()-samples..].to_vec()
        };

        
        ui.vertical(|ui|{
            Frame::none()
                // .fill(theme.performance_graph_color)
                .stroke(Stroke::new(1.0, theme.graph_frame_color))
                .inner_margin(4.)
                .outer_margin(8.)
                .rounding(8.)
                .show(ui, |ui| {

                    let mut plot = Plot::new(metric.as_str())
                        .legend(Legend::default())
                        .width(128.)
                        .height(32.)
                        .auto_bounds_x()
                        .auto_bounds_y()
                        .set_margin_fraction(vec2(0.0,0.0) )
                        .show_axes(false)
                        .show_grid(false)
                        .allow_drag([false, false])
                        .allow_scroll(false)
                        .show_background(false)
                        .show_x(false)
                        .show_y(false)
                        ;

                    if [Metric::CpuUsage].contains(&metric) {
                        plot = plot.include_y(100.);
                    }

                    // if [
                    //     Metric::ResidentSetSizeBytes, 
                    //     Metric::VirtualMemorySizeBytes,
                    //     Metric::FdNum,
                    //     // Metric::DiskIoReadBytes,
                    //     // Metric::DiskIoWriteBytes,
                    //     Metric::DiskIoReadPerSec,
                    //     Metric::DiskIoWritePerSec,
                    //     Metric::Tps,
                    // ].contains(&metric) {
                    //     plot = plot.include_y(100.);
                    // }

                    // let mut color = graph_color.linear_multiply(0.5);
                    // let color = graph_color.linear_multiply(0.15);
                    let color = graph_color.gamma_multiply(0.5);
                    // let color = graph_color;
                    // color[3] = 250;
                    let line = Line::new(PlotPoints::Owned(graph_data))
                        .color(color)
                        .style(LineStyle::Solid)
                        .fill(0.0);

                    let plot_result = plot.show(ui, |plot_ui| {
                        plot_ui.line(line);
                    });

                    let text = format!("{} {}", i18n(metric.title()).to_uppercase(), metric.format(value, true, true));
                    let rich_text = egui::RichText::new(text).size(10.).color(Color32::WHITE).raised();//.background_color(Color32::from_black_alpha(128));
                    let label = Label::new(rich_text);
                    let mut rect = plot_result.response.rect;
                    rect.set_bottom(rect.top() + 12.);
                    ui.put(rect, label);
                });
        });
    }
}

const METRICS : [Metric;23] = [
    Metric::CpuUsage,
    Metric::ResidentSetSizeBytes,
    // Metric::VirtualMemorySizeBytes,
    Metric::FdNum,
    Metric::DiskIoReadBytes,
    Metric::DiskIoWriteBytes,
    Metric::DiskIoReadPerSec,
    Metric::DiskIoWritePerSec,
    // Metric::BorshLiveConnections,
    // Metric::BorshConnectionAttempts,
    // Metric::BorshHandshakeFailures,
    // Metric::JsonLiveConnections,
    // Metric::JsonConnectionAttempts,
    // Metric::JsonHandshakeFailures,
    Metric::ActivePeers,
    Metric::BlocksSubmitted,
    Metric::HeaderCounts,
    Metric::DepCounts,
    Metric::BodyCounts,
    Metric::TxnCounts,
    Metric::Tps,
    Metric::ChainBlockCounts,
    Metric::MassCounts,
    Metric::BlockCount,
    Metric::HeaderCount,
    Metric::TipHashesCount,
    Metric::Difficulty,
    Metric::PastMedianTime,
    Metric::VirtualParentHashesCount,
    Metric::VirtualDaaScore,
];