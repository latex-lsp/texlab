use lsp_types::CompletionParams;

use crate::{component_db::COMPONENT_DATABASE, features::cursor::CursorContext};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_component_commands<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
) -> Option<()> {
    let range = context.cursor.command_range(context.offset)?;

    for component in COMPONENT_DATABASE.linked_components(&context.request.workspace) {
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
