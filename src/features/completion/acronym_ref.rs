use lsp_types::CompletionParams;
use rowan::ast::AstNode;

use crate::{features::cursor::CursorContext, syntax::latex};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_acronyms<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
) -> Option<()> {
    let (_, range, group) = context.find_curly_group_word()?;
    latex::AcronymReference::cast(group.syntax().parent()?)?;

    for document in context.request.workspace.iter() {
        if let Some(data) = document.data().as_latex() {
            for name in latex::SyntaxNode::new_root(data.green.clone())
                .descendants()
                .filter_map(latex::AcronymDefinition::cast)
                .filter_map(|node| node.name())
                .filter_map(|name| name.key())
                .map(|name| name.to_string())
            {
                items.push(InternalCompletionItem::new(
                    range,
                    InternalCompletionItemData::Acronym { name },
                ));
            }
        }
    }
    Some(())
}
