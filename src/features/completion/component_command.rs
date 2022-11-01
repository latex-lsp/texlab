use crate::{component_db::COMPONENT_DATABASE, util::cursor::CursorContext};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_component_commands<'db>(
    context: &'db CursorContext,
    items: &mut Vec<InternalCompletionItem<'db>>,
) -> Option<()> {
    let range = context.cursor.command_range(context.offset)?;

    for component in COMPONENT_DATABASE.linked_components(context.db, context.document) {
        for command in &component.commands {
            items.push(InternalCompletionItem::new(
                range,
                InternalCompletionItemData::ComponentCommand {
                    name: &command.name,
                    image: command.image.as_deref(),
                    glyph: command.glyph.as_deref(),
                    file_names: &component.file_names,
                },
            ));
        }
    }

    Some(())
}
