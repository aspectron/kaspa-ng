use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use quote::ToTokens;
use std::collections::HashSet;
use std::convert::Into;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Error, Expr, ExprArray, Result, Token,
};

#[derive(Debug)]
struct Modules {
    function_name: Expr,
    modules: ExprArray,
}

impl Parse for Modules {
    fn parse(input: ParseStream) -> Result<Self> {
        let parsed = Punctuated::<Expr, Token![,]>::parse_terminated(input);
        if parsed.is_err() {
            return Err(Error::new(
                Span::call_site(),
                "usage: declare_modules!(<array of module names>)".to_string(),
            ));
        }

        let parsed = parsed.unwrap();
        if parsed.len() != 2 {
            return Err(Error::new_spanned(
                parsed,
                "usage: declare_modules!(<function name>, <array of module names>)".to_string(),
            ));
        }

        let mut iter = parsed.iter();
        let function_name = iter.next().unwrap().clone();
        let modules = get_modules(iter.next().unwrap().clone())?;

        Ok(Modules {
            function_name,
            modules,
        })
    }
}

fn get_modules(modules: Expr) -> Result<ExprArray> {
    let modules = match modules {
        Expr::Array(array) => array,
        _ => {
            return Err(Error::new_spanned(
                modules,
                "the argument must be an array of enum variants".to_string(),
            ));
        }
    };

    for ph in modules.elems.iter() {
        match ph {
            Expr::Path(_exp_path) => {}
            _ => {
                return Err(Error::new_spanned(
                    ph,
                    "modules should contain enum variants".to_string(),
                ));
            }
        }
    }

    Ok(modules)
}

pub fn register_modules(input: TokenStream) -> TokenStream {
    let module = parse_macro_input!(input as Modules);
    render(module)
}

fn render(modules: Modules) -> TokenStream {
    let Modules {
        function_name,
        modules,
    } = modules;

    let mut pub_mod = HashSet::new();
    let mut use_mod = Vec::new();
    let mut targets = Vec::new();
    for module in modules.elems.into_iter() {
        let decl = module.to_token_stream().to_string();
        let parts = decl.split("::").map(String::from).collect::<Vec<_>>();
        let last = parts.last().unwrap().trim();
        let first = parts.first().unwrap().trim().to_string();

        let module_name = module.clone();
        let type_name = Ident::new(&last.to_case(Case::UpperCamel), Span::call_site());
        targets.push(quote! {
            modules.insert_typeid(#module_name::#type_name::new(runtime.clone()));
        });

        pub_mod.insert(first.clone());

        use_mod.push(quote! {
            pub use #module::#type_name;
        });
    }

    let pub_mod = pub_mod.into_iter().map(|i| {
        let rust_module = Ident::new(&i, Span::call_site());
        quote! {
            pub mod #rust_module;
        }
    });

    let ts: TokenStream = quote! {

        #(#pub_mod)*

        #(#use_mod)*

        pub fn #function_name (runtime : &Runtime) -> HashMap::<TypeId, Module> {
            let mut modules = HashMap::<TypeId, Module>::new();

            #(#targets)*

            modules
        }

    }
    .into();

    // println!("{}", ts.to_string());

    ts
}
