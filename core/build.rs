use std::error::Error;
use vergen::EmitBuilder;
extern crate winres;

// https://docs.rs/vergen/latest/vergen/struct.EmitBuilder.html#method.emit
fn main() -> Result<(), Box<dyn Error>> {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("resources/icons/exeicon.ico");
        res.compile().unwrap();
    }

    EmitBuilder::builder()
        .all_build()
        .all_cargo()
        .all_git()
        .all_rustc()
        .emit()?;
    Ok(())
}
