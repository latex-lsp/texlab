use rowan::ast::AstNode;

use crate::{syntax::latex, util::cursor::CursorContext};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_glossary_entries<'db>(
    context: &'db CursorContext,
    items: &mut Vec<InternalCompletionItem<'db>>,
) -> Option<()> {
    let (_, range, group) = context.find_curly_group_word()?;
    latex::GlossaryEntryReference::cast(group.syntax().parent()?)?;

    for document in context
        .workspace
        .related(context.db, context.distro, context.document)
    {
        if let Some(data) = document.parse(context.db).as_tex() {
            for node in data.root(context.db).descendants() {
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
