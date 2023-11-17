use crate::events::Events;
use crate::interop::Interop;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct Signals {
    interop: Interop,
    iterations: AtomicU64,
}

impl Signals {
    pub fn bind(interop: &Interop) {
        let signals = Arc::new(Signals {
            interop: interop.clone(),
            iterations: AtomicU64::new(0),
        });

        ctrlc::set_handler(move || {
            let v = signals.iterations.fetch_add(1, Ordering::SeqCst);

            match v {
                0 => {
                    // post a graceful exit event to the main event loop
                    println!("^SIGTERM - shutting down...");
                    signals.interop.try_send(Events::Exit).unwrap_or_else(|e| {
                        println!("Error sending exit event: {:?}", e);
                    });
                }
                1 => {
                    // start interop abort sequence
                    // (attempt to gracefully shutdown kaspad if running)
                    // this will execute process::exit(1) after 5 seconds
                    println!("^SIGTERM - aborting...");
                    crate::interop::abort();
                }
                _ => {
                    // exit the process immediately
                    println!("^SIGTERM - halting");
                    std::process::exit(1);
                }
            }
        })
        .expect("Error setting signal handler");
    }
}
