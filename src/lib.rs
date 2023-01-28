#![allow(clippy::needless_lifetimes)]

pub(crate) mod citation;
mod client;
pub mod db;
pub(crate) mod distro;
pub mod features;
mod options;
pub mod parser;
mod server;
pub mod syntax;
pub(crate) mod util;

pub use self::{options::*, server::Server};

#[salsa::jar(db = Db)]
pub struct Jar(
    db::Word,
    db::Location,
    db::Location_path,
    db::Contents,
    db::Contents_line_index,
    db::LinterData,
    db::Document,
    db::Document_parse,
    db::Document_can_be_root,
    db::Document_can_be_built,
    db::parse::TexDocumentData,
    db::parse::TexDocumentData_analyze,
    db::parse::BibDocumentData,
    db::parse::LogDocumentData,
    db::analysis::TexLink,
    db::analysis::label::Number,
    db::analysis::label::Name,
    db::analysis::TheoremEnvironment,
    db::analysis::GraphicsPath,
    db::analysis::TexAnalysis,
    db::analysis::TexAnalysis_has_document_environment,
    db::MissingDependencies,
    db::hidden_dependency,
    db::source_dependency,
    db::dependency_graph,
    db::Workspace,
    db::Workspace_working_dir,
    db::Workspace_output_dir,
    db::Workspace_parents,
    db::Workspace_related,
    db::Workspace_number_of_label,
    db::diagnostics::tex::collect,
    db::diagnostics::bib::collect,
    db::diagnostics::log::collect,
    db::diagnostics::collect,
    db::diagnostics::collect_filtered,
);

pub trait Db: salsa::DbWithJar<Jar> {}

impl<DB> Db for DB where DB: ?Sized + salsa::DbWithJar<Jar> {}

#[salsa::db(crate::Jar)]
pub struct Database {
    storage: salsa::Storage<Self>,
}

impl Default for Database {
    fn default() -> Self {
        let storage = salsa::Storage::default();
        let db = Self { storage };
        db::Workspace::new(
            &db,
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
        );

        db
    }
}

impl salsa::Database for Database {}

impl salsa::ParallelDatabase for Database {
    fn snapshot(&self) -> salsa::Snapshot<Self> {
        salsa::Snapshot::new(Self {
            storage: self.storage.snapshot(),
        })
    }
}

pub(crate) fn normalize_uri(uri: &mut lsp_types::Url) {
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

#[cfg(test)]
mod tests;
