#[cfg(feature = "citation")]
pub mod citeproc;

#[cfg(feature = "test")]
pub mod test;

cfg_if::cfg_if! {
    if #[cfg(feature = "server")] {
        mod config;
        mod build;

        pub mod server;
    }
}

pub mod completion;
pub mod components;
pub mod definition;
pub mod diagnostics;
pub mod feature;
pub mod folding;
pub mod forward_search;
pub mod highlight;
pub mod hover;
pub mod link;
pub mod outline;
pub mod protocol;
pub mod reference;
pub mod rename;
pub mod symbol;
pub mod syntax;
pub mod tex;
pub mod workspace;
