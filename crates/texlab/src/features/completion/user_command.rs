use crate::util::cursor::CursorContext;

use super::builder::CompletionBuilder;

pub fn complete<'db>(
    context: &'db CursorContext,
    builder: &mut CompletionBuilder<'db>,
) -> Option<()> {
    let range = context.cursor.command_range(context.offset)?;
    let token = context.cursor.as_tex()?;

    let db = context.db;
    for document in context.related() {
        if let Some(data) = document.parse(db).as_tex() {
            let text = document.text(db);
            for name in data
                .analyze(db)
                .command_name_ranges(db)
                .iter()
                .copied()
                .filter(|range| *range != token.text_range())
                .map(|range| &text[std::ops::Range::<usize>::from(range)])
            {
                builder.user_command(range, name);
            }
        }
    }

    Some(())
}
