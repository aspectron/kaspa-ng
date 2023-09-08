use crate::interop::Interop;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use crate::events::Events;

pub struct Signals {
    interop : Interop,
    iterations: AtomicU64,
}

impl Signals {
    pub fn bind(interop: &Interop) {
        let signals = Arc::new(Signals { interop : interop.clone(), iterations: AtomicU64::new(0) });
        
        ctrlc::set_handler(move || {
            let v = signals.iterations.fetch_add(1, Ordering::SeqCst);
            if v > 1 {
                println!("^SIGTERM - halting");
                std::process::exit(1);
            }
    
            println!("^SIGTERM - shutting down...");
            signals.interop.try_send(Events::Exit).unwrap_or_else(|e| {
                println!("Error sending exit event: {:?}", e);
            });
        })
        .expect("Error setting signal handler");

    }
}