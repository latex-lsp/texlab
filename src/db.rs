pub mod analysis;
pub mod diagnostics;
mod discovery;
pub mod document;
pub mod parse;
pub mod workspace;

pub use discovery::*;

#[salsa::interned]
pub struct Word {
    #[return_ref]
    pub text: String,
}
