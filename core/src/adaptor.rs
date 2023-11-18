// Chrome Extension Wallet Adaptor

use crate::imports::*;
use wasm_bindgen::prelude::*;

struct Inner {
    #[allow(dead_code)]
    runtime: Runtime,
}

impl Inner {
    pub fn new(runtime: Runtime) -> Self {
        Self { runtime }
    }
}

#[wasm_bindgen]
pub struct Adaptor {
    #[allow(dead_code)]
    inner: Arc<Inner>,
}

impl Adaptor {
    pub fn new(runtime: Runtime) -> Self {
        Self {
            inner: Arc::new(Inner::new(runtime.clone())),
        }
    }
}

#[wasm_bindgen]
impl Adaptor {
    #[wasm_bindgen(js_name = "sendTransaction")]
    pub fn send_transaction(&self) -> Result<()> {
        Ok(())
    }
}

// static mut ADAPTOR : Option<Adaptor> = None;
