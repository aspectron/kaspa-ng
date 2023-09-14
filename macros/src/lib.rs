use proc_macro::TokenStream;
mod register;

#[proc_macro]
pub fn register_modules(input: TokenStream) -> TokenStream {
    register::register_modules(input)
}
