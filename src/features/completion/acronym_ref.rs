use rowan::ast::AstNode;

use crate::{syntax::latex, util::cursor::CursorContext};

use super::builder::CompletionBuilder;

pub fn complete<'db>(
    context: &'db CursorContext,
    builder: &mut CompletionBuilder<'db>,
) -> Option<()> {
    let (_, range, group) = context.find_curly_group_word()?;
    latex::AcronymReference::cast(group.syntax().parent()?)?;

    for document in context.related() {
        if let Some(data) = document.parse(context.db).as_tex() {
            for name in data
                .root(context.db)
                .descendants()
                .filter_map(latex::AcronymDefinition::cast)
                .filter_map(|node| node.name())
                .filter_map(|name| name.key())
            {
                builder.glossary_entry(range, name.to_string());
            }
        }
    }

    Some(())
}
