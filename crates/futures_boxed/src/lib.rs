#![feature(async_await)]
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
    let decl = boxed_fn_decl(&fn_.decl, &fn_.constness, &fn_.ident);
    let block = &fn_.block;
    let tokens = quote! {
        #(#attrs)*
        #vis #decl {
            use futures::future::FutureExt;
            let task = async move #block;
            task.boxed()
        }
    };

    tokens.into()
}

fn boxed_trait_method(method: TraitItemMethod) -> TokenStream {
    let attrs = &method.attrs;
    let decl = boxed_fn_decl(&method.sig.decl, &method.sig.constness, &method.sig.ident);
    let tokens = quote! {
        #(#attrs)*
        #decl;
    };

    tokens.into()
}

fn boxed_fn_decl(
    decl: &FnDecl,
    constness: &Option<syn::token::Const>,
    ident: &Ident,
) -> TokenStream2 {
    let generics = &decl.generics;
    let inputs = &decl.inputs;
    let return_ty = match &decl.output {
        ReturnType::Default => quote!(()),
        ReturnType::Type(_, ty) => ty.into_token_stream(),
    };

    quote! {
        #constness fn #ident #generics(#inputs) -> futures::future::BoxFuture<'_, #return_ty>
    }
}
