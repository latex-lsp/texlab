use crate::util::cursor::CursorContext;

use super::builder::CompletionBuilder;

pub fn complete<'db>(
    context: &'db CursorContext,
    builder: &mut CompletionBuilder<'db>,
) -> Option<()> {
    let range = context.find_environment_name()?;

    for package in context.included_packages() {
        for name in &package.environments {
            builder.component_environment(range, name, &package.file_names);
        }
    }

    Some(())
}
