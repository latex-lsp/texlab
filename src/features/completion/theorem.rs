use crate::util::cursor::CursorContext;

use super::builder::CompletionBuilder;

pub fn complete_theorem_environments<'db>(
    context: &'db CursorContext,
    builder: &mut CompletionBuilder<'db>,
) -> Option<()> {
    let (_, range) = context.find_environment_name()?;

    let db = context.db;
    for document in context.related() {
        if let Some(data) = document.parse(db).as_tex() {
            for environment in data.analyze(db).theorem_environments(db) {
                builder.user_environment(range, environment.name(db).text(db));
            }
        }
    }

    Some(())
}
