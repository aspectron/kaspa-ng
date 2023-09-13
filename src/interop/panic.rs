use std::{process, panic};

pub fn init_panic_handler() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        default_hook(panic_info);
        println!("Halting...");
        process::exit(1);
    }));
}
