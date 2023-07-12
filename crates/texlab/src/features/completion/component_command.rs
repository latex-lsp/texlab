use crate::util::cursor::CursorContext;

use super::builder::CompletionBuilder;

pub fn complete<'db>(
    context: &'db CursorContext,
    builder: &mut CompletionBuilder<'db>,
) -> Option<()> {
    let range = context.cursor.command_range(context.offset)?;

    for package in context.included_packages() {
        for command in &package.commands {
            builder.component_command(
                range,
                &command.name,
                command.image.as_deref(),
                command.glyph.as_deref(),
                &package.file_names,
            );
        }
    }

    Some(())
}
