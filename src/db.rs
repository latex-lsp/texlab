pub mod analysis;
pub mod dependency;
pub mod diagnostics;
pub mod document;
pub mod parse;
pub mod workspace;

#[salsa::interned]
pub struct Word {
    #[return_ref]
    pub text: String,
}
