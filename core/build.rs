use std::error::Error;
use vergen_gitcl::{Build, Cargo, Emitter, Gitcl, Rustc};

// https://docs.rs/vergen-gitcl/latest/vergen_gitcl/
fn main() -> Result<(), Box<dyn Error>> {
    let build = Build::all_build();
    let cargo = Cargo::all_cargo();
    let gitcl = Gitcl::all_git();
    let rustc = Rustc::all_rustc();
    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&gitcl)?
        .add_instructions(&rustc)?
        .emit()?;
    Ok(())
}
