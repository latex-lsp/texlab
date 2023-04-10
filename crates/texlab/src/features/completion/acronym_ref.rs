use base_db::DocumentData;
use rowan::ast::AstNode;
use syntax::latex;

use crate::util::cursor::CursorContext;

use super::builder::CompletionBuilder;

pub fn complete<'db>(
    context: &'db CursorContext,
    builder: &mut CompletionBuilder<'db>,
) -> Option<()> {
    let (_, range, group) = context.find_curly_group_word()?;
    latex::AcronymReference::cast(group.syntax().parent()?)?;

    for document in &context.related {
        let DocumentData::Tex(data) = &document.data else { continue };

        for name in data
            .root_node()
            .descendants()
            .filter_map(latex::AcronymDefinition::cast)
            .filter_map(|node| node.name())
            .filter_map(|name| name.key())
        {
            builder.glossary_entry(range, name.to_string());
        }
    }

    Some(())
}
