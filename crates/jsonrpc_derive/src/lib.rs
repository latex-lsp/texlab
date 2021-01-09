#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::*;

macro_rules! unwrap {
    ($input:expr, $arm:pat => $value:expr) => {{
        match $input {
            $arm => $value,
            _ => panic!(),
        }
    }};
}

enum MethodKind {
    Request,
    Notification,
}

struct MethodMeta {
    pub name: String,
    pub kind: MethodKind,
}

impl MethodMeta {
    pub fn parse(attr: &Attribute) -> Self {
        let meta = attr.parse_meta().unwrap();
        if meta.path().get_ident().unwrap() != "jsonrpc_method" {
            panic!("Expected jsonrpc_method attribute");
        }

        let nested = unwrap!(meta, Meta::List(x) => x.nested);
        let name = unwrap!(&nested[0], NestedMeta::Lit(Lit::Str(x)) => x.value());
        let kind = {
            let lit = unwrap!(&nested[1], NestedMeta::Meta(Meta::NameValue(x)) => &x.lit);
            let kind = unwrap!(lit, Lit::Str(x) => x.value());
            match kind.as_str() {
                "request" => MethodKind::Request,
                "notification" => MethodKind::Notification,
                _ => panic!(
                    "Invalid method kind. Valid options are \"request\" and \"notification\""
                ),
            }
        };

        Self { name, kind }
    }
}

#[proc_macro_attribute]
pub fn jsonrpc_method(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn jsonrpc_server(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let impl_: ItemImpl = parse_macro_input!(item);
    let generics = &impl_.generics;
    let self_ty = &impl_.self_ty;
    let (requests, notifications) = generate_server_skeletons(&impl_.items);

    let tokens = quote! {
        #impl_

        #[async_trait::async_trait]
        impl #generics jsonrpc::RequestHandler for #self_ty {
            async fn handle_request(&self, request: jsonrpc::Request) -> jsonrpc::Response {
                use jsonrpc::*;

                match request.method.as_str() {
                    #(#requests),*,
                    _ => {
                        Response::error(Error::method_not_found_error(), Some(request.id))
                    }
                }
            }

            async fn handle_notification(&self, notification: jsonrpc::Notification) {
                match notification.method.as_str() {
                    #(#notifications),*,
                    _ => log::warn!("{}: {}", "Method not found", notification.method),
                }
            }
        }
    };

    tokens.into()
}

#[proc_macro_attribute]
pub fn jsonrpc_client(attr: TokenStream, item: TokenStream) -> TokenStream {
    let trait_: ItemTrait = parse_macro_input!(item);
    let trait_ident = &trait_.ident;
    let stubs = generate_client_stubs(&trait_.items);
    let attr: AttributeArgs = parse_macro_input!(attr);
    let struct_ident = unwrap!(attr.first().unwrap(), NestedMeta::Meta(Meta::Path(x)) => x);

    let tokens = quote! {
        #[async_trait::async_trait]
        #trait_

        pub struct #struct_ident {
            client: jsonrpc::Client
        }

        impl #struct_ident
        {
            pub fn new(output: futures::channel::mpsc::Sender<String>) -> Self {
                Self {
                    client: jsonrpc::Client::new(output),
                }
            }
        }

        #[async_trait::async_trait]
        impl #trait_ident for #struct_ident
        {
            #(#stubs)*
        }

        #[async_trait::async_trait]
        impl jsonrpc::ResponseHandler for #struct_ident
        {
            async fn handle(&self, response: jsonrpc::Response) -> () {
                self.client.handle(response).await
            }
        }
    };

    tokens.into()
}

fn generate_server_skeletons(items: &Vec<ImplItem>) -> (Vec<TokenStream2>, Vec<TokenStream2>) {
    let mut requests = Vec::new();
    let mut notifications = Vec::new();
    for item in items {
        let method = unwrap!(item, ImplItem::Method(x) => x);
        if method.attrs.is_empty() {
            continue;
        }

        let ident = &method.sig.ident;
        let param_ty = unwrap!(&method.sig.inputs[1], FnArg::Typed(x) => &x.ty);
        let meta = MethodMeta::parse(method.attrs.first().unwrap());
        let name = &meta.name.as_str();

        match meta.kind {
            MethodKind::Request => {
                requests.push(quote!(
                    #name => {
                        let handler = |param: #param_ty| async move {
                           self.#ident(param).await
                        };

                        jsonrpc::handle_request(request, handler).await
                    }
                ));
            }
            MethodKind::Notification => {
                notifications.push(quote!(
                    #name => {
                        let handler = |param: #param_ty| async move {
                           self.#ident(param).await;
                        };

                        jsonrpc::handle_notification(notification, handler).await;
                    }
                ));
            }
        }
    }

    (requests, notifications)
}

fn generate_client_stubs(items: &Vec<TraitItem>) -> Vec<TokenStream2> {
    let mut stubs = Vec::new();
    for item in items {
        let method = unwrap!(item, TraitItem::Method(x) => x);
        let attrs = &method.attrs;
        let sig = &method.sig;
        let param = unwrap!(&sig.inputs[1], FnArg::Typed(x) => &x.pat);
        let meta = MethodMeta::parse(attrs.first().unwrap());
        let name = &meta.name;

        let stub = match meta.kind {
            MethodKind::Request => quote!(
                #sig {
                    let result = self.client.send_request(#name.to_owned(), #param).await?;
                    serde_json::from_value(result).map_err(|_| jsonrpc::Error::deserialize_error())
                }
            ),
            MethodKind::Notification => quote!(
                #sig {
                    self.client.send_notification(#name.to_owned(), #param).await
                }
            ),
        };

        stubs.push(stub);
    }

    stubs
}
