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
    server::Server,
    workspace::{Workspace, WorkspaceEvent},
};

pub fn normalize_uri(uri: &mut lsp_types::Url) {
    fn fix_drive_letter(text: &str) -> Option<String> {
        if !text.is_ascii() {
            return None;
        }

        match &text[1..] {
            ":" => Some(text.to_ascii_uppercase()),
            "%3A" | "%3a" => Some(format!("{}:", text[0..1].to_ascii_uppercase())),
            _ => None,
        }
    }

    if let Some(mut segments) = uri.path_segments() {
        if let Some(mut path) = segments.next().and_then(fix_drive_letter) {
            for segment in segments {
                path.push('/');
                path.push_str(segment);
            }

            uri.set_path(&path);
        }
    }

    uri.set_fragment(None);
}
