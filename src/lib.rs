#![feature(await_macro, async_await, futures_api)]

pub mod build;
pub mod completion;
pub mod definition;
pub mod feature;
pub mod folding;
pub mod formatting;
pub mod highlight;
pub mod link;
pub mod lsp;
pub mod range;
pub mod reference;
pub mod rename;
pub mod server;
pub mod syntax;
pub mod workspace;
