use crate::imports::*;
use super::*;

pub struct WalletMenu { }

impl WalletMenu {
    pub fn new() -> Self {
        Self { }
    }

    pub fn render(&mut self, core: &mut Core, ui : &mut Ui, max_height: f32) {

        let wallet_name = if let Some(wallet_descriptor) = core.wallet_descriptor.as_ref() {
            wallet_descriptor.title.as_deref().unwrap_or(wallet_descriptor.filename.as_str())
        } else {
            ui.label("Missing wallet descriptor");
            return;
        };

        PopupPanel::new(ui, "wallet_selector_popup",format!("Wallet: {}", wallet_name), |ui| {

            ScrollArea::vertical()
                .id_source("wallet_selector_popup_scroll")
                .auto_shrink([true; 2])
                .show(ui, |ui| {

                    let wallet_list = core.wallet_list().clone();

                    wallet_list.into_iter().for_each(|wallet_descriptor| {

                        let title = if let Some(title) = wallet_descriptor.title.clone() {
                            title
                        } else if wallet_descriptor.filename.as_str() == "kaspa" {
                            "Kaspa Wallet".to_string()
                        } else {
                            "NO NAME".to_string()
                        };

                        if ui.add(CompositeButton::new(
                            title,
                            wallet_descriptor.filename.clone(),
                        )).clicked()
                        {
                            core.get_mut::<modules::WalletOpen>().open(wallet_descriptor.clone());
                            core.select::<modules::WalletOpen>();
                        }
                    });

                    ui.label("");
                    ui.separator();
                    ui.label("");

                    if ui.medium_button(
                        "Create New Wallet",
                    ).clicked()
                    {
                        core.select::<modules::WalletCreate>();
                    }
                });

        })
        .with_min_width(240.)
        .with_max_height(max_height)
        .with_caption(true)
        .with_close_button(true)
        .with_pulldown_marker(true)
        .build(ui);

    }
}

pub struct AccountMenu { }

impl AccountMenu {
    pub fn new() -> Self {
        Self { }
    }
    pub fn render(&mut self, core: &mut Core, ui : &mut Ui, account_manager : &mut AccountManager, rc : &RenderContext<'_>, max_height: f32) {
        let RenderContext { account, network_type, .. } = rc;
        PopupPanel::new(ui, "account_selector_popup",format!("Account: {}", account.name_or_id()), |ui| {

            egui::ScrollArea::vertical()
                .id_source("account_selector_popup_scroll")
                .auto_shrink([true; 2])
                .show(ui, |ui| {

                    if let Some(account_collection) = core.account_collection() {
                        account_collection.iter().for_each(|account| {
                            if ui
                                .button(format!("Select {}\n{}", account.name_or_id(),account.balance().map(|balance|sompi_to_kaspa_string_with_suffix(balance.mature, network_type)).unwrap_or("N/A".to_string())))
                                .clicked()
                            {
                                account_manager.state = AccountManagerState::Overview {
                                    account: account.clone(),
                                };
                            }
                        });

                        ui.label("");
                        ui.separator();
                        ui.label("");
                        use egui_phosphor::light::FOLDER_NOTCH_PLUS;
                        if ui.medium_button(format!("{FOLDER_NOTCH_PLUS} Create New Account")).clicked() {
                            core.select::<modules::AccountCreate>();
                        }
                    }

                });

        })
        .with_min_width(240.)
        .with_max_height(max_height)
        .with_caption(true)
        .with_close_button(true)    
        .with_pulldown_marker(true)
        .build(ui);
    }
}


pub struct ToolsMenu { }

impl ToolsMenu {
    pub fn new() -> Self {
        Self { }
    }
    pub fn render(&mut self, _core: &mut Core, ui : &mut Ui, _account_manager : &mut AccountManager, _rc : &RenderContext<'_>, max_height: f32) {

        PopupPanel::new(ui, "tools_popup",i18n("Tools"), |ui| {

            egui::ScrollArea::vertical()
                .id_source("tools_popup_scroll")
                .auto_shrink([true; 2])
                .show(ui, |ui| {

                    let _ = ui.button("Create Account");
                    let _ = ui.button("Import");
                    let _ = ui.button("Export");
                    // ui.button("Export");

                    // if let Some(account_collection) = core.account_collection() {
                    //     account_collection.iter().for_each(|account| {
                    //         if ui
                    //             .button(format!("Select {}\n{}", account.name_or_id(),account.balance().map(|balance|sompi_to_kaspa_string_with_suffix(balance.mature, network_type)).unwrap_or("N/A".to_string())))
                    //             .clicked()
                    //         {
                    //             account_manager.state = AccountManagerState::Overview {
                    //                 account: account.clone(),
                    //             };
                    //         }
                    //     });

                    //     ui.label("");
                    //     ui.separator();
                    //     ui.label("");
                    //     use egui_phosphor::light::FOLDER_NOTCH_PLUS;
                    //     if ui.medium_button(format!("{FOLDER_NOTCH_PLUS} Create New Account")).clicked() {
                    //         core.select::<modules::AccountCreate>();
                    //     }
                    // }

                });

        })
        .with_min_width(240.)
        .with_max_height(max_height)
        .with_pulldown_marker(true)
        .with_close_on_interaction(true)
        .build(ui);
    }
}

