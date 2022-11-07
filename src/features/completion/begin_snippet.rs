use crate::util::cursor::CursorContext;

use super::builder::CompletionBuilder;

pub fn complete_begin_snippet(
    context: &CursorContext,
    builder: &mut CompletionBuilder,
) -> Option<()> {
    let range = context.cursor.command_range(context.offset)?;
    builder.begin_snippet(range);
    Some(())
}
