use lsp_types::CompletionParams;
use rowan::ast::AstNode;

use crate::{features::cursor::CursorContext, syntax::latex};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_glossary_entries<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
) -> Option<()> {
    let (_, range, group) = context.find_curly_group_word()?;
    latex::GlossaryEntryReference::cast(group.syntax().parent()?)?;

    for document in context.request.workspace.documents_by_uri.values() {
        if let Some(data) = document.data.as_latex() {
            for node in latex::SyntaxNode::new_root(data.green.clone()).descendants() {
                if let Some(name) = latex::GlossaryEntryDefinition::cast(node.clone())
                    .and_then(|entry| entry.name())
                    .and_then(|name| name.key())
                    .map(|name| name.to_string())
                {
                    items.push(InternalCompletionItem::new(
                        range,
                        InternalCompletionItemData::GlossaryEntry { name },
                    ));
                } else if let Some(name) = latex::AcronymDefinition::cast(node)
                    .and_then(|entry| entry.name())
                    .and_then(|name| name.key())
                    .map(|name| name.to_string())
                {
                    items.push(InternalCompletionItem::new(
                        range,
                        InternalCompletionItemData::Acronym { name },
                    ));
                }
            }
        }
    }

    Some(())
}
