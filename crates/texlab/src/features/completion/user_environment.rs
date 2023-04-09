use base_db::DocumentData;

use crate::util::cursor::CursorContext;

use super::builder::CompletionBuilder;

pub fn complete<'db>(
    context: &'db CursorContext,
    builder: &mut CompletionBuilder<'db>,
) -> Option<()> {
    let (name, range) = context.find_environment_name()?;

    for document in &context.related {
        let DocumentData::Tex(data) = &document.data else { continue };
        for name in data
            .semantics
            .environments
            .iter()
            .filter(|n| n.as_str() != name)
        {
            builder.user_environment(range, name);
        }
    }

    Some(())
}
