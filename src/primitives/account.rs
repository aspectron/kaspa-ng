use crate::imports::*;

pub struct Context {
    qr: Arc<RetainedImage>,
    receive_address: String,
}

impl Context {
    pub fn new(account: &Arc<dyn runtime::Account>) -> Option<Arc<Self>> {
        if account.wallet().network_id().is_ok() {
            let receive_address = account.receive_address().unwrap().to_string();
            let qr = Arc::new(render_qrcode(&receive_address, 200, 200));
            Some(Arc::new(Self {
                qr,
                receive_address,
            }))
        } else {
            None
        }
    }

    pub fn address(&self) -> &str {
        self.receive_address.as_str()
    }

    pub fn qr(&self) -> &RetainedImage {
        &self.qr
    }
}

struct Inner {
    runtime: Arc<dyn runtime::Account>,
    context: Mutex<Option<Arc<Context>>>,
}

impl Inner {
    fn new(runtime: Arc<dyn runtime::Account>) -> Self {
        let context = Context::new(&runtime);
        Self {
            runtime,
            context: Mutex::new(context),
        }
    }
}

#[derive(Clone)]
pub struct Account {
    inner: Arc<Inner>,
}

impl From<Arc<dyn runtime::Account>> for Account {
    fn from(runtime: Arc<dyn runtime::Account>) -> Self {
        Self {
            inner: Arc::new(Inner::new(runtime)),
        }
    }
}

impl Account {
    pub fn runtime(&self) -> Arc<dyn runtime::Account> {
        self.inner.runtime.clone()
    }

    pub fn name_or_id(&self) -> String {
        self.inner.runtime.name_or_id()
    }

    pub fn balance(&self) -> Option<Balance> {
        self.inner.runtime.balance()
    }

    // pub fn address(&self) -> Result<String> {
    //     self.inner.context.lock().unwrap().receive_address
    //     Ok(self.inner.runtime.receive_address()?.into())
    // }

    pub fn context(&self) -> Option<Arc<Context>> {
        self.inner.context.lock().unwrap().clone()
    }

    pub fn update(&self) -> Result<()> {
        let context = Context::new(&self.inner.runtime);
        *self.inner.context.lock().unwrap() = context;

        Ok(())
    }
}
