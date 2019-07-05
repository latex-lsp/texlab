#![feature(async_await)]
#![recursion_limit = "128"]

pub mod action;
pub mod build;
pub mod client;
pub mod codec;
pub mod completion;
pub mod data;
pub mod definition;
pub mod diagnostics;
pub mod feature;
pub mod folding;
pub mod formatting;
pub mod forward_search;
pub mod highlight;
pub mod hover;
pub mod link;
pub mod outline;
pub mod reference;
pub mod rename;
pub mod scenario;
pub mod server;
pub mod syntax;
pub mod tex;
pub mod workspace;
