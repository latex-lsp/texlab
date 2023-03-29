use rowan::ast::AstNode;
use syntax::{
    bibtex::{self, HasName},
    latex,
};

use crate::util::cursor::CursorContext;

use super::DefinitionResult;

pub(super) fn goto_definition(context: &CursorContext) -> Option<Vec<DefinitionResult>> {
    let db = context.db;

    let word = context
        .cursor
        .as_tex()
        .filter(|token| token.kind() == latex::WORD)?;

    let key = latex::Key::cast(word.parent()?)?;

    latex::Citation::cast(key.syntax().parent()?.parent()?)?;

    let origin_selection_range = latex::small_range(&key);

    for document in context.related() {
        if let Some(data) = document.parse(db).as_bib() {
            for entry in data.root(db).children().filter_map(bibtex::Entry::cast) {
                if let Some(key) = entry.name_token().filter(|k| k.text() == word.text()) {
                    return Some(vec![DefinitionResult {
                        origin_selection_range,
                        target: document,
                        target_selection_range: key.text_range(),
                        target_range: entry.syntax().text_range(),
                    }]);
                }
            }
        }
    }

    None
}
