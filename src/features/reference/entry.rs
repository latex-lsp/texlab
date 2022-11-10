use lsp_types::ReferenceContext;
use rowan::ast::AstNode;

use crate::{
    db::parse::DocumentData,
    syntax::{
        bibtex::{self, HasName},
        latex,
    },
    util::cursor::CursorContext,
};

use super::ReferenceResult;

pub(super) fn find_all_references(
    context: &CursorContext<&ReferenceContext>,
    results: &mut Vec<ReferenceResult>,
) -> Option<()> {
    let db = context.db;
    let (key_text, _) = context
        .find_citation_key_word()
        .or_else(|| context.find_citation_key_command())
        .or_else(|| context.find_entry_key())?;

    for document in context.related() {
        match document.parse(db) {
            DocumentData::Tex(data) => {
                data.root(db)
                    .descendants()
                    .filter_map(latex::Citation::cast)
                    .filter_map(|citation| citation.key_list())
                    .flat_map(|keys| keys.keys())
                    .filter(|key| key.to_string() == key_text)
                    .map(|key| latex::small_range(&key))
                    .for_each(|range| {
                        results.push(ReferenceResult { document, range });
                    });
            }
            DocumentData::Bib(data) if context.params.include_declaration => {
                data.root(db)
                    .children()
                    .filter_map(bibtex::Entry::cast)
                    .filter_map(|entry| entry.name_token())
                    .filter(|key| key.text() == key_text)
                    .map(|key| key.text_range())
                    .for_each(|range| {
                        results.push(ReferenceResult { document, range });
                    });
            }
            DocumentData::Bib(_) | DocumentData::Log(_) => {}
        };
    }

    Some(())
}
