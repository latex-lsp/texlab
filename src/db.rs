pub mod analysis;
pub mod diagnostics;
mod discovery;
mod document;
pub mod parse;
mod workspace;

pub use {discovery::*, document::*, workspace::*};

#[salsa::interned]
pub struct Word {
    #[return_ref]
    pub text: String,
}
