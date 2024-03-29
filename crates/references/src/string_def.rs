use base_db::DocumentLocation;
use rowan::ast::AstNode;
use syntax::bibtex;

use crate::{Reference, ReferenceContext, ReferenceKind};

pub(super) fn find_all(context: &mut ReferenceContext) -> Option<()> {
    let document = context.params.feature.document;
    let data = document.data.as_bib()?;
    let root = data.root_node();
    let name = root
        .token_at_offset(context.params.offset)
        .filter(|token| token.kind() == bibtex::NAME)
        .find(|token| {
            let parent = token.parent().unwrap();
            bibtex::Value::can_cast(parent.kind()) || bibtex::StringDef::can_cast(parent.kind())
        })?;

    for string in &data.semantics.strings {
        if string.name.text == name.text() {
            context.results.push(Reference {
                location: DocumentLocation::new(document, string.name.range),
                kind: ReferenceKind::Definition,
            });
        }
    }

    for token in root
        .descendants()
        .filter_map(bibtex::Value::cast)
        .filter_map(|token| token.syntax().first_token())
        .filter(|token| token.text() == name.text())
    {
        context.results.push(Reference {
            location: DocumentLocation::new(document, token.text_range()),
            kind: ReferenceKind::Reference,
        });
    }

    Some(())
}
