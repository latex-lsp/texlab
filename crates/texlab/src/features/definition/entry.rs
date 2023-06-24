use base_db::DocumentData;
use rowan::ast::AstNode;
use syntax::latex;

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

        for entry in data
            .semantics
            .entries
            .iter()
            .filter(|entry| entry.name.text == word.text())
        {
            return Some(vec![DefinitionResult {
                origin_selection_range,
                target: document,
                target_selection_range: entry.name.range,
                target_range: entry.full_range,
            }]);
        }
    }

    None
}
