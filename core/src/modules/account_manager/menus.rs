use egui_phosphor::thin::*;

use crate::imports::*;
use super::*;

#[derive(Default)]
pub struct WalletMenu { }

impl WalletMenu {
    // pub fn new() -> Self {
    //     Self { }
    // }

    pub fn render(&mut self, core: &mut Core, ui : &mut Ui, max_height: f32) {
        
        // let (wallet_name,wallet_filename) = if let Some(wallet_descriptor) = core.wallet_descriptor.as_ref() {
        //     (wallet_descriptor.title.as_deref().unwrap_or(wallet_descriptor.filename.as_str()).to_string(),wallet_descriptor.filename.clone())
        // } else {
        //     ui.label("Missing wallet descriptor");
        //     return;
        // };

        let wallet_name = if let Some(wallet_descriptor) = core.wallet_descriptor.as_ref() {
            wallet_descriptor.title.as_deref().unwrap_or(wallet_descriptor.filename.as_str()).to_string()
        } else {
            ui.label(i18n("Missing wallet descriptor"));
            return;
        };

        let label = if core.device().desktop(){
            format!("{} {} ⏷", i18n("Wallet:"), wallet_name)
        }else{
            format!("{} ⏷", wallet_name)
        };

        self.render_selector(core,ui,max_height,|ui|{ ui.add(Label::new(label).sense(Sense::click())) });
    }
    
    pub fn render_selector(
        &mut self,
        core: &mut Core,
        ui : &mut Ui,
        max_height: f32,
        widget: impl FnOnce(&mut Ui) -> Response,
    ) {
    
        let wallet_filename = if let Some(wallet_descriptor) = core.wallet_descriptor.as_ref() {
            wallet_descriptor.filename.clone()
        } else {
            ui.label(i18n("Missing wallet descriptor"));
            return;
        };

        // PopupPanel::new(PopupPanel::id(ui,"wallet_selector_popup"),|ui|{ ui.add(Label::new(format!("{} {} ⏷", i18n("Wallet:"), wallet_name)).sense(Sense::click())) }, |ui, _| {
        PopupPanel::new(PopupPanel::id(ui,"wallet_selector_popup"),widget, |ui, _| {

            ScrollArea::vertical()
                .id_salt("wallet_selector_popup_scroll")
                .auto_shrink([true; 2])
                .show(ui, |ui| {

                    let mut wallet_list = core.wallet_list().clone();
                    wallet_list.sort();
                    wallet_list.into_iter().for_each(|wallet_descriptor| {

                        let title = if let Some(title) = wallet_descriptor.title.clone() {
                            title
                        } else if wallet_descriptor.filename.as_str() == "kaspa" {
                            "Kaspa Wallet".to_string()
                        } else {
                            "NO NAME".to_string()
                        };


                        let icon = if wallet_descriptor.filename == wallet_filename {
                            // Composite::Icon(egui_phosphor::thin::LOCK_KEY_OPEN)
                            // Composite::Icon(egui_phosphor::thin::COINS)

                            if !core.state().is_connected() {
                                RichText::new(egui_phosphor::thin::CLOUD_X)
                            } else if !core.state().is_synced() {
                                RichText::new(egui_phosphor::thin::CLOUD_ARROW_DOWN)
                            } else {
                                RichText::new(egui_phosphor::thin::CLOUD_CHECK)
                            }

                        } else {
                            RichText::new(egui_phosphor::thin::FINGERPRINT_SIMPLE).color(Color32::DARK_GRAY)
                        };

                        if ui.add(CompositeButton::image_and_text(
                            Composite::icon(icon),
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

                    if ui.add_sized(theme_style().large_button_size,CompositeButton::opt_image_and_text(
                        // Some(Composite::icon(LIST)),
                        None,
                        Some(i18n("Create New Wallet").into()),
                        None,
                    )).clicked() {
                        core.select::<modules::WalletCreate>();
                    }
                });

        })
        .with_min_width(240.)
        .with_max_height(max_height)
        .with_close_on_interaction(true)
        .build(ui);

    }
}

#[derive(Default)]
pub struct AccountMenu { }

impl AccountMenu {
    // pub fn new() -> Self {
    //     Self { }
    // }

    pub fn render(
        &mut self, 
        core: &mut Core, 
        ui : &mut Ui, 
        max_height: f32,
        account_manager : &mut AccountManager, 
        rc : &RenderContext, 
        // rc : &RenderContext, 
    ) {

        let label = if core.device().desktop(){
            format!("{} {} ⏷",i18n("Account:"), rc.account.name_or_id())
        }else{
            format!("{} ⏷", rc.account.name_or_id())
        };

        self.render_selector(core,ui,max_height,account_manager,rc,|ui|{ ui.add(Label::new(label).sense(Sense::click())) });
        
    }

    pub fn render_selector(
        &mut self,
        core: &mut Core,
        ui : &mut Ui,
        max_height: f32,
        account_manager : &mut AccountManager, 
        // rc : &RenderContext, 
        rc : &RenderContext, 
        widget: impl FnOnce(&mut Ui) -> Response,
    ) {

        let RenderContext { account, network_type, .. } = rc;

        // PopupPanel::new(PopupPanel::id(ui,"account_selector_popup"),|ui|{ ui.add(Label::new(format!("{} {} ⏷",i18n("Account:"), account.name_or_id())).sense(Sense::click())) }, |ui, close| {
        PopupPanel::new(PopupPanel::id(ui,"account_selector_popup"),widget, |ui, close| {

            egui::ScrollArea::vertical()
                .id_salt("account_selector_popup_scroll")
                .auto_shrink([true; 2])
                .show(ui, |ui| {

                    let mut account_list = if let Some(account_collection) = core.account_collection() {
                        account_collection.list().clone()
                    } else {
                        ui.label(i18n("No accounts found"));
                        return;
                    };

                    if let Some(prv_key_data_map) = core.prv_key_data_map() {
                        
                        for prv_key_data_info in prv_key_data_map.values() {
                            CollapsingHeader::new(prv_key_data_info.name_or_id())
                                // .default_open(true)
                                .open(Some(true))
                                .show(ui, |ui| {

                                    account_list.retain(|selectable_account|{
                                        if selectable_account.descriptor().prv_key_data_ids().contains(&prv_key_data_info.id) {

                                            if ui.account_selector_button(selectable_account, network_type, account.id() == selectable_account.id(), core.balance_padding()).clicked() {
                                                account_manager.request_estimate();
                                                let wallet = core.wallet();
                                                let device = core.device().clone();

                                                account_manager.select(wallet,Some(selectable_account.clone()),device,true);
                                            }

                                            false
                                        } else {
                                            true
                                        }
                                    });
                            });
                        }
                    }

                    if account_list.is_not_empty() {
                        
                        ui.separator();

                        account_list.iter().for_each(|selectable_account|{
                            if ui.account_selector_button(selectable_account, network_type, account.id() == selectable_account.id(), core.balance_padding()).clicked() {
                                account_manager.request_estimate();
                                account_manager.state = AccountManagerState::Overview {
                                    account: selectable_account.clone(),
                                };
                            }
                        });
                    }

                    ui.add_space(8.);
                    ui.separator();
                    ui.add_space(8.);
                    if ui.add_sized(theme_style().large_button_size,CompositeButton::opt_image_and_text(
                        Some(Composite::icon(LIST)),
                        Some(i18n("Add Account").into()),
                        None,
                    )).clicked() {
                        *close = true;
                        core.select::<modules::AccountCreate>();
                    }
                });

        })
        .with_min_width(240.)
        .with_max_height(max_height)
        // .with_caption("Accounts")
        // .with_close_button(true)    
        .with_close_on_interaction(true)
        .build(ui);
    }
}


#[derive(Default)]
pub struct ToolsMenu { }

impl ToolsMenu {
    pub fn new() -> Self {
        Self { }
    }
    pub fn render(&mut self, core: &mut Core, ui : &mut Ui, _account_manager : &mut AccountManager, _rc : &RenderContext, max_height: f32) {

        PopupPanel::new(PopupPanel::id(ui,"tools_popup"),|ui|{ ui.add(Label::new(format!("{} ⏷", i18n("Tools"))).sense(Sense::click())) }, |ui, _| {

            egui::ScrollArea::vertical()
                .id_salt("tools_popup_scroll")
                .auto_shrink([true; 2])
                .show(ui, |ui| {

                    // let _ = ui.button(i18n("Create Account"));
                    // let _ = ui.button(i18n("Import"));
                    if ui.large_button(i18n("Export Wallet Data")).clicked() {
                        core.select::<modules::Export>();
                    }
                    if ui.large_button(i18n("Address derivation scan")).clicked() {
                        core.select::<modules::Scanner>();
                    }
                });

        })
        .with_min_width(240.)
        .with_max_height(max_height)
        .with_close_on_interaction(true)
        .build(ui);
    }
}

