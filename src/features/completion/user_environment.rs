use crate::util::cursor::CursorContext;

use super::builder::CompletionBuilder;

pub fn complete<'db>(
    context: &'db CursorContext,
    builder: &mut CompletionBuilder<'db>,
) -> Option<()> {
    let (name, range) = context.find_environment_name()?;

    for document in context.related() {
        if let Some(data) = document.parse(context.db).as_tex() {
            for name in data
                .analyze(context.db)
                .environment_names(context.db)
                .iter()
                .filter(|n| n.as_str() != name)
            {
                builder.user_environment(range, name);
            }
        }
    }

    Some(())
}
