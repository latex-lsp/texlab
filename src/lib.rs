#![feature(await_macro, async_await)]
#![recursion_limit = "128"]

pub mod build;
pub mod client;
pub mod codec;
pub mod completion;
pub mod data;
pub mod definition;
pub mod diagnostics;
pub mod event;
pub mod feature;
pub mod folding;
pub mod formatting;
pub mod highlight;
pub mod hover;
pub mod link;
pub mod reference;
pub mod rename;
pub mod resolver;
pub mod server;
pub mod syntax;
pub mod workspace;
