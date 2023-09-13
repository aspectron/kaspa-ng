use crate::stages::*;
use crate::{imports::*, interop::spawn_with_result};
use egui::*;
// use workflow_core::task::spawn;

#[derive(Clone)]
pub enum State {
    Select,
    Unlock(Option<Arc<Error>>),
    Unlocking,
}

pub struct Open {
    #[allow(dead_code)]
    interop: Interop,
    secret: String,
    pub state: State,
    pub message: Option<String>,

    selected_wallet : Option<String>,

    back_color : Color32,
}

impl Open {
    pub fn new(interop: Interop) -> Self {
        Self {
            interop,
            secret: String::new(),
            state: State::Select,
            message: None,
            selected_wallet : None,
            back_color : Color32::from_rgb(0, 0, 0),
        }
    }

    pub fn lock(&mut self) {
        self.state = State::Unlock(None);
    }
}

impl SectionT for Open {
    fn render(
        &mut self,
        wallet: &mut Wallet,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {

        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {

            let size = egui::Vec2::new(200_f32, 40_f32);
            let unlock_result = Payload::<Result<()>>::new("test");

            match self.state.clone() {
                State::Select => {

                    if ui.label("HELLO CLICKED!").clicked() {
                        println!("HELLO CLICKED!");
                    }

                    if ui.add(egui::Button::new("test click")).clicked() {
                        println!("BUTTON CLICKED");
                    }

                    // if ui.add(Label::new(egui::RichText::new("HELLO"))).clicked() {
                    //     println!("HELLO CLICKED!");
                    // }

                    Panel::new(self)
                        .with_caption("Select Wallet")
                        .with_close(|_|{

                            println!("CLOSE CLICKED");
                        })
                        .with_header(|_ctx,ui| {
                            ui.label("Select a wallet to unlock");
                        })
                        .with_body(|ctx,ui| {
                            for wallet in wallet.wallet_list.iter() {
                                if ui.add_sized(size, egui::Button::new(wallet.filename.clone())).clicked() {
                                    ctx.selected_wallet = Some(wallet.filename.clone());
                                    ctx.state = State::Unlock(None);
                                }
                            }
                        })
                        .render(ui);
                }

                State::Unlock(error) => {

                    Panel::new(self)
                        .with_caption("Unlock Wallet")
                        .with_back(|ctx|{
                            println!("clicking BACK!");
                            ctx.state = State::Select;
                        })
                        .with_close(|_ctx|{})
                        .with_body(|ctx,ui| {
                            ui.label(" ");
                            ui.label(format!("Opening wallet: \"{}\"",ctx.selected_wallet.as_ref().unwrap()));
                            ui.label(" ");
    
                            if let Some(err) = error {
                                ui.label(" ");
                                ui.label(egui::RichText::new("Error unlocking wallet").color(egui::Color32::from_rgb(255, 120, 120)));
                                ui.label(egui::RichText::new(err.to_string()).color(egui::Color32::from_rgb(255, 120, 120)));
                                ui.label(" ");
                            }
    
                            ui.label("Enter your password to unlock your wallet");
                            ui.label(" ");
    
                            ui.add_sized(
                                size,
                                TextEdit::singleline(&mut ctx.secret)
                                    .hint_text("Enter Password...")
                                    .password(true)
                                    .vertical_align(Align::Center),
                            );
    
                            if ui
                                .add_sized(size, egui::Button::new("Unlock"))
                                .clicked()
                            {
                                let secret = kaspa_wallet_core::secret::Secret::new(ctx.secret.as_bytes().to_vec());
                                ctx.secret.zeroize();
                                let wallet = ctx.interop.wallet().clone();
                                let wallet_name = ctx.selected_wallet.clone();//.expect("Wallet name not set");
                                
                                spawn_with_result(&unlock_result, async move {
                                    wallet.load(secret, wallet_name).await?;
                                    Ok(())
                                });
    
                                ctx.state = State::Unlocking;
                            }
    
                            ui.label(" ");
    
                        })
                        // .with_footer(|ui|{
                        //     if ui
                        //         .add_sized(size, egui::Button::new("Select a different wallet"))
                        //         .clicked() {
                        //             self.state = State::Select;
                        //         }
                        // })
                        .render(ui);


                    // egui::ScrollArea::vertical()
                    //     .id_source("unlock-wallet")
                    //     .show(ui, |ui| {


                    //     if ui
                    //         .add_sized(size, egui::Button::new("Select a different wallet"))
                    //         .clicked() {
                    //             self.state = State::Select;
                    //         }
                    // });
                }
                State::Unlocking => {
                    ui.heading("Unlocking");
                    // ui.separator();
                    ui.label(" ");
                    ui.label("Unlocking wallet, please wait...");
                    ui.label(" ");
                    ui.add_space(64.);
                    ui.add(egui::Spinner::new().size(92.));

                    if let Some(result) = unlock_result.take() {

                        match result {
                            Ok(_) => {
                                println!("Unlock success");
                                // self.state = State::Unlock;
                                wallet.select::<section::Overview>();
                            }
                            Err(err) => {
                                println!("Unlock error: {}", err);
                                self.state = State::Unlock(Some(Arc::new(err)));
                            }
                        }
                        // ui.label(format!("Result: {:?}", result));
                        // _ctx.value = result.unwrap();
                        // Stage::Next
                    } else {
                        // Stage::Current
                    }

                }
            }

        });
    }
}
