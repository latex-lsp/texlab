use base_db::DocumentData;
use rowan::ast::AstNode;
use syntax::{
    bibtex::{self, HasName},
    latex,
};

use crate::util::cursor::CursorContext;

use super::DefinitionResult;

pub(super) fn goto_definition<'a>(
    context: &CursorContext<'a>,
) -> Option<Vec<DefinitionResult<'a>>> {
    let word = context
        .cursor
        .as_tex()
        .filter(|token| token.kind() == latex::WORD)?;

    let key = latex::Key::cast(word.parent()?)?;

    latex::Citation::cast(key.syntax().parent()?.parent()?)?;

    let origin_selection_range = latex::small_range(&key);

    for document in &context.project.documents {
        let DocumentData::Bib(data) = &document.data else { continue };

        for entry in data.root_node().children().filter_map(bibtex::Entry::cast) {
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

    None
}
