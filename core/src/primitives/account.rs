use kaspa_wallet_core::account::{
    BIP32_ACCOUNT_KIND, KEYPAIR_ACCOUNT_KIND, LEGACY_ACCOUNT_KIND, MULTISIG_ACCOUNT_KIND,
};

use crate::imports::*;

pub struct AccountContext {
    qr: load::Bytes,
    receive_address: Address,
    uri: String,
}

impl AccountContext {
    pub fn new(descriptor: &AccountDescriptor) -> Option<Arc<Self>> {
        if let Some(receive_address) = descriptor.receive_address() {
            let address_string = receive_address.to_string();
            let qr = render_qrcode(&address_string, 128, 128);
            let uri = format!("bytes://{}-{}.svg", address_string, theme_color().name);
            Some(Arc::new(Self {
                qr: qr.as_bytes().to_vec().into(),
                receive_address: receive_address.clone(),
                uri,
            }))
        } else {
            None
        }
    }

    pub fn address(&self) -> &Address {
        &self.receive_address
    }

    pub fn qr(&self) -> load::Bytes {
        self.qr.clone()
    }

    pub fn uri(&self) -> String {
        self.uri.clone()
    }
}

struct Inner {
    id: AccountId,
    account_kind: AccountKind,
    balance: Mutex<Option<Balance>>,
    descriptor: Mutex<AccountDescriptor>,
    context: Mutex<Option<Arc<AccountContext>>>,
    transactions: Mutex<TransactionCollection>,
    total_transaction_count: AtomicU64,
    transaction_start: AtomicU64,
    is_loading: AtomicBool,
    network: Mutex<Network>,
}

impl Inner {
    fn new(network: Network, descriptor: AccountDescriptor) -> Self {
        let context = AccountContext::new(&descriptor);
        Self {
            id: *descriptor.account_id(),
            account_kind: *descriptor.account_kind(),
            balance: Mutex::new(None),
            descriptor: Mutex::new(descriptor),
            context: Mutex::new(context),
            transactions: Mutex::new(TransactionCollection::default()),
            total_transaction_count: AtomicU64::new(0),
            transaction_start: AtomicU64::new(0),
            is_loading: AtomicBool::new(true),
            network: Mutex::new(network),
        }
    }
}

#[derive(Clone)]
pub struct Account {
    inner: Arc<Inner>,
}

// impl From<AccountDescriptor> for Account {
//     fn from(descriptor: AccountDescriptor) -> Self {
//         Self {
//             inner: Arc::new(Inner::new(network, descriptor)),
//         }
//     }
// }

impl Account {
    pub fn from(network: Network, descriptor: AccountDescriptor) -> Self {
        Self {
            inner: Arc::new(Inner::new(network, descriptor)),
        }
    }

    pub fn descriptor(&self) -> MutexGuard<'_, AccountDescriptor> {
        self.inner.descriptor.lock().unwrap()
    }

    pub fn receive_address(&self) -> Address {
        self.inner
            .context
            .lock()
            .unwrap()
            .as_ref()
            .map(|context| context.address().clone())
            .unwrap()
    }

    pub fn transactions(&self) -> MutexGuard<'_, TransactionCollection> {
        self.inner.transactions.lock().unwrap()
    }

    pub fn id(&self) -> AccountId {
        self.inner.id
    }

    pub fn name_or_id(&self) -> String {
        self.descriptor().name_or_id()
    }

    pub fn requires_bip39_passphrase(&self, core: &Core) -> bool {
        let descriptor = self.descriptor();
        let prv_key_data_ids = descriptor.prv_key_data_ids();
        core.prv_key_data_map()
            .as_ref()
            .map(|prv_key_data_map| {
                prv_key_data_ids.into_iter().any(|prv_key_data_id| {
                    prv_key_data_map
                        .get(&prv_key_data_id)
                        .map(|prv_key_data_info| prv_key_data_info.requires_bip39_passphrase())
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    }

    pub fn account_kind(&self) -> &AccountKind {
        &self.inner.account_kind
    }

    pub fn balance(&self) -> Option<Balance> {
        self.inner.balance.lock().unwrap().clone()
    }

    pub fn update_theme(&self) {
        let descriptor = self.descriptor().clone();
        *self.inner.context.lock().unwrap() = AccountContext::new(&descriptor);
    }

    pub fn context(&self) -> Option<Arc<AccountContext>> {
        self.inner.context.lock().unwrap().clone()
    }

    pub fn update(&self, descriptor: AccountDescriptor) {
        *self.inner.context.lock().unwrap() = AccountContext::new(&descriptor);
        *self.inner.descriptor.lock().unwrap() = descriptor;
    }

    pub fn update_balance(&self, balance: Option<Balance>) -> Result<()> {
        *self.inner.balance.lock().unwrap() = balance;

        Ok(())
    }

    pub fn update_network(&self, network: Network) {
        *self.inner.network.lock().unwrap() = network;
    }

    pub fn network(&self) -> Network {
        *self.inner.network.lock().unwrap()
    }

    pub fn set_loading(&self, is_loading: bool) {
        self.inner.is_loading.store(is_loading, Ordering::SeqCst);
    }

    pub fn is_loading(&self) -> bool {
        self.inner.is_loading.load(Ordering::SeqCst)
    }

    pub fn set_transaction_count(&self, count: u64) {
        self.inner
            .total_transaction_count
            .store(count, Ordering::SeqCst);
    }

    pub fn transaction_count(&self) -> u64 {
        self.inner.total_transaction_count.load(Ordering::SeqCst)
    }

    pub fn set_transaction_start(&self, start: u64) {
        self.inner.transaction_start.store(start, Ordering::SeqCst);
    }

    pub fn transaction_start(&self) -> u64 {
        self.inner.transaction_start.load(Ordering::SeqCst)
    }

    pub fn load_transactions(
        &self,
        mut transactions: Vec<Arc<TransactionRecord>>,
        total: u64,
    ) -> Result<()> {
        self.transactions().clear();

        transactions.sort_by(|a, b| {
            if let Some(b_ts) = b.unixtime_msec {
                if let Some(a_ts) = a.unixtime_msec {
                    return b_ts.cmp(&a_ts);
                }
            }
            b.block_daa_score.cmp(&a.block_daa_score)
        });

        self.set_transaction_count(total);
        self.transactions()
            .load(transactions.into_iter().map(|t| t.into()));

        Ok(())
    }

    pub fn clear_transactions(&self) {
        self.set_transaction_count(0);
        self.transactions().clear();
    }
}

impl IdT for Account {
    type Id = AccountId;

    fn id(&self) -> &Self::Id {
        &self.inner.id
    }
}

impl std::fmt::Debug for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.id(), f)
    }
}

pub type AccountCollection = Collection<AccountId, Account>;

pub trait DescribeAccount {
    fn describe(&self) -> (&'static str, &'static str);
}

impl DescribeAccount for AccountKind {
    fn describe(&self) -> (&'static str, &'static str) {
        match self.as_ref() {
            LEGACY_ACCOUNT_KIND => ("Legacy Account", "KDX, PWA (kaspanet.io)"),
            BIP32_ACCOUNT_KIND => ("Kaspa Core BIP32", "kaspawallet, kaspium"),
            MULTISIG_ACCOUNT_KIND => ("Multi-Signature", ""),
            KEYPAIR_ACCOUNT_KIND => ("Keypair", "secp256k1"),
            _ => ("", ""),
        }
    }
}

pub trait AccountSelectorButtonExtension {
    fn account_selector_button(
        &mut self,
        account: &Account,
        network_type: &NetworkType,
        selected: bool,
        balance_padding: bool,
    ) -> Response;
}

impl AccountSelectorButtonExtension for Ui {
    fn account_selector_button(
        &mut self,
        account: &Account,
        network_type: &NetworkType,
        selected: bool,
        balance_padding: bool,
    ) -> Response {
        let account_name = account.name_or_id();

        let icon = if selected {
            Composite::icon(egui_phosphor::thin::QUEUE)
        } else {
            Composite::icon(egui_phosphor::thin::LIST_DASHES)
        };

        let large_button_size = theme_style().large_button_size() + vec2(32., 0.);
        if let Some(balance) = account.balance() {
            let color = self.style().visuals.text_color();
            self.add_sized(
                large_button_size,
                CompositeButton::image_and_text(
                    icon,
                    RichText::new(account_name).size(14.),
                    s2kws_layout_job(
                        balance_padding,
                        balance.mature,
                        network_type,
                        color,
                        FontId::monospace(16.),
                    ),
                ),
            )
        } else {
            self.add_sized(
                large_button_size,
                CompositeButton::image_and_text(
                    icon,
                    RichText::new(account_name).size(14.),
                    RichText::new("N/A").font(FontId::monospace(16.)),
                ),
            )
        }
    }
}
