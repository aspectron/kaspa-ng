use crate::imports::*;

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        pub struct System {
            pub cpu_physical_core_count : Option<usize>,
            pub cpu_frequency : Option<u64>,
            pub cpu_brand : Option<String>,
            pub total_memory : u64,
            pub long_os_version : Option<String>,
            pub fd_limit : i32,
        }

        impl Default for System {
            fn default() -> Self {
                Self::new()
            }
        }

        impl System {
            pub fn new() -> Self {

                use sysinfo::*;
                let mut system = System::new();
                system.refresh_cpu_specifics(CpuRefreshKind::new().with_frequency());
                system.refresh_memory();
                let cpus = system.cpus();
                let cpu_physical_core_count = system.physical_core_count();
                let long_os_version = system.long_os_version();
                let total_memory = system.total_memory();

                let (cpu_frequency,cpu_brand) = cpus
                    .first()
                    .map(|cpu|(cpu.frequency(),cpu.brand().to_string())).unzip();

                let fd_limit = kaspa_utils::fd_budget::limit();

                Self {
                    cpu_physical_core_count,
                    cpu_frequency,
                    cpu_brand,
                    total_memory,
                    long_os_version,
                    fd_limit,
                }
            }

            pub fn render(&self, ui: &mut Ui) {
                use kaspa_metrics_core::data::as_data_size;

                CollapsingHeader::new(i18n("System"))
                    .default_open(true)
                    .show(ui, |ui| {
                        if let Some(os) = self.long_os_version.clone() {
                            ui.label(os);
                        }

                        if let Some(cpu_physical_core_count) = self.cpu_physical_core_count {
                            if let Some(cpu_brand) = self.cpu_brand.as_ref() {
                                ui.label(cpu_brand.clone()); //format!("{cpu_brand}"));
                            }
                            let freq = self.cpu_frequency.map(|freq|format!(" @ {:.2} GHz", freq as f64 / 1000.0)).unwrap_or_default();
                            ui.label(format!("{} CPU cores {freq}", cpu_physical_core_count));
                        }
                        ui.label(format!("{} RAM", as_data_size(self.total_memory as f64, false)));
                        ui.label(format!("Handles: {}", self.fd_limit.separated_string()));
                    });
            }
        }
    } else {

        pub struct System {

        }

        impl System {
            pub fn new() -> Self {
                Self {

                }
            }

            pub fn render(&self, _ui: &mut Ui) {

            }
        }

        impl Default for System {
            fn default() -> Self {
                Self::new()
            }
        }
    }
}
