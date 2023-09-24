use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::LitByte;
use rand::Rng;
    
    pub fn mask(_input: TokenStream) -> TokenStream {
        
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill(&mut bytes);
    let mask = bytes.iter().map(|b| LitByte::new(*b, Span::call_site()));
    quote! {
        [#(#mask),*]
    }.into()
}
