use crate::imports::*;
use crate::result::Result;
use kaspa_wallet_core::events as kaspa;

#[derive(Debug)]
pub enum Events {
    Wallet(kaspa::Events),
    TryUnlock(Secret),
    UnlockSuccess,
    UnlockFailure { message : String },
    Lock,
    Send,
    Deposit,
    Overview,
    Transactions,
    Accounts,
    Settings,
    Exit,
}

impl Events {
    pub fn info(&self) -> String {
        match self {
            Events::Wallet(_) => "Wallet".to_string(),
            Events::TryUnlock(_) => "TryUnlock".to_string(),
            Events::UnlockSuccess {..} => "UnlockSuccess".to_string(),
            Events::UnlockFailure {..} => "UnlockFailure".to_string(),
            Events::Lock {..} => "Lock".to_string(),
            Events::Send {..} => "Send".to_string(),
            Events::Deposit {..} => "Deposit".to_string(),
            Events::Overview {..} => "Overview".to_string(),
            Events::Transactions {..} => "Transactions".to_string(),
            Events::Accounts {..} => "Accounts".to_string(),
            Events::Settings {..} => "Settings".to_string(),
            Events::Exit {..} => "Exit".to_string(),
        }
    }

    pub fn handle(&self, wallet : &mut Wallet) -> Result<()> {
        match self {
            Events::TryUnlock(_secret) => {
                let mut unlock = wallet.get_mut::<section::Unlock>();
                unlock.message = Some("Error unlocking wallet...".to_string());
                unlock.lock();
            },
            Events::UnlockSuccess => {
                wallet.select::<section::Overview>();
            },
            Events::UnlockFailure {..} => {

            },
            Events::Wallet(event) => {
                match event {
                    kaspa::Events::UtxoProcStart => {},
                    kaspa::Events::UtxoProcStop => {},
                    kaspa::Events::UtxoProcError(_err) => {
                        // terrorln!(this,"{err}");
                    },
                    #[allow(unused_variables)]
                    kaspa::Events::Connect{ url, network_id } => {
                        // log_info!("Connected to {url}");
                    },
                    #[allow(unused_variables)]
                    kaspa::Events::Disconnect{ url, network_id } => {
                        // tprintln!(this, "Disconnected from {url}");
                        // this.term().refresh_prompt();
                    },
                    kaspa::Events::UtxoIndexNotEnabled => {
                        // tprintln!(this, "Error: Kaspa node UTXO index is not enabled...")
                    },
                    kaspa::Events::SyncState(_state) => {
                        // this.sync_state.lock().unwrap().replace(state);
                        // this.term().refresh_prompt();
                    }
                    kaspa::Events::ServerStatus {
                        is_synced:_,
                        server_version:_,
                        url:_,
                        ..
                    } => {

                        // tprintln!(this, "Connected to Kaspa node version {server_version} at {url}");

                        // let is_open = this.wallet.is_open();

                        // if !is_synced {
                        //     if is_open {
                        //         terrorln!(this, "Unable to update the wallet state - Kaspa node is currently syncing with the network...");

                        //     } else {
                        //         terrorln!(this, "Kaspa node is currently syncing with the network, please wait for the sync to complete...");
                        //     }
                        // }

                        // this.term().refresh_prompt();

                    },
                    kaspa::Events::WalletHint {
                        hint:_
                    } => {

                        // if let Some(hint) = hint {
                        //     tprintln!(this, "\nYour wallet hint is: {hint}\n");
                        // }

                    },
                    kaspa::Events::WalletOpen |
                    kaspa::Events::WalletReload => {

                        // load all accounts
                        // this.wallet().activate_all_stored_accounts().await.unwrap_or_else(|err|terrorln!(this, "{err}"));

                        // // list all accounts
                        // this.list().await.unwrap_or_else(|err|terrorln!(this, "{err}"));

                        // // load default account if only one account exists
                        // this.wallet().autoselect_default_account_if_single().await.ok();
                        // this.term().refresh_prompt();

                    },
                    kaspa::Events::WalletClose => {
                        // this.term().refresh_prompt();
                    },
                    kaspa::Events::DAAScoreChange(_daa) => {
                        // if this.is_mutted() && this.flags.get(Track::Daa) {
                        //     tprintln!(this, "{NOTIFY} DAA: {daa}");
                        // }
                    },
                    kaspa::Events::Reorg {
                        record:_
                    } => {
                        // if !this.is_mutted() || (this.is_mutted() && this.flags.get(Track::Pending)) {
                        //     let include_utxos = this.flags.get(Track::Utxo);
                        //     let tx = record.format_with_state(&this.wallet,Some("reorg"),include_utxos).await;
                        //     tx.iter().for_each(|line|tprintln!(this,"{NOTIFY} {line}"));
                        // }
                    },
                    kaspa::Events::External {
                        record:_
                    } => {
                        // if !this.is_mutted() || (this.is_mutted() && this.flags.get(Track::Tx)) {
                        //     let include_utxos = this.flags.get(Track::Utxo);
                        //     let tx = record.format_with_state(&this.wallet,Some("external"),include_utxos).await;
                        //     tx.iter().for_each(|line|tprintln!(this,"{NOTIFY} {line}"));
                        // }
                    },
                    kaspa::Events::Pending {
                        record:_daa, is_outgoing : _
                    } => {
                        // if !this.is_mutted() || (this.is_mutted() && this.flags.get(Track::Pending)) {
                        //     let include_utxos = this.flags.get(Track::Utxo);
                        //     let tx = record.format_with_state(&this.wallet,Some("pending"),include_utxos).await;
                        //     tx.iter().for_each(|line|tprintln!(this,"{NOTIFY} {line}"));
                        // }
                    },
                    kaspa::Events::Maturity {
                        record:_, is_outgoing : _
                    } => {
                        // if !this.is_mutted() || (this.is_mutted() && this.flags.get(Track::Tx)) {
                        //     let include_utxos = this.flags.get(Track::Utxo);
                        //     let tx = record.format_with_state(&this.wallet,Some("confirmed"),include_utxos).await;
                        //     tx.iter().for_each(|line|tprintln!(this,"{NOTIFY} {line}"));
                        // }
                    },
                    kaspa::Events::Outgoing {
                        record:_
                    } => {
                        // if !this.is_mutted() || (this.is_mutted() && this.flags.get(Track::Tx)) {
                        //     let include_utxos = this.flags.get(Track::Utxo);
                        //     let tx = record.format_with_state(&this.wallet,Some("confirmed"),include_utxos).await;
                        //     tx.iter().for_each(|line|tprintln!(this,"{NOTIFY} {line}"));
                        // }
                    },
                    kaspa::Events::Balance {
                        balance:_,
                        id:_,
                        mature_utxo_size:_,
                        pending_utxo_size:_,
                    } => {

                        // if !this.is_mutted() || (this.is_mutted() && this.flags.get(Track::Balance)) {
                        //     let network_id = this.wallet.network_id().expect("missing network type");
                        //     let network_type = NetworkType::from(network_id);
                        //     let balance = BalanceStrings::from((&balance,&network_type, None));
                        //     let id = id.short();

                        //     let pending_utxo_info = if pending_utxo_size > 0 {
                        //         format!("({pending_utxo_size} pending)")
                        //     } else { "".to_string() };
                        //     let utxo_info = style(format!("{} UTXOs {pending_utxo_info}", mature_utxo_size.separated_string())).dim();

                        //     tprintln!(this, "{NOTIFY} {} {id}: {balance}   {utxo_info}",style("balance".pad_to_width(8)).blue());
                        // }

                        // this.term().refresh_prompt();
                    }
                }
            }
            _ => unimplemented!()
        }

        Ok(())        
    }
}