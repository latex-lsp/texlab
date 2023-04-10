use base_db::DocumentData;
use lsp_types::ReferenceContext;
use rowan::ast::AstNode;
use syntax::{
    bibtex::{self, HasName},
    latex,
};

use crate::util::cursor::CursorContext;

use super::ReferenceResult;

pub(super) fn find_all_references<'a>(
    context: &CursorContext<'a, &ReferenceContext>,
    results: &mut Vec<ReferenceResult<'a>>,
) -> Option<()> {
    let (key_text, _) = context
        .find_citation_key_word()
        .or_else(|| context.find_citation_key_command())
        .or_else(|| context.find_entry_key())?;

    for document in &context.project.documents {
        match &document.data {
            DocumentData::Tex(data) => {
                data.root_node()
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
                data.root_node()
                    .children()
                    .filter_map(bibtex::Entry::cast)
                    .filter_map(|entry| entry.name_token())
                    .filter(|key| key.text() == key_text)
                    .map(|key| key.text_range())
                    .for_each(|range| {
                        results.push(ReferenceResult { document, range });
                    });
            }
            DocumentData::Bib(_)
            | DocumentData::Aux(_)
            | DocumentData::Log(_)
            | DocumentData::Root
            | DocumentData::Tectonic => {}
        };
    }

    Some(())
}
