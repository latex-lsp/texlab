use base_db::{
    semantics::{bib::Entry, tex::Citation},
    util::queries::Object,
    Document,
};
use rowan::ast::AstNode;
use rustc_hash::FxHashMap;
use syntax::bibtex;

use crate::{InlayHint, InlayHintBuilder, InlayHintData};

pub(super) fn find_hints(builder: &mut InlayHintBuilder) -> Option<()> {
    let params = &builder.params.feature;
    let data = params.document.data.as_tex()?;
    let range = builder.params.range;

    let entries = Entry::find_all(&params.project)
        .map(|(document, entry)| (entry.name_text(), (document, entry)))
        .collect::<FxHashMap<_, _>>();

    for citation in data
        .semantics
        .citations
        .iter()
        .filter(|citation| citation.name.range.intersect(range).is_some())
    {
        if let Some(hint) = process_citation(&entries, citation) {
            builder.hints.push(hint);
        }
    }

    Some(())
}

fn process_citation<'a>(
    entries: &FxHashMap<&str, (&'a Document, &'a Entry)>,
    citation: &'a Citation,
) -> Option<InlayHint<'a>> {
    let offset = citation.name.range.end();
    let (document, entry) = entries.get(citation.name.text.as_str())?;

    let data = document.data.as_bib()?;
    let root = &data.root_node();
    let name = root
        .token_at_offset(entry.name.range.start())
        .right_biased()?;

    let entry = name.parent_ancestors().find_map(bibtex::Entry::cast)?;
    let options = citeproc::Options {
        mode: citeproc::Mode::Overview,
    };

    let text = citeproc::render(&entry, &options)?;
    let data = InlayHintData::Citation(text);
    Some(InlayHint { offset, data })
}
