use crate::imports::*;
use super::*;

pub struct Processor<'context> {
    context: &'context mut ManagerContext,
}

impl<'context> Processor<'context> {
    pub fn new(context: &'context mut ManagerContext) -> Self {
        Self { context }
    }

    pub fn render(&mut self, core : &mut Core, ui: &mut Ui, rc : &RenderContext) {

        let RenderContext { account, network_type, .. } = rc;
        let network_type = *network_type;

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

                    // let priority_fees_sompi = if self.context.enable_priority_fees {
                    // let priority_fees_sompi = self.context.priority_fees_sompi;
                    // } else { 0 };
                    // let fee_rate = self.context.priority_fee_rate + 1.0;
                    // println!("PROCESSOR: fee_rate: {}", fee_rate);


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

                    let priority_fee_sompi = self.context.priority_fees_sompi;

                    let status = self.context.estimate.clone();
                    spawn(async move {
                        let base_request = AccountsEstimateRequest {
                            account_id,
                            destination: payment_output.clone().into(),
                            // priority_fee_sompi: Fees::SenderPays(priority_fees_sompi),
                            priority_fee_sompi: Fees::SenderPays(0),
                            fee_rate: Some(0.0),
                            payload: None,
                        };

                        let base_result = runtime().wallet().accounts_estimate_call(base_request).await;

                        let base_mass = base_result.as_ref().map(|r| r.generator_summary.aggregate_mass).unwrap_or_default();

                        let fee_rate = if base_mass == 0 {
                            1.0
                        } else {
                            (priority_fee_sompi as f64 / base_mass as f64) + 1.0
                        };

                        let actual_request = AccountsEstimateRequest {
                            account_id,
                            destination: payment_output.into(),
                            // priority_fee_sompi: Fees::SenderPays(priority_fees_sompi),
                            priority_fee_sompi: Fees::SenderPays(0),
                            fee_rate: Some(fee_rate),
                            payload: None,
                        };

                        let actual_result = runtime().wallet().accounts_estimate_call(actual_request).await;

                        // match runtime().wallet().accounts_estimate_call(request).await {
                        match (base_result, actual_result) {
                            (Ok(base_estimate_response), Ok(actual_estimate_response)) => {
                                *status.lock().unwrap() = EstimatorStatus::GeneratorSummary { base_estimate : base_estimate_response.generator_summary, actual_estimate : actual_estimate_response.generator_summary };
                            }
                            (_,Err(error)) => {
                                *status.lock().unwrap() = EstimatorStatus::Error(error.to_string());
                            }
                            (_,_) => {
                                *status.lock().unwrap() = EstimatorStatus::Error("Unknown error".to_string());
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

                    let priority_fee_sompi = self.context.priority_fees_sompi;

                    // let priority_fees_sompi = if self.context.enable_priority_fees {
                    //     self.context.priority_fees_sompi
                    // } else { 0 };
                    // let fee_rate = self.context.priority_fee_rate + 1.0;

                    // let fee_rate = {
 
                    // };
                    
                    // ---


                    let wallet_secret = Secret::from(self.context.wallet_secret.clone());
                    let payment_secret = account.requires_bip39_passphrase(core).then_some(Secret::from(self.context.payment_secret.clone()));

                    match self.context.transaction_kind.unwrap() {
                        TransactionKind::Send => {

                            let address = Address::try_from(self.context.destination_address_string.as_str()).expect("invalid address");
                            let account_id = account.id();
                            let send_amount_sompi = self.context.send_amount_sompi;
                            let payment_output = PaymentOutput {
                                address,
                                amount: send_amount_sompi,
                            };
        
                            spawn_with_result(&send_result, async move {

                                let fee_rate = calculate_fee_rate(network_type, account_id, send_amount_sompi, priority_fee_sompi).await;

                                let request = AccountsSendRequest {
                                    account_id,
                                    destination: payment_output.into(),
                                    wallet_secret,
                                    payment_secret,
                                    fee_rate: Some(fee_rate),
                                    // priority_fee_sompi: Fees::SenderPays(priority_fees_sompi),
                                    priority_fee_sompi: Fees::SenderPays(0),
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
                                let fee_rate = calculate_fee_rate(network_type, source_account_id, transfer_amount_sompi, priority_fee_sompi).await;

                                let request = AccountsTransferRequest {
                                    source_account_id,
                                    destination_account_id,
                                    wallet_secret,
                                    payment_secret,
                                    fee_rate: Some(fee_rate),
                                    priority_fee_sompi: Some(Fees::SenderPays(0)),
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

async fn calculate_fee_rate(network_type : NetworkType, account_id : AccountId, send_amount_sompi : u64, priority_fee_sompi : u64) -> f64 {

    let address = match network_type {
        NetworkType::Testnet => Address::try_from("kaspatest:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhqrxplya").unwrap(),
        NetworkType::Mainnet => Address::try_from("kaspa:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqkx9awp4e").unwrap(),
        _ => panic!("Unsupported network"),
    };

    // let account_id = account_id;
    let payment_output = PaymentOutput {
        address,
        amount: send_amount_sompi,
    };


    let base_request = AccountsEstimateRequest {
        account_id,
        destination: payment_output.clone().into(),
        // priority_fee_sompi: Fees::SenderPays(priority_fees_sompi),
        priority_fee_sompi: Fees::SenderPays(0),
        fee_rate: Some(0.0),
        payload: None,
    };

    let base_result = runtime().wallet().accounts_estimate_call(base_request).await;

    let base_mass = base_result.as_ref().map(|r| r.generator_summary.aggregate_mass).unwrap_or_default();

    if base_mass == 0 {
        1.0
    } else {
        (priority_fee_sompi as f64 / base_mass as f64) + 1.0
    }
}