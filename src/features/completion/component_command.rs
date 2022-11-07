use crate::{component_db::COMPONENT_DATABASE, util::cursor::CursorContext};

use super::builder::CompletionBuilder;

pub fn complete_component_commands<'db>(
    context: &'db CursorContext,
    builder: &mut CompletionBuilder<'db>,
) -> Option<()> {
    let range = context.cursor.command_range(context.offset)?;

    for component in COMPONENT_DATABASE.linked_components(context.db, context.document) {
        for command in &component.commands {
            builder.component_command(
                range,
                &command.name,
                command.image.as_deref(),
                command.glyph.as_deref(),
                &component.file_names,
            );
        }
    }

    Some(())
}
