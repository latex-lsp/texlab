use rowan::ast::AstNode;

use crate::{syntax::latex, util::cursor::CursorContext, LANGUAGE_DATA};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_colors<'db>(
    context: &'db CursorContext,
    items: &mut Vec<InternalCompletionItem<'db>>,
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
