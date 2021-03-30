use std::sync::Arc;

use smol_str::SmolStr;

use crate::protocol::Uri;

use super::{Document, ParserDatabase};

#[salsa::query_group(LinkDatabaseStorage)]
pub trait LinkDatabase: ParserDatabase {
    fn base_uri(&self, document: Document) -> Arc<Uri>;

    fn implicit_links_by_extension(
        &self,
        document: Document,
        extension: SmolStr,
    ) -> Option<Arc<Vec<Arc<Uri>>>>;

    fn implicit_links(&self, document: Document) -> Arc<ImplicitLinks>;

    // fn target_uri(&self) -> Arc<Vec<Arc<Uri>>>;

    // fn explicit_links(&self, document: Document) -> Vec<ExplicitLink>;
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Default)]
pub struct ImplicitLinks {
    pub aux: Arc<Vec<Arc<Uri>>>,
    pub log: Arc<Vec<Arc<Uri>>>,
    pub pdf: Arc<Vec<Arc<Uri>>>,
}

fn base_uri(db: &dyn LinkDatabase, document: Document) -> Arc<Uri> {
    let document_uri = db.lookup_intern_document(document).uri;
    match &db.options().latex.root_directory {
        Some(root_dir) => Uri::from_directory_path(root_dir)
            .map(Arc::new)
            .unwrap_or_else(|()| Arc::clone(&document_uri)),
        None => Arc::clone(&document_uri),
    }
}

fn implicit_links_by_extension(
    db: &dyn LinkDatabase,
    document: Document,
    extension: SmolStr,
) -> Option<Arc<Vec<Arc<Uri>>>> {
    let uri = db.lookup_intern_document(document).uri;
    let mut targets = vec![Arc::new(uri.with_extension(&extension)?)];
    if uri.scheme() == "file" {
        let file_path = uri.to_file_path().ok()?;
        let file_stem = file_path.file_stem()?;
        let aux_name = format!("{}.{}", file_stem.to_str()?, extension);

        let options = db.options();
        let current_dir = db.current_dir();
        if let Some(root_dir) = options.latex.root_directory.as_ref() {
            let path = current_dir.join(root_dir).join(&aux_name);
            targets.push(Arc::new(Uri::from_file_path(path).ok()?));
        }

        if let Some(build_dir) = options.latex.build.output_directory.as_ref() {
            let path = current_dir.join(build_dir).join(&aux_name);
            targets.push(Arc::new(Uri::from_file_path(path).ok()?));
        }
    }
    Some(Arc::new(targets))
}

fn implicit_links(db: &dyn LinkDatabase, document: Document) -> Arc<ImplicitLinks> {
    let aux = db
        .implicit_links_by_extension(document, "aux".into())
        .unwrap_or_default();

    let log = db
        .implicit_links_by_extension(document, "log".into())
        .unwrap_or_default();

    let pdf = db
        .implicit_links_by_extension(document, "pdf".into())
        .unwrap_or_default();

    Arc::new(ImplicitLinks { aux, log, pdf })
}
