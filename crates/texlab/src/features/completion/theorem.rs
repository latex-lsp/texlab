use base_db::DocumentData;

use crate::util::cursor::CursorContext;

use super::builder::CompletionBuilder;

pub fn complete<'db>(
    context: &'db CursorContext,
    builder: &mut CompletionBuilder<'db>,
) -> Option<()> {
    let range = context.find_environment_name()?;

    for document in &context.project.documents {
        let DocumentData::Tex(data) = &document.data else { continue };
        for theorem in &data.semantics.theorem_definitions {
            builder.user_environment(range, &theorem.name.text);
        }
    }

    Some(())
}
