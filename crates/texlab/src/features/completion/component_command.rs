use crate::util::{components::COMPONENT_DATABASE, cursor::CursorContext};

use super::builder::CompletionBuilder;

pub fn complete<'db>(
    context: &'db CursorContext,
    builder: &mut CompletionBuilder<'db>,
) -> Option<()> {
    let range = context.cursor.command_range(context.offset)?;

    for component in COMPONENT_DATABASE.linked_components(&context.related) {
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
