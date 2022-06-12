use lsp_types::CompletionParams;
use rowan::ast::AstNode;

use crate::{features::cursor::CursorContext, syntax::latex, LANGUAGE_DATA};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_colors<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
) -> Option<()> {
    let (_, range, group) = context.find_curly_group_word()?;
    latex::ColorReference::cast(group.syntax().parent()?)?;

    for name in &LANGUAGE_DATA.colors {
        items.push(InternalCompletionItem::new(
            range,
            InternalCompletionItemData::Color { name },
        ));
    }

    Some(())
}
