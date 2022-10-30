pub mod analysis;
pub mod dependency;
pub mod document;
pub mod parse;
pub mod workspace;

#[salsa::interned]
pub struct Word {
    #[return_ref]
    pub text: String,
}

#[salsa::input(singleton)]
pub struct Distro {
    #[return_ref]
    pub file_name_db: crate::distro::Resolver,
}
