use std::sync::Arc;

use lsp_types::GotoDefinitionParams;
use rowan::ast::AstNode;

use crate::{
    features::cursor::CursorContext,
    syntax::bibtex::{self, HasName},
};

use super::DefinitionResult;

pub(super) fn goto_string_definition(
    context: &CursorContext<GotoDefinitionParams>,
) -> Option<Vec<DefinitionResult>> {
    let main_document = context.request.main_document();

    let data = main_document.data().as_bibtex()?;
    let key = context
        .cursor
        .as_bibtex()
        .filter(|token| token.kind() == bibtex::NAME)?;

    bibtex::Value::cast(key.parent()?)?;

    let origin_selection_range = key.text_range();

    for string in bibtex::SyntaxNode::new_root(data.green.clone())
        .children()
        .filter_map(bibtex::StringDef::cast)
    {
        if let Some(string_name) = string.name_token().filter(|k| k.text() == key.text()) {
            return Some(vec![DefinitionResult {
                origin_selection_range,
                target_uri: Arc::clone(main_document.uri()),
                target_selection_range: string_name.text_range(),
                target_range: string.syntax().text_range(),
            }]);
        }
    }

    None
}
