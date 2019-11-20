#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::{TokenStream, TokenTree};
use quote::quote;
use quote::ToTokens;
use std::iter::FromIterator;
use syn::export::TokenStream2;
use syn::*;

#[proc_macro_attribute]
pub fn boxed(_attr: TokenStream, item: TokenStream) -> TokenStream {
    match parse::<ItemFn>(item.clone()) {
        Ok(fn_) => boxed_fn(fn_),
        Err(_) => {
            let item = TokenStream::from_iter(item.into_iter().filter(|x| match x {
                TokenTree::Ident(x) if x.to_string() == "async" => false,
                _ => true,
            }));

            let method: TraitItemMethod = parse(item).unwrap();
            boxed_trait_method(method)
        }
    }
}

fn boxed_fn(fn_: ItemFn) -> TokenStream {
    let attrs = &fn_.attrs;
    let vis = &fn_.vis;
    let sig = boxed_fn_sig(&fn_.sig);
    let block = &fn_.block;
    let tokens = quote! {
        #(#attrs)*
        #vis #sig {
            use futures::future::FutureExt;
            let task = async move #block;
            task.boxed()
        }
    };

    tokens.into()
}

fn boxed_trait_method(method: TraitItemMethod) -> TokenStream {
    let attrs = &method.attrs;
    let sig = boxed_fn_sig(&method.sig);
    let tokens = quote! {
        #(#attrs)*
        #sig;
    };

    tokens.into()
}

fn boxed_fn_sig(sig: &Signature) -> TokenStream2 {
    let constness = &sig.constness;
    let ident = &sig.ident;
    let generics = &sig.generics;
    let inputs = &sig.inputs;
    let return_ty = match &sig.output {
        ReturnType::Default => quote!(()),
        ReturnType::Type(_, ty) => ty.into_token_stream(),
    };

    quote! {
        #constness fn #ident #generics(#inputs) -> futures::future::BoxFuture<'_, #return_ty>
    }
}
