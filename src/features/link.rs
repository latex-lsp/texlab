mod include;

use lsp_types::{DocumentLink, Url};
use rowan::TextRange;

use crate::{
    db::{document::Document, workspace::Workspace},
    util::{line_index::LineIndex, line_index_ext::LineIndexExt},
    Db,
};

pub fn find_all(db: &dyn Db, uri: &Url) -> Option<Vec<DocumentLink>> {
    let document = Workspace::get(db).lookup_uri(db, uri)?;
    let mut builder = LinkBuilder {
        db,
        line_index: document.contents(db).line_index(db),
        links: Vec::new(),
    };

    include::find_links(db, document, &mut builder);
    Some(builder.links)
}

struct LinkBuilder<'db> {
    db: &'db dyn Db,
    line_index: &'db LineIndex,
    links: Vec<DocumentLink>,
}

impl<'db> LinkBuilder<'db> {
    pub fn push(&mut self, range: TextRange, target: Document) {
        let range = self.line_index.line_col_lsp_range(range);
        let target = Some(target.location(self.db).uri(self.db).clone());
        self.links.push(DocumentLink {
            range,
            target,
            tooltip: None,
            data: None,
        });
    }
}
