use crate::imports::*;
use egui_phosphor::light::CLIPBOARD_TEXT;
// use kaspa_rpc_core::hash;
use std::{borrow::Cow, collections::hash_map::Entry};
pub use xxhash_rust::xxh3::xxh3_64;

pub struct RequestUri {
    pub address : String,
    pub amount_sompi : Option<u64>,
    pub label : Option<String>,
}

impl std::fmt::Display for RequestUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut uri = self.address.clone();
        if let Some(amount_sompi) = self.amount_sompi {
            uri = format!("{}?amount={}", uri, sompi_to_kaspa(amount_sompi));
        }
        if let Some(label) = self.label.as_ref() {
            uri = format!("{}?label={}", uri, label);
        }
        write!(f, "{}", uri)
    }
}

pub struct Request {
    #[allow(dead_code)]
    runtime: Runtime,
    account : Option<Account>,
    qr : HashMap<String, (String,load::Bytes)>,
    amount : String,
    amount_sompi : Option<u64>,
    label : String,
    error : Option<String>,
}

impl Request {
    pub fn new(runtime: Runtime) -> Self {
        Self { 
            runtime,
            account : None,
            qr : Default::default(),
            amount : String::default(),
            amount_sompi : None,
            label : String::default(),
            error : None,
        }
    }

    fn create_request_uri(&self, address : impl Into<String>, amount_sompi : Option<u64>, label : Option<impl Into<String>>) -> RequestUri {

        RequestUri {
            address : address.into(),
            amount_sompi,
            label : label.map(|l|l.into()),
        }
    }

    fn qr(&mut self, request_uri : &str) -> (String,load::Bytes) {

        let hash = format!("{:x}",xxh3_64(format!("{request_uri}{}", theme_color().name).as_bytes()));
        let (qr_uri,qr_bytes) = match self.qr.entry(hash.clone()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let uri = format!("bytes://{hash}.svg");
                // let bits = qrcode::types::Mode::Alphanumeric.data_bits_count(request_uri.len());
                let qr = render_qrcode_with_version(request_uri, 192, 192, qrcode::Version::Normal(10));
                entry.insert((uri, qr.as_bytes().to_vec().into()))
            },
        };

        (qr_uri.clone(),qr_bytes.clone())
    }

    fn render_destination(&mut self, ui: &mut Ui, address : &str, request_uri : &RequestUri ) {

        let request_uri = request_uri.to_string();

        let (qr_uri, qr_bytes) = self.qr(request_uri.as_str());


        let response = ui.add(Label::new(format!("Address: {} {CLIPBOARD_TEXT}", format_address_string(address, Some(12)))).sense(Sense::click()))
        .on_hover_ui_at_pointer(|ui|{
            ui.vertical(|ui|{
                ui.label(i18n("Click to copy address to clipboard"));
            });
        });

        if response.clicked() {
            //ui.output_mut(|o| address.clone_into(&mut o.copied_text));
            ui.ctx().copy_text(address.to_string());
            runtime().notify_clipboard(i18n("Address copied to clipboard"));
        }

        ui.label(" ");
        
        // --

        let response = ui.add(Label::new(format!("URI: {} {CLIPBOARD_TEXT}", format_partial_string(request_uri.as_str(), Some(24)))).sense(Sense::click()))
        .on_hover_ui_at_pointer(|ui|{
            ui.vertical(|ui|{
                ui.label(i18n("Click to copy URI to clipboard"));
            });
        });

        if response.clicked() {
            //ui.output_mut(|o| address.clone_into(&mut o.copied_text));
            ui.ctx().copy_text(address.to_string());
            runtime().notify_clipboard(i18n("URI copied to clipboard"));
        }

        ui.label(" ");

        // --

        ui.add(
            Image::new(ImageSource::Bytes { uri : Cow::Owned(qr_uri), bytes: qr_bytes })
            .fit_to_original_size(1.0)
            .texture_options(TextureOptions::NEAREST)
        );
        
        // ui.label(" ");

        // --

    }

    pub fn select(&mut self, account : &Account) {
        self.account = Some(account.clone());
    }

}

impl ModuleT for Request {

    fn style(&self) -> ModuleStyle {
        ModuleStyle::Mobile
    }

    fn deactivate(&mut self, _core: &mut Core) {
        self.account = None;
        self.error = None;
        self.qr.clear();
    }

    fn render(
        &mut self,
        core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {

        let close = Rc::new(RefCell::new(false));

        Panel::new(self)
            .with_caption(i18n("Payment Request"))
            .with_back_enabled(core.has_stack(), |_|{
                *close.borrow_mut() = true;
            })
            .with_header(|this, ui| {
                if let Some(account) = this.account.as_ref() {
                    ui.label("");
                    ui.label(i18n_args("Payment request to account: {account}", &[("account", account.name_or_id())]));
                }
                // ui.label(text);
            })
            .with_body(|this, ui| {


                if let Some(account) = this.account.as_ref() {
                    let address = account.receive_address().to_string();
                    let label = this.label.is_not_empty().then_some(this.label.clone());
                    let request_uri = this.create_request_uri(address.clone(), this.amount_sompi, label);

                    this.render_destination(ui, address.as_str(), &request_uri);
                }

                ui.label("");

                ui.label(i18n("Enter the amount"));
                
                let amount = this.amount.clone();
                ui.add_sized(
                    theme_style().panel_editor_size,
                    TextEdit::singleline(&mut this.amount)
                        .vertical_align(Align::Center),
                );

                if amount != this.amount {
                    match try_kaspa_str_to_sompi(this.amount.as_str()) {
                        Ok(Some(amount_sompi)) => {
                            this.amount_sompi = Some(amount_sompi);
                            this.error = None;
                        },
                        Ok(None) => {
                            this.amount_sompi = None;
                            this.error = None;
                        },
                        Err(_err) => {
                            this.amount_sompi = None;
                            this.error = Some(i18n("Please enter a valid amount of KAS").to_string());
                        },
                    }
                }

                if let Some(error) = this.error.as_ref() {
                    ui.label("");
                    ui.colored_label(error_color(), error);
                }

                ui.label(" ");

            })
            .with_footer(|_ctx, ui| {
                if ui.large_button(i18n("Close")).clicked() {
                    *close.borrow_mut() = true;
                }
            })
            .render(ui);

        if *close.borrow() {
            core.back();
        }
    }
}
