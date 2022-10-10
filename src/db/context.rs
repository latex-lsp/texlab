use crate::distro::Resolver;

#[salsa::input(singleton)]
pub struct ServerContext {
    #[return_ref]
    pub file_name_db: Resolver,

    #[return_ref]
    pub artifact_dir: String,
}
