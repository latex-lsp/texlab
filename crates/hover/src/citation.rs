use base_db::{DocumentData, util::queries};
use rowan::ast::AstNode;
use syntax::bibtex;

use crate::{Hover, HoverData, HoverParams};

pub(super) fn find_hover<'a>(params: &HoverParams<'a>) -> Option<Hover<'a>> {
    let HoverParams { feature, offset } = params;

    let (name, range) = match &feature.document.data {
        DocumentData::Tex(data) => {
            let result = queries::object_at_cursor(
                &data.semantics.citations,
                *offset,
                queries::SearchMode::Full,
            )?;
            (&result.object.name.text, result.range)
        }
        DocumentData::Bib(data) => {
            let result = queries::object_at_cursor(
                &data.semantics.entries,
                *offset,
                queries::SearchMode::Name,
            )?;
            (&result.object.name.text, result.range)
        }
        _ => return None,
    };

    let text = feature.project.documents.iter().find_map(|document| {
        let data = document.data.as_bib()?;
        let root = bibtex::Root::cast(data.root_node())?;
        let entry = root.find_entry(name)?;
        citeproc::render(&entry, &data.semantics)
    })?;

    let data = HoverData::Citation(text);
    Some(Hover { range, data })
}
