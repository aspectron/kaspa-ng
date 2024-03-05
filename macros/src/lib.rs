use proc_macro::TokenStream;
mod crnd;
mod register;

#[proc_macro]
pub fn register_modules(input: TokenStream) -> TokenStream {
    register::register_modules(input)
}

#[proc_macro]
pub fn crnd(input: TokenStream) -> TokenStream {
    crnd::crnd(input)
}
