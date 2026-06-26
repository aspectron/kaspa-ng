use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use rand::RngExt;
use syn::LitByte;

pub fn crnd(_input: TokenStream) -> TokenStream {
    let mut bytes = [0u8; 32];
    rand::rng().fill(&mut bytes);
    let crnd = bytes.iter().map(|b| LitByte::new(*b, Span::call_site()));
    quote! {
        [#(#crnd),*]
    }
    .into()
}
