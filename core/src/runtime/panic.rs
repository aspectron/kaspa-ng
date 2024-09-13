use std::backtrace::Backtrace;
use std::fs;
use std::panic;
use workflow_core::dirs;

pub fn init_graceful_panic_handler() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let backtrace = Backtrace::capture();
        // println!("panic! \n{:#?}\n{:#?}", panic_info, backtrace);
        let _ = fs::write(
            dirs::home_dir().unwrap_or_default().join("kaspa-ng.log"),
            format!("{:#?}\n{:#?}", panic_info, backtrace),
        );
        println!("An unexpected condition (panic) has occurred. Additional information has been written to `~/kaspa-ng.log`");
        default_hook(panic_info);
        crate::runtime::abort();
    }));
}

pub fn init_ungraceful_panic_handler() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let backtrace = Backtrace::capture();
        let _ = fs::write(
            dirs::home_dir()
                .unwrap_or_default()
                .join("kaspa-ng-service.log"),
            format!("{:#?}\n{:#?}", panic_info, backtrace),
        );
        default_hook(panic_info);
        println!("An unexpected condition (panic) has occurred. Additional information has been written to `~/kaspa-ng-service.log`");
        println!("Exiting...");
        std::process::exit(1);
    }));
}
