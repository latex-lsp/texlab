#![feature(await_macro, async_await)]
#![recursion_limit = "128"]

extern crate proc_macro;

use quote::quote;
use syn;

macro_rules! unwrap {
    ($input:expr, $arm:pat => $value:expr) => {{
        match $input {
            $arm => $value,
            _ => unreachable!(),
        }
    }};
}

#[proc_macro_attribute]
pub fn jsonrpc_server(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let impl_: syn::ItemImpl = syn::parse_macro_input!(item);
    let mut requests = Vec::new();
    let mut notifications = Vec::new();
    let methods = &get_methods(&impl_);
    for method in methods {
        let ident = &method.sig.ident;
        let attribute = method.attrs.first().unwrap().clone();

        let meta = attribute.parse_meta().unwrap();
        let meta_list = unwrap!(meta, syn::Meta::List(x) => x);
        let meta_nested = meta_list.nested.first().unwrap();
        let meta_lit = unwrap!(meta_nested.value(), syn::NestedMeta::Literal(x) => x);
        let name = unwrap!(meta_lit, syn::Lit::Str(x) => x.value());
        let name_str = name.as_str();

        if is_request_method(&method) {
            requests.push(quote!(#name_str => await!(self.#ident(request))));
        } else {
            notifications.push(quote!(#name_str => self.#ident(notification)));
        }
    }

    let self_ty = &impl_.self_ty;
    let result = quote! {
        impl jsonrpc::Server for #self_ty {
            fn handle_request(&self, request: jsonrpc::Request)
                -> futures::future::BoxFuture<'_, jsonrpc::Response> {
                use futures::prelude::*;
                let handler = async move {
                    match request.method.as_str() {
                        #(#requests),*,
                        _ => {
                            let error = jsonrpc::Error {
                                code: jsonrpc::ErrorCode::MethodNotFound,
                                message: "Method not found".to_owned(),
                                data: serde_json::Value::Null,
                            };

                            jsonrpc::Response::error(error, Some(request.id))
                        }
                    }
                };

                handler.boxed()
            }

            fn handle_notification(&self, notification: jsonrpc::Notification) {
                match notification.method.as_str() {
                    #(#notifications),*,
                    _ => log::warn!("{}: {}", "Method not found", notification.method),
                }
            }
        }

        #impl_
    };

    result.into()
}

#[proc_macro_attribute]
pub fn jsonrpc_method(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let method: syn::ImplItemMethod = syn::parse_macro_input!(item);
    let name = &method.sig.ident;
    let block = &method.block;
    let param_decl = match &method.sig.decl.inputs[1] {
        syn::FnArg::Captured(arg) => arg,
        _ => panic!("Could not extract parameter type"),
    };
    let param_name = &param_decl.pat;
    let param_type = &param_decl.ty;
    let return_ty = &method.sig.decl.output;

    let result = if is_request_method(&method) {
        quote! {
            pub async fn #name(&self, request: jsonrpc::Request) -> jsonrpc::Response {
                let handler = async move |#param_name: #param_type| #return_ty #block;
                await!(jsonrpc::handle_request(request, handler))
            }
        }
    } else {
        quote! {
            pub fn #name(&self, notification: jsonrpc::Notification) {
                let handler = move |#param_name: #param_type| #block;
                jsonrpc::handle_notification(notification, handler);
            }
        }
    };

    result.into()
}

fn get_methods(impl_: &syn::ItemImpl) -> Vec<&syn::ImplItemMethod> {
    let mut methods = Vec::new();
    for item in &impl_.items {
        let method = unwrap!(item, syn::ImplItem::Method(x) => x);
        if !method.attrs.is_empty() {
            methods.push(method);
        }
    }

    methods
}

fn is_request_method(method: &syn::ImplItemMethod) -> bool {
    match method.sig.decl.output {
        syn::ReturnType::Type(_, _) => true,
        syn::ReturnType::Default => false,
    }
}
