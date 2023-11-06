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
            let qr = render_qrcode(&receive_address, 200, 200);
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
    descriptor: Mutex<AccountDescriptor>,
    context: Mutex<Option<Arc<Context>>>,
}

impl Inner {
    fn new(descriptor: AccountDescriptor) -> Self {
        let context = Context::new(&descriptor);
        Self {
            id: *descriptor.account_id(),
            balance: Mutex::new(None),
            descriptor: Mutex::new(descriptor),
            context: Mutex::new(context),
        }
    }

    fn descriptor(&self) -> MutexGuard<'_, AccountDescriptor> {
        self.descriptor.lock().unwrap()
    }
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
    // pub fn runtime(&self) -> Arc<dyn runtime::Account> {
    //     self.inner.runtime.clone()
    // }

    pub fn descriptor(&self) -> MutexGuard<'_, AccountDescriptor> {
        self.inner.descriptor()
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
}
