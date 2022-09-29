use std::sync::Arc;

use lsp_types::GotoDefinitionParams;
use rowan::ast::AstNode;

use crate::{
    features::cursor::CursorContext,
    syntax::{
        bibtex::{self, HasName},
        latex,
    },
};

use super::DefinitionResult;

pub(super) fn goto_entry_definition(
    context: &CursorContext<GotoDefinitionParams>,
) -> Option<Vec<DefinitionResult>> {
    let word = context
        .cursor
        .as_latex()
        .filter(|token| token.kind() == latex::WORD)?;

    let key = latex::Key::cast(word.parent()?)?;

    latex::Citation::cast(key.syntax().parent()?.parent()?)?;

    let origin_selection_range = latex::small_range(&key);

    for document in context.request.workspace.iter() {
        if let Some(data) = document.data.as_bibtex() {
            for entry in bibtex::SyntaxNode::new_root(data.green.clone())
                .children()
                .filter_map(bibtex::Entry::cast)
            {
                if let Some(key) = entry.name_token().filter(|k| k.text() == word.text()) {
                    return Some(vec![DefinitionResult {
                        origin_selection_range,
                        target_uri: Arc::clone(&document.uri),
                        target_selection_range: key.text_range(),
                        target_range: entry.syntax().text_range(),
                    }]);
                }
            }
        }
    }

    None
}
