use base_db::{semantics::Span, DocumentLocation};
use rowan::ast::AstNode;
use rustc_hash::FxHashSet;
use syntax::latex;

use crate::{Reference, ReferenceContext, ReferenceKind};

pub(super) fn find_all(context: &mut ReferenceContext) -> Option<()> {
    let data = context.params.feature.document.data.as_tex()?;
    let token = data
        .root_node()
        .token_at_offset(context.params.offset)
        .find(|token| token.kind() == latex::COMMAND_NAME)?;

    let project = &context.params.feature.project;

    for document in &project.documents {
        if let Some(data) = document.data.as_tex() {
            let defs: FxHashSet<Span> = data
                .root_node()
                .descendants()
                .filter_map(|node| {
                    latex::OldCommandDefinition::cast(node.clone())
                        .and_then(|node| node.name())
                        .or_else(|| {
                            latex::NewCommandDefinition::cast(node).and_then(|node| node.name())
                        })
                        .map(|name| Span::command(&name))
                })
                .collect();

            for command in &data.semantics.commands {
                let command_text = command.text.trim_end_matches('*');
                let token_text = token.text()[1..].trim_end_matches('*');
                if command_text == token_text {
                    let kind = if defs.contains(command) {
                        ReferenceKind::Definition
                    } else {
                        ReferenceKind::Reference
                    };

                    context.results.push(Reference {
                        location: DocumentLocation::new(document, command.range),
                        kind,
                    });
                }
            }
        }
    }

    Some(())
}
