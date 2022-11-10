use lsp_types::ReferenceContext;
use rowan::ast::AstNode;

use crate::{
    syntax::bibtex::{self, HasName},
    util::cursor::CursorContext,
};

use super::ReferenceResult;

pub(super) fn find_all_references(
    context: &CursorContext<&ReferenceContext>,
    results: &mut Vec<ReferenceResult>,
) -> Option<()> {
    let db = context.db;
    let name_text = context
        .cursor
        .as_bib()
        .filter(|token| token.kind() == bibtex::NAME)
        .filter(|token| {
            let parent = token.parent().unwrap();
            bibtex::Value::can_cast(parent.kind()) || bibtex::StringDef::can_cast(parent.kind())
        })?
        .text();

    let data = context.document.parse(db).as_bib()?;
    for node in data.root(db).descendants() {
        if let Some(name) = bibtex::StringDef::cast(node.clone())
            .and_then(|string| string.name_token())
            .filter(|name| context.params.include_declaration && name.text() == name_text)
            .or_else(|| {
                bibtex::Value::cast(node)
                    .and_then(|token| token.syntax().first_token())
                    .filter(|name| name.text() == name_text)
            })
        {
            results.push(ReferenceResult {
                document: context.document,
                range: name.text_range(),
            });
        }
    }

    Some(())
}
