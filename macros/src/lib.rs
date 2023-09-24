use proc_macro::TokenStream;
mod mask;
mod register;

#[proc_macro]
pub fn register_modules(input: TokenStream) -> TokenStream {
    register::register_modules(input)
}

#[proc_macro]
pub fn mask(input: TokenStream) -> TokenStream {
    mask::mask(input)
}
