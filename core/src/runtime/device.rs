use crate::imports::*;

#[derive(Default)]
struct Inner {
    pub is_portrait: bool,
    pub is_mobile: bool,
}

#[derive(Default)]
pub struct Device {
    inner: Arc<Mutex<Inner>>,
}

impl Device {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                is_portrait: false,
                is_mobile: false,
            })),
        }
    }

    fn inner(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap()
    }

    pub fn is_portrait(&self) -> bool {
        self.inner().is_portrait
    }

    pub fn is_mobile(&self) -> bool {
        self.inner().is_mobile
    }

    pub fn toggle_portrait(&self) {
        let mut inner = self.inner();
        inner.is_portrait = !inner.is_portrait;
    }

    pub fn toggle_mobile(&self) {
        let mut inner = self.inner();
        inner.is_mobile = !inner.is_mobile;
    }

    pub fn is_singular_layout(&self) -> bool {
        let inner = self.inner();
        inner.is_mobile || inner.is_portrait
    }
}
