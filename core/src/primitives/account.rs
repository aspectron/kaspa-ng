use crate::imports::*;

pub struct Context {
    // qr: Arc<RetainedImage>,
    // qr: Arc<String>,
    qr: load::Bytes,
    receive_address: String,
}

impl Context {
    pub fn new(descriptor: &AccountDescriptor) -> Option<Arc<Self>> {
        // if account.wallet().network_id().is_ok() {
        if let Some(receive_address) = descriptor.receive_address().map(String::from) {
            // let receive_address = account.receive_address().unwrap().to_string();
            let qr = render_qrcode(&receive_address, 128, 128);
            Some(Arc::new(Self {
                qr: qr.as_bytes().to_vec().into(),
                receive_address,
            }))
        } else {
            None
        }
    }

    pub fn address(&self) -> &str {
        self.receive_address.as_str()
    }

    pub fn qr(&self) -> load::Bytes {
        self.qr.clone()
    }
}

struct Inner {
    // runtime: Arc<dyn runtime::Account>,
    id: AccountId,
    balance: Mutex<Option<Balance>>,
    utxo_sizes: Mutex<Option<(usize, usize)>>,
    descriptor: Mutex<AccountDescriptor>,
    context: Mutex<Option<Arc<Context>>>,
    transactions: Mutex<TransactionCollection>,
    total_transaction_count: AtomicU64,
    is_loading: AtomicBool,
}

impl Inner {
    fn new(descriptor: AccountDescriptor) -> Self {
        let context = Context::new(&descriptor);
        Self {
            id: *descriptor.account_id(),
            balance: Mutex::new(None),
            utxo_sizes: Mutex::new(None),
            descriptor: Mutex::new(descriptor),
            context: Mutex::new(context),
            transactions: Mutex::new(TransactionCollection::default()),
            total_transaction_count: AtomicU64::new(0),
            is_loading: AtomicBool::new(true),
        }
    }

    // fn descriptor(&self) -> MutexGuard<'_, AccountDescriptor> {
    //     self.descriptor.lock().unwrap()
    // }

    // fn transactions(&self) -> MutexGuard<'_, AccountDescriptor> {
    //     self.descriptor.lock().unwrap()
    // }
}

#[derive(Clone)]
pub struct Account {
    inner: Arc<Inner>,
}

impl From<AccountDescriptor> for Account {
    fn from(descriptor: AccountDescriptor) -> Self {
        Self {
            inner: Arc::new(Inner::new(descriptor)),
        }
    }
}

impl Account {
    pub fn descriptor(&self) -> MutexGuard<'_, AccountDescriptor> {
        self.inner.descriptor.lock().unwrap()
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

    pub fn balance(&self) -> Option<Balance> {
        self.inner.balance.lock().unwrap().clone()
    }

    pub fn utxo_sizes(&self) -> Option<(usize, usize)> {
        *self.inner.utxo_sizes.lock().unwrap()
    }

    // pub fn address(&self) -> Result<String> {
    //     self.inner.context.lock().unwrap().receive_address
    //     Ok(self.inner.runtime.receive_address()?.into())
    // }

    pub fn context(&self) -> Option<Arc<Context>> {
        self.inner.context.lock().unwrap().clone()
    }

    pub fn update(&self, descriptor: AccountDescriptor) -> Result<()> {
        *self.inner.context.lock().unwrap() = Context::new(&descriptor);
        *self.inner.descriptor.lock().unwrap() = descriptor;

        Ok(())
    }

    pub fn update_balance(
        &self,
        balance: Option<Balance>,
        mature_utxo_size: usize,
        pending_utxo_size: usize,
    ) -> Result<()> {
        *self.inner.balance.lock().unwrap() = balance;
        *self.inner.utxo_sizes.lock().unwrap() = Some((mature_utxo_size, pending_utxo_size));

        Ok(())
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

    pub fn load_transactions(
        &self,
        transactions: Vec<Arc<TransactionRecord>>,
        total: u64,
    ) -> Result<()> {
        self.set_transaction_count(total);
        self.transactions()
            .load(transactions.into_iter().map(|t| t.into()));

        Ok(())
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

// #[derive(Default)]
// pub struct AccountCollection {
//     list : Vec<Account>,
//     map : HashMap<AccountId, Account>,
// }

// impl AccountCollection {
//     pub fn new() -> Self {
//         Self {
//             list : Vec::new(),
//             map : HashMap::new(),
//         }
//     }

//     pub fn len(&self) -> usize {
//         self.list.len()
//     }

//     pub fn is_empty(&self) -> bool {
//         self.list.is_empty()
//     }

//     pub fn insert(&mut self, account : Account) {
//         self.list.push(account.clone());
//         self.map.insert(account.id(), account);
//     }

//     pub fn first(&self) -> Option<&Account> {
//         self.list.first()
//     }

//     pub fn get(&self, id : &AccountId) -> Option<&Account> {
//         self.map.get(id)
//     }

//     pub fn list(&self) -> &Vec<Account> {
//         &self.list
//     }

//     pub fn list_mut(&mut self) -> &mut Vec<Account> {
//         &mut self.list
//     }

//     pub fn remove(&mut self, id : &AccountId) -> Option<Account> {
//         if let Some(account) = self.map.remove(id) {
//             self.list.retain(|a| a.id() != *id);
//             Some(account)
//         } else {
//             None
//         }
//     }

//     pub fn iter(&self) -> impl Iterator<Item = &Account> {
//         self.list.iter()
//     }

//     pub fn clear(&mut self) {
//         self.list.clear();
//         self.map.clear();
//     }

// }

// impl From<Vec<Account>> for AccountCollection {
//     fn from(list : Vec<Account>) -> Self {
//         Self {
//             map :   list
//                     .clone()
//                     .into_iter()
//                     .map(|account| (account.id(), account)).collect::<HashMap::<_,_>>(),
//             list,

//         }
//     }
// }
