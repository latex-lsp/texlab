use rowan::ast::AstNode;

use crate::{syntax::latex, util::cursor::CursorContext};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_acronyms<'db>(
    context: &'db CursorContext,
    items: &mut Vec<InternalCompletionItem<'db>>,
) -> Option<()> {
    let (_, range, group) = context.find_curly_group_word()?;
    latex::AcronymReference::cast(group.syntax().parent()?)?;

    for document in context.related() {
        if let Some(data) = document.parse(context.db).as_tex() {
            for name in data
                .root(context.db)
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
