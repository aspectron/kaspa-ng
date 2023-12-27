use crate::imports::*;
use super::*;

pub struct Processor<'context> {
    context: &'context mut ManagerContext,
}

impl<'context> Processor<'context> {
    pub fn new(context: &'context mut ManagerContext) -> Self {
        Self { context }
    }

    pub fn render(&mut self, core : &mut Core, ui: &mut Ui, rc : &RenderContext<'_>) {

        let RenderContext { account, network_type, .. } = rc;

        ui.add_space(8.);
        match self.context.transaction_kind.as_ref().unwrap() {
            TransactionKind::Send => {
                ui.label("Sending funds");
                ui.add_space(8.);
            }
            TransactionKind::Transfer => {
                // ui.label("Transferring funds");
            }
        }

        let send_result = Payload::<Result<GeneratorSummary>>::new("send_result");

        match &self.context.action {
            Action::Estimating => {

                let request_estimate = Estimator::new(self.context).render(core, ui, rc);

                if request_estimate {

                    let priority_fees_sompi = if self.context.enable_priority_fees {
                        self.context.priority_fees_sompi
                    } else { 0 };

                    let address = match network_type {
                        NetworkType::Testnet => Address::try_from("kaspatest:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhqrxplya").unwrap(),
                        NetworkType::Mainnet => Address::try_from("kaspa:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqkx9awp4e").unwrap(),
                        _ => panic!("Unsupported network"),
                    };

                    let account_id = account.id();
                    let payment_output = PaymentOutput {
                        address,
                        amount: self.context.send_amount_sompi,
                    };

                    let status = self.context.estimate.clone();
                    spawn(async move {
                        let request = AccountsEstimateRequest {
                            account_id,
                            destination: payment_output.into(),
                            priority_fee_sompi: Fees::SenderPaysAll(priority_fees_sompi),
                            payload: None,
                        };

                        match runtime().wallet().accounts_estimate_call(request).await {
                            Ok(response) => {
                                *status.lock().unwrap() = EstimatorStatus::GeneratorSummary(response.generator_summary);
                            }
                            Err(error) => {
                                *status.lock().unwrap() = EstimatorStatus::Error(error.to_string());
                            }    
                        }

                        runtime().egui_ctx().request_repaint();
                        Ok(())
                    });
                } 

            }

            Action::Sending => {

                let proceed_with_send = WalletSecret::new(self.context).render(ui, core, rc);

                if proceed_with_send {

                    if self.context.destination_address_string.is_not_empty() && self.context.transfer_to_account.is_some() {
                        unreachable!("expecting only one of destination address or transfer to account");
                    }

                    let priority_fees_sompi = if self.context.enable_priority_fees {
                        self.context.priority_fees_sompi
                    } else { 0 };

                    let wallet_secret = Secret::try_from(self.context.wallet_secret.clone()).expect("expecting wallet secret");
                    let payment_secret = account.requires_bip39_passphrase(core).then_some(Secret::try_from(self.context.payment_secret.clone()).expect("expecting payment secret"));

                    match self.context.transaction_kind.unwrap() {
                        TransactionKind::Send => {

                            let address = Address::try_from(self.context.destination_address_string.as_str()).expect("invalid address");
                            let account_id = account.id();
                            let payment_output = PaymentOutput {
                                address,
                                amount: self.context.send_amount_sompi,
                            };
        
                            spawn_with_result(&send_result, async move {
                                let request = AccountsSendRequest {
                                    account_id,
                                    destination: payment_output.into(),
                                    wallet_secret,
                                    payment_secret,
                                    priority_fee_sompi: Fees::SenderPaysAll(priority_fees_sompi),
                                    payload: None,
                                };
        
                                let generator_summary = runtime().wallet().accounts_send_call(request).await?.generator_summary;
                                runtime().request_repaint();
                                Ok(generator_summary)
                            });

                        }

                        TransactionKind::Transfer => {
                            let destination_account_id = self.context.transfer_to_account.as_ref().expect("transfer destination account").id();
                            let source_account_id = account.id();
                            let transfer_amount_sompi = self.context.send_amount_sompi;

                            spawn_with_result(&send_result, async move {
                                let request = AccountsTransferRequest {
                                    source_account_id,
                                    destination_account_id,
                                    wallet_secret,
                                    payment_secret,
                                    priority_fee_sompi: Some(Fees::SenderPaysAll(priority_fees_sompi)),
                                    transfer_amount_sompi,
                                };
        
                                let generator_summary = runtime().wallet().accounts_transfer_call(request).await?.generator_summary;
                                runtime().request_repaint();
                                Ok(generator_summary)
                            });
                        }
                    }
            
                    self.context.action = Action::Processing;
                }

            }
            Action::Processing => {
                ui.add_space(16.);
                ui.add(egui::Spinner::new().size(92.));

                if let Some(result) = send_result.take() {
                    match result {
                        Ok(_) => {
                            self.context.reset_send_state();
                            self.context.action = Action::None;
                        }
                        Err(error) => {
                            println!();
                            println!("Transaction error: {error}");
                            println!();
                            self.context.reset_send_state();
                            self.context.action = Action::Error(Arc::new(error));
                        }
                    }
                }
            }
            _ => { }
        }

    }
}