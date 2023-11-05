mod include;

use base_db::{Document, Workspace};
use lsp_types::{DocumentLink, Url};
use rowan::TextRange;

use crate::util::line_index_ext::LineIndexExt;

pub fn find_all(workspace: &Workspace, uri: &Url) -> Option<Vec<DocumentLink>> {
    let document = workspace.lookup(uri)?;
    let mut builder = LinkBuilder {
        workspace,
        document,
        links: Vec::new(),
    };

    include::find_links(&mut builder);
    Some(builder.links)
}

struct LinkBuilder<'a> {
    workspace: &'a Workspace,
    document: &'a Document,
    links: Vec<DocumentLink>,
}

impl<'a> LinkBuilder<'a> {
    pub fn push(&mut self, range: TextRange, target: &Document) {
        let Some(range) = self.document.line_index.line_col_lsp_range(range) else {
            return;
        };

        let target = Some(target.uri.clone());
        self.links.push(DocumentLink {
            range,
            target,
            tooltip: None,
            data: None,
        });
    }
}
