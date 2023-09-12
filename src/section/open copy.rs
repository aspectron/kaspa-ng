use crate::stages::*;
use crate::{imports::*, interop::spawn_with_result};
use egui::*;
// use workflow_core::task::spawn;

pub enum State {
    Select,
    Unlock(Option<String>),
    Unlocking,
}

pub struct Open {
    #[allow(dead_code)]
    interop: Interop,
    secret: String,
    pub state: State,
    pub message: Option<String>,

    selected_wallet : Option<String>,
}

impl Open {
    pub fn new(interop: Interop) -> Self {
        Self {
            interop,
            secret: String::new(),
            state: State::Select,
            message: None,
            selected_wallet : None,
        }
    }

    pub fn lock(&mut self) {
        self.state = State::Unlock(None);
    }

/*
    fn render_unlocking(&mut self, ui: &mut Ui, _wallet: &mut Wallet) {
        ui.heading("Unlocking");
        ui.separator();
        ui.label("Unlocking wallet, please wait...");
    }

    fn render_locked(&mut self, ui: &mut Ui, wallet: &mut Wallet) {
        let size = egui::Vec2::new(200_f32, 40_f32);

        if let Some(message) = &self.message {
            ui.label(" ");
            // ui.add(egui::Label::new(message));
            // ui.label(egui::RichText::new(message).heading().color(egui::Color32::from_rgb(255, 255, 255)));
            // ui.label(egui::RichText::new(message).heading().color(egui::Color32::from_rgb(255, 128, 128)));
            ui.label(egui::RichText::new(message).color(egui::Color32::from_rgb(255, 128, 128)));
            ui.label(" ");
        }

        // ui.add(egui::Label::new("Password"));
        ui.label(" ");


        // let list = wallet.wallet_list.keys().collect::<Vec<_>>();
        // if self.selected_wallet.is_none() {
        //     if wallet.wallet_list.contains_key("kaspa") {
        //         self.selected_wallet = Some("kaspa".to_string());
        //     } else {
        //         self.selected_wallet = list.first().map(|x| x.to_string());
        //     }
        // }


        if !wallet.wallet_list.is_empty() {

            for wallet in wallet.wallet_list.iter() {
                if ui.add_sized(size, egui::Button::new(wallet.filename.clone())).clicked() {

                }
            }

            // egui::ComboBox::from_id_source("wallet-selector")
            // .selected_text(self.selected_wallet.clone().unwrap_or("Please select...".to_string()))
            // .show_ui(ui, |ui| {
            //         ui.style_mut().wrap = Some(false);
            //         ui.set_min_width(60.0);
            //         wallet.wallet_list.iter().for_each(|(filename,descriptor)| {
            //             ui.selectable_value(&mut self.selected_wallet, Some(filename.clone()), format!("{} ({})", filename, descriptor.title.clone().unwrap_or_default()));
            //         });
            //     });

        }

        ui.label(" ");

        ui.add_sized(
            size,
            TextEdit::singleline(&mut self.secret)
                .hint_text("Enter Password...")
                .password(true)
                .vertical_align(Align::Center),
        );

        // ui.add_sized(egui::Vec2::new(120_f32,40_f32), egui::Button::new("Testing 123"));

        if ui.add_sized(size, egui::Button::new("Unlock")).clicked() {
            println!("secret: {}", self.secret);
            let secret = kaspa_wallet_core::secret::Secret::new(self.secret.as_bytes().to_vec());
            // unsafe {
            self.secret.zeroize();
            self.state = State::Unlocking;
            // self.sender.try_send(Events::TryUnlock(secret.into())).unwrap();

            // wallet.spawn(|wallet| async move {
            //     wallet.wallet().load(secret,None).await
            // }).map(|wallet : &mut Wallet, ok| {
            //     println!("Wallet unlock success: {:?}", ok);
            //     wallet.select(Section::Overview);
            // }).or_else(|wallet : &mut Wallet, err| {
            //     wallet.select(Section::Unlock);
            //     let s = wallet.get_mut(Section::Unlock);
            //     // wallet
            //     println!("Wallet unlock error: {:?}", err);
            // });

            let _sender = wallet.sender();
            let wallet = wallet.wallet().clone();
            // let s = secret.0;

            // spawn(wallet.load(secret,None));

            spawn(async move {
                println!("inside executor spawn...");
                // let result =
                wallet.load(secret, None).await?;
                // println!("Wallet unlock result: {:?}", result);
                Ok(())
            });

            // let channel = wallet.spawn(wallet.wallet().load(secret, None));
        }

        ui.label(" ");
        if ui.add_sized(size, egui::Button::new("Test")).clicked() {
            println!("[0] secret: {}", self.secret);

            with_secret(|secret| {
                println!("[1] secret: {:?}", secret);
            });
        }
        if ui.add_sized(size, egui::Button::new("Test2")).clicked() {
            println!("[0] secret: {}", self.secret);

            #[derive(Default, Debug)]
            struct Test {
                value: usize,
            }

            crate::stages::Stages::<Test>::default()
                .with_window(|_ctx| egui::Window::new("Wizard Window"))
                // .stage::<_, fn(&mut Ui, &mut Test) -> Disposition>(|ui: &mut Ui, ctx: &mut Test| {
                .stage(|ui: &mut Ui, _ctx: &mut Test| {
                    ui.label("stage 1");
                    // if ui.add(egui::Button::new("Prev")).clicked() {
                    //     return Disposition::Previous;
                    // }

                    if ui.add(egui::Button::new("Next")).clicked() {
                        return Stage::Next;
                    }
                    Stage::Current
                })
                .stage(|ui: &mut Ui, _ctx: &mut Test| {
                    ui.label("stage 2");
                    if ui.add(egui::Button::new("Prev")).clicked() {
                        return Stage::Previous;
                    }

                    if ui.add(egui::Button::new("Next")).clicked() {
                        return Stage::Next;
                    }
                    if ui.add(egui::Button::new("Cancel")).clicked() {
                        return Stage::Cancel;
                    }

                    let payload = Payload::<Result<usize>>::new("test");

                    if ui
                        .add_enabled(!payload.is_pending(), egui::Button::new("Test SPAWN"))
                        .clicked()
                    {
                        spawn_with_result(&payload, async move {
                            // payload.store(());
                            Ok(123)
                        });
                        return Stage::Current;
                    }

                    if let Some(result) = payload.take() {
                        // ui.label(format!("Result: {:?}", result));
                        _ctx.value = result.unwrap();
                        Stage::Next
                    } else {
                        Stage::Current
                    }
                })
                .stage(|ui: &mut Ui, _ctx: &mut Test| {
                    ui.label("stage 3");

                    ui.label(format!("Result: {:?}", _ctx.value));

                    if ui.add(egui::Button::new("Prev")).clicked() {
                        return Stage::Previous;
                    }

                    if ui.add(egui::Button::new("Next")).clicked() {
                        return Stage::Next;
                    }
                    Stage::Current
                })
                .finish(|ctx| {
                    println!("finish: {:#?}", ctx);
                });

            // cascade(
            //     |ui: &mut Ui| {
            //         println!("[1] secret: ");

            //         if ui.add(egui::Button::new("Test123")).clicked() {
            //             return Some(123u64);
            //         }
            //         if ui.add(egui::Button::new("Test456")).clicked() {
            //             return Some(456u64);
            //         }

            //         // None
            //         Some(768u64)
            //     },
            //     |v| {
            //         println!("*** V: {v}");
            //     },
            // );
        }
    }
*/

}

impl SectionT for Open {
    fn render(
        &mut self,
        wallet: &mut Wallet,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        // ui.horizontal_centered(|ui| {

        //     ui.heading("Unlock");
        // });
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {

            let size = egui::Vec2::new(200_f32, 40_f32);
            let unlock_result = Payload::<Result<()>>::new("test");

            match &self.state {
                State::Select => {
                    ui.heading("Select Wallet");
                    ui.separator();
                    ui.label("Select a wallet to unlock");
                    ui.separator();
                    ui.label(" ");

                    for wallet in wallet.wallet_list.iter() {
                        if ui.add_sized(size, egui::Button::new(wallet.filename.clone())).clicked() {
                            self.selected_wallet = Some(wallet.filename.clone());
                            self.state = State::Unlock(None);
                        }
                    }
        
                }
                State::Unlock(message) => {
                    ui.heading("Unlock Wallet");
                    ui.separator();

                    if let Some(message) = message {
                        ui.label(" ");
                        ui.label(message);
                        ui.label(" ");
                    }

                    ui.label("Enter your password to unlock your wallet");
                    ui.separator();
                    ui.label(" ");

                    ui.add_sized(
                        size,
                        TextEdit::singleline(&mut self.secret)
                            .hint_text("Enter Password...")
                            .password(true)
                            .vertical_align(Align::Center),
                    );



                    if ui
                        // .add_sized(!payload.is_pending(), egui::Button::new("Test SPAWN"))
                        .add_sized(size, egui::Button::new("Unlock"))
                        .clicked()
                    {
                        let secret = kaspa_wallet_core::secret::Secret::new(self.secret.as_bytes().to_vec());
                        self.secret.zeroize();
                        let wallet = self.interop.wallet().clone();
                        let wallet_name = self.selected_wallet.clone();//.expect("Wallet name not set");
                        
                        spawn_with_result(&unlock_result, async move {
                            wallet.load(secret, wallet_name).await?;
                            Ok(())
                        });
                        // return Stage::Current;
                        self.state = State::Unlocking;
                    }


                    if ui
                        .add_sized(size, egui::Button::new("Select a different wallet"))
                        .clicked() {
                            self.state = State::Select;
                        }


                    // if ui.add_sized(size, egui::Button::new("Unlock")).clicked() {
                    //     println!("secret: {}", self.secret);
                    //     let secret = kaspa_wallet_core::secret::Secret::new(self.secret.as_bytes().to_vec());
                    //     // unsafe {
                    //     self.secret.zeroize();
                    //     self.state = State::Unlocking;
                    // }                        
                }
                State::Unlocking => {
                    ui.heading("Unlocking");
                    ui.separator();
                    ui.label("Unlocking wallet, please wait...");

                    if let Some(result) = unlock_result.take() {

                        match result {
                            Ok(_) => {
                                println!("Unlock success");
                                // self.state = State::Unlock;
                                wallet.select::<section::Overview>();
                            }
                            Err(err) => {
                                println!("Unlock error: {}", err);
                                self.state = State::Unlock(Some(err.to_string()));
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



            // #[derive(Default, Debug)]
            // struct Test {
            //     value: usize,
            // }


            // crate::stages::Stages::<Test>::default()
            //     .with_window(|_ctx| egui::Window::new("Wizard Window"))
            //     // .stage::<_, fn(&mut Ui, &mut Test) -> Disposition>(|ui: &mut Ui, ctx: &mut Test| {
            //     .stage(|ui: &mut Ui, _ctx: &mut Test| {
            //         ui.label("stage 1");
            //         // if ui.add(egui::Button::new("Prev")).clicked() {
            //         //     return Disposition::Previous;
            //         // }

            //         if ui.add(egui::Button::new("Next")).clicked() {
            //             return Stage::Next;
            //         }
            //         Stage::Current
            //     });


            // ui.label(" ");
            // ui.heading("Unlock your wallet");
            // ui.separator();
            // match self.state {
            //     State::Locked => self.render_locked(ui, wallet),
            //     State::Unlocking => self.render_unlocking(ui, wallet),
            // }
        });
    }
}
