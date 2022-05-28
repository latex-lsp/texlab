use std::sync::Arc;

use lsp_types::ReferenceParams;
use rowan::ast::AstNode;

use crate::{
    features::cursor::CursorContext,
    syntax::bibtex::{self, HasName},
};

use super::ReferenceResult;

pub(super) fn find_string_references(
    context: &CursorContext<ReferenceParams>,
    results: &mut Vec<ReferenceResult>,
) -> Option<()> {
    let name_text = context
        .cursor
        .as_bibtex()
        .filter(|token| token.kind() == bibtex::NAME)
        .filter(|token| {
            let parent = token.parent().unwrap();
            bibtex::Value::can_cast(parent.kind()) || bibtex::StringDef::can_cast(parent.kind())
        })?
        .text();

    let document = context.request.main_document();
    let data = document.data.as_bibtex()?;
    for node in bibtex::SyntaxNode::new_root(data.green.clone()).descendants() {
        if let Some(name) = bibtex::StringDef::cast(node.clone())
            .and_then(|string| string.name_token())
            .filter(|name| {
                context.request.params.context.include_declaration && name.text() == name_text
            })
            .or_else(|| {
                bibtex::Value::cast(node)
                    .and_then(|token| token.syntax().first_token())
                    .filter(|name| name.text() == name_text)
            })
        {
            results.push(ReferenceResult {
                uri: Arc::clone(&document.uri),
                range: name.text_range(),
            });
        }
    }

    Some(())
}
