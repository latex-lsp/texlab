use base_db::DocumentData;
use rowan::ast::AstNode;
use syntax::bibtex::{self, HasName};

use crate::util::cursor::CursorContext;

use super::DefinitionResult;

pub(super) fn goto_definition<'a>(
    context: &CursorContext<'a>,
) -> Option<Vec<DefinitionResult<'a>>> {
    let DocumentData::Bib(data) = &context.document.data else { return None };

    let key = context
        .cursor
        .as_bib()
        .filter(|token| token.kind() == bibtex::NAME)?;

    bibtex::Value::cast(key.parent()?)?;

    let origin_selection_range = key.text_range();

    data.root_node()
        .children()
        .filter_map(bibtex::StringDef::cast)
        .find_map(|string| {
            let string_name = string.name_token().filter(|k| k.text() == key.text())?;
            Some(vec![DefinitionResult {
                origin_selection_range,
                target: context.document,
                target_selection_range: string_name.text_range(),
                target_range: string.syntax().text_range(),
            }])
        })
}
