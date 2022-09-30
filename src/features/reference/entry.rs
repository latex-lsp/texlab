use std::sync::Arc;

use lsp_types::ReferenceParams;
use rowan::ast::AstNode;

use crate::{
    features::cursor::CursorContext,
    syntax::{
        bibtex::{self, HasName},
        latex,
    },
    DocumentData,
};

use super::ReferenceResult;

pub(super) fn find_entry_references(
    context: &CursorContext<ReferenceParams>,
    results: &mut Vec<ReferenceResult>,
) -> Option<()> {
    let (key_text, _) = context
        .find_citation_key_word()
        .or_else(|| context.find_citation_key_command())
        .or_else(|| context.find_entry_key())?;

    for document in context.request.workspace.iter() {
        match document.data() {
            DocumentData::Latex(data) => {
                latex::SyntaxNode::new_root(data.green.clone())
                    .descendants()
                    .filter_map(latex::Citation::cast)
                    .filter_map(|citation| citation.key_list())
                    .flat_map(|keys| keys.keys())
                    .filter(|key| key.to_string() == key_text)
                    .map(|key| latex::small_range(&key))
                    .for_each(|range| {
                        let uri = Arc::clone(document.uri());
                        results.push(ReferenceResult { uri, range });
                    });
            }
            DocumentData::Bibtex(data) if context.request.params.context.include_declaration => {
                bibtex::SyntaxNode::new_root(data.green.clone())
                    .children()
                    .filter_map(bibtex::Entry::cast)
                    .filter_map(|entry| entry.name_token())
                    .filter(|key| key.text() == key_text)
                    .map(|key| key.text_range())
                    .for_each(|range| {
                        let uri = Arc::clone(document.uri());
                        results.push(ReferenceResult { uri, range });
                    });
            }
            DocumentData::Bibtex(_) | DocumentData::BuildLog(_) => {}
        }
    }

    Some(())
}
