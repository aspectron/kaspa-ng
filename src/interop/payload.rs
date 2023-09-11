use std::sync::{atomic::{AtomicBool,Ordering}, OnceLock};

use crate::imports::*;

// enum State {
//     // None,
//     Pending,
//     Ready,
// }

struct Inner<T> 
where T : Send
{
    id : String,
    // state : Mutex<State>,
    payload : Mutex<Option<T>>,
    pending : AtomicBool,
}

// #[derive(Clone)]
pub struct Payload<T = ()> 
where T: Send
{
    inner : Arc<Inner<T>>,
}

impl<T> Clone for Payload<T> 
where T: Send
{
    fn clone(&self) -> Payload<T> {
        Payload { inner: self.inner.clone() }
    }
}

impl<T> Payload<T> 
where T: Send + 'static
{

    pub fn new<S: std::fmt::Display>(id : S) -> Self {

        let id = id.to_string();
        // let mut registry = REGISTRY.lock().unwrap();
        // if registry.is_none() {
        //     *registry = Some(HashMap::new());
        // }
        // let registry = registry.as_mut().unwrap();
        let mut registry = registry().lock().unwrap();

        if let Some(payload) = registry.get(&id) {
            if let Some(p)  = payload.downcast_ref::<Payload<T>>() {
                let inner = p.inner.clone();
                Self { inner }
            } else {
                panic!("Unable to downcast Payload `{id}`");
            }
        } else {

            let inner = Arc::new(Inner {
                id : id.clone(),
                payload : Mutex::new(None),
                pending : AtomicBool::new(false)
            });

            registry.insert(id,Box::new(Payload { inner : inner.clone() }));
            Self { inner }
        }

    }

    pub fn store(&self, data : T) {
        let mut payload = self.inner.payload.lock().unwrap();
        *payload = Some(data);
    }

    pub fn is_pending(&self) -> bool {
        self.inner.pending.load(Ordering::SeqCst)
    }

    // pub fn is_ready(&self) -> bool {
    //     let payload = self.inner.payload.lock().unwrap();
    //     matches!(payload.0,Some(State::Ready))
    // }

    pub fn is_some(&self) -> bool {
        let payload = self.inner.payload.lock().unwrap();
        payload.is_some()
    }

    pub fn take(&self) -> Option<T> {
        let mut payload = self.inner.payload.lock().unwrap();
        payload.take()
    }
}

// static mut REGISTRY : Option<Mutex<HashMap<String,Box<dyn Any>>>> = None;

// static REGISTRY : Arc<Mutex<Option<HashMap<>>>> = Arc::new(Mutex::new(None));

// fn registry() -> Arc<Mutex<Option<HashMap<String,Box<dyn Any + Send>>>>> {


// }

fn registry() -> &'static Mutex<HashMap<String,Box<dyn Any + Sync + Send>>> {
    static MEM: OnceLock<Mutex<HashMap<String,Box<dyn Any + Sync + Send>>>> = OnceLock::new();
    MEM.get_or_init(|| {
        Mutex::new(HashMap::new())
    })
}