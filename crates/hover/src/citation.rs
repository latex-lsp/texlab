use base_db::{util::queries, DocumentData};
use rowan::ast::AstNode;
use syntax::bibtex;

use crate::{Hover, HoverData, HoverParams};

pub(super) fn find_hover<'db>(params: &HoverParams<'db>) -> Option<Hover<'db>> {
    let offset = params.offset;

    let (name, range) = match &params.document.data {
        DocumentData::Tex(data) => {
            let result = queries::object_at_cursor(
                &data.semantics.citations,
                offset,
                queries::SearchMode::Full,
            )?;
            (&result.object.name.text, result.range)
        }
        DocumentData::Bib(data) => {
            let result = queries::object_at_cursor(
                &data.semantics.entries,
                offset,
                queries::SearchMode::Name,
            )?;
            (&result.object.name.text, result.range)
        }
        _ => return None,
    };

    let text = params.project.documents.iter().find_map(|document| {
        let data = document.data.as_bib()?;
        let root = bibtex::Root::cast(data.root_node())?;
        let entry = root.find_entry(name)?;
        citeproc::render(&entry)
    })?;

    let data = HoverData::Citation(text);
    Some(Hover { range, data })
}
