use crate::imports::*;
use crate::egui::easy_mark::easy_mark;
pub struct Changelog {
    #[allow(dead_code)]
    interop: Interop,
    changelog : &'static str,
    // account : Option<Arc<dyn runtime::Account>>,
}

impl Changelog {
    pub fn new(interop: Interop) -> Self {

        // let changelog = ;

        // let changelog = std::env::current_exe().and_then(|mut path| {
        //     path.pop();
        //     path.push("CHANGELOG.md");
        //     // Ok(path)
        //     if path.exists() {
        //         Ok(path)
        //     } else {
        //         Err(std::io::Error::new(std::io::ErrorKind::NotFound, path.to_string_lossy()))
        //     }
        // }).and_then(|path| {
        //     std::fs::read_to_string(path)
        // }).unwrap_or_else(|err| format!("# Unable to read CHANGELOG.md\n\n# {err}"));
        //.unwrap_or_else(|_| PathBuf::from("CHANGELOG.md"));
        
        //.unwrap_or_else(|_| PathBuf::from("CHANGELOG.md"))

        // let changelog = std::fs::read_to_string(path).unwrap_or_else(|| "# Unable to read CHANGELOG.md".to_string());

        Self { 
            interop,
            changelog : include_str!("../../../CHANGELOG.md")
        }
    }

    // pub fn select(&mut self, account : Option<Arc<dyn runtime::Account>>) {
    //     self.account = account;
    // }
}

impl ModuleT for Changelog {

    fn style(&self) -> ModuleStyle {
        ModuleStyle::Default
    }

    fn render(
        &mut self,
        _wallet: &mut Wallet,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            // .stick_to_bottom(true)
            .show(ui, |ui| {
                easy_mark(ui, self.changelog.as_str());
            });
    }
}
