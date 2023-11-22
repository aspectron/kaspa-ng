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

        let screen_rect = ui.ctx().screen_rect();
        let logo_size = vec2(648., 994.,) * 0.25;
        let left = screen_rect.width() - logo_size.x - 8.;
        let top = 32.;
        let logo_rect = Rect::from_min_size(Pos2::new(left, top), logo_size);

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

                        if core.state().is_connected() {
                            self.render_graphs(core,ui);
                        } else {
                            ui.label(i18n("Not connected"));
                        }
                    });

                if let Some(system) = runtime().system() {
                    system.render(ui);
                }

                ui.add_space(48.);
            });
        });

        SidePanel::right("overview_right")
            .exact_width(width/2.)
            .resizable(false)
            .show_separator_line(false)
            .show_inside(ui, |ui| {

                Image::new(ImageSource::Bytes { uri : Cow::Borrowed("bytes://logo.svg"), bytes : Bytes::Static(crate::app::KASPA_NG_LOGO_SVG)})
                    .maintain_aspect_ratio(true)
                    .max_size(logo_size)
                    .fit_to_exact_size(logo_size)
                    .shrink_to_fit()
                    .texture_options(TextureOptions::LINEAR)
                    .tint(Color32::from_f32(0.8))
                    .paint_at(ui, logo_rect);

                egui::ScrollArea::vertical()
                    .id_source("overview_metrics")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
            
                        CollapsingHeader::new(i18n("Market"))
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.label("TODO");
                            });

                        CollapsingHeader::new(i18n("Resources"))
                            .default_open(true)
                            .show(ui, |ui| {
                                // egui::special_emojis
                                // use egui_phosphor::light::{DISCORD_LOGO,GITHUB_LOGO};
                                Hyperlink::from_label_and_url(
                                    format!("• {}",i18n("Kaspa NextGen on GitHub")),
                                    "https://github.com/aspectron/kaspa-ng"
                                ).open_in_new_tab(true).ui(ui);
                                Hyperlink::from_label_and_url(
                                    format!("• {}",i18n("Rusty Kaspa on GitHub")),
                                    "https://github.com/kaspanet/rusty-kaspa",
                                ).open_in_new_tab(true).ui(ui);
                                Hyperlink::from_label_and_url(
                                    format!("• {}",i18n("NPM Modules for NodeJS")),
                                    "https://www.npmjs.com/package/kaspa",
                                ).open_in_new_tab(true).ui(ui);
                                Hyperlink::from_label_and_url(
                                    format!("• {}",i18n("WASM SDK for JavaScript and TypeScript")),
                                    "https://github.com/kaspanet/rusty-kaspa/wasm",
                                ).open_in_new_tab(true).ui(ui);
                                Hyperlink::from_label_and_url(
                                    format!("• {}",i18n("Rust Wallet SDK")),
                                    "https://docs.rs/kaspa-wallet-core/0.0.4/kaspa_wallet_core/",
                                ).open_in_new_tab(true).ui(ui);
                                Hyperlink::from_label_and_url(
                                    format!("• {}",i18n("Kaspa Discord")),
                                    "https://discord.com/invite/kS3SK5F36R",
                                ).open_in_new_tab(true).ui(ui);
                            });

                        let version = env!("CARGO_PKG_VERSION");
                        let download = |platform: &str| { format!("https://github.com/aspectron/kaspa-ng/releases/download/{}/kaspa-ng-{}-{}.zip", version, version, platform) };
                        CollapsingHeader::new(i18n("Redistributables"))
                            .default_open(false)
                            .show(ui, |ui| {
                                ["windows-x64", "linux-gnu-amd64", "macos-arm64"].into_iter().for_each(|platform| {
                                    Hyperlink::from_label_and_url(
                                        format!("• kaspa-ng-{}-{}.zip", version, platform),
                                        download(platform),
                                    ).open_in_new_tab(true).ui(ui);
                                });
                            });

                        CollapsingHeader::new(i18n("Music"))
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.label("TODO");
                            });


                        CollapsingHeader::new(i18n("Build"))
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.label(format!("Kaspa NG v{}-{} + Rusty Kaspa v{}", env!("CARGO_PKG_VERSION"),crate::app::GIT_DESCRIBE, kaspa_wallet_core::version()));
                                ui.label(format!("Timestamp: {}", crate::app::BUILD_TIMESTAMP));
                                ui.label(format!("rustc {}-{} {}  llvm {}", 
                                    crate::app::RUSTC_SEMVER,
                                    crate::app::RUSTC_COMMIT_HASH.chars().take(8).collect::<String>(),
                                    crate::app::RUSTC_CHANNEL,
                                    crate::app::RUSTC_LLVM_VERSION,
                                ));
                                ui.label(format!("architecture {}", 
                                    crate::app::CARGO_TARGET_TRIPLE
                                ));
                                ui.label("Codename: \"This is the way\"");
                            });


                        CollapsingHeader::new(i18n("License Information"))
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.label("TODO");
                            });
                    });

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

        // let last_connect_time = core.state().last_connect_time();
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
            let available_samples = runtime().metrics_service().samples_since_connection();
            if available_samples < duration {
                duration = available_samples;
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

                    if [Metric::NodeCpuUsage].contains(&metric) {
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

                    let text = format!("{} {}", i18n(metric.title().1).to_uppercase(), metric.format(value, true, true));
                    let rich_text = egui::RichText::new(text).size(10.).color(Color32::WHITE).raised();//.background_color(Color32::from_black_alpha(128));
                    let label = Label::new(rich_text).wrap(false);
                    let mut rect = plot_result.response.rect;
                    rect.set_bottom(rect.top() + 12.);
                    // rect.set_right(rect.left() + 12.);
                    ui.put(rect, label);

                    // plot_result.response.on_hover_text("Test 123");
                });
        });
    }
}

const METRICS : [Metric;23] = [
    Metric::NodeCpuUsage,
    Metric::NodeResidentSetSizeBytes,
    // Metric::VirtualMemorySizeBytes,
    Metric::NodeFileHandlesCount,
    Metric::NodeDiskIoReadBytes,
    Metric::NodeDiskIoWriteBytes,
    Metric::NodeDiskIoReadPerSec,
    Metric::NodeDiskIoWritePerSec,
    // Metric::BorshLiveConnections,
    // Metric::BorshConnectionAttempts,
    // Metric::BorshHandshakeFailures,
    // Metric::JsonLiveConnections,
    // Metric::JsonConnectionAttempts,
    // Metric::JsonHandshakeFailures,
    Metric::NodeActivePeers,
    Metric::NodeBlocksSubmittedCount,
    Metric::NodeHeadersProcessedCount,
    Metric::NodeDependenciesProcessedCount,
    Metric::NodeBodiesProcessedCount,
    Metric::NodeTransactionsProcessedCount,
    Metric::NodeChainBlocksProcessedCount,
    Metric::NodeMassProcessedCount,
    Metric::NodeDatabaseBlocksCount,
    Metric::NodeDatabaseHeadersCount,
    Metric::NetworkTransactionsPerSecond,
    Metric::NetworkTipHashesCount,
    Metric::NetworkDifficulty,
    Metric::NetworkPastMedianTime,
    Metric::NetworkVirtualParentHashesCount,
    Metric::NetworkVirtualDaaScore,
];