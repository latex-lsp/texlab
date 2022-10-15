mod capabilities;
pub mod citation;
mod client;
pub mod component_db;
mod debouncer;
mod diagnostics;
mod dispatch;
pub mod distro;
mod document;
mod environment;
pub mod features;
mod label;
mod lang_data;
mod language;
mod line_index;
mod line_index_ext;
mod options;
pub mod parser;
mod range;
mod server;
pub mod syntax;
mod workspace;

pub use self::{
    capabilities::ClientCapabilitiesExt,
    document::*,
    environment::Environment,
    label::*,
    lang_data::*,
    language::DocumentLanguage,
    line_index::{LineCol, LineColUtf16, LineIndex},
    line_index_ext::LineIndexExt,
    options::*,
    range::RangeExt,
    server::Server,
    workspace::{Workspace, WorkspaceEvent},
};

pub(crate) fn normalize_uri(uri: &mut lsp_types::Url) {
    if let Some(mut segments) = uri.path_segments() {
        if let Some(root) = segments
            .next()
            .filter(|name| name.is_ascii() && name.len() == 2 && name.ends_with(":"))
        {
            let mut path = root.to_ascii_uppercase();
            for segment in segments {
                path.push('/');
                path.push_str(segment);
            }

            uri.set_path(&path);
        }
    }

    uri.set_fragment(None);
}
