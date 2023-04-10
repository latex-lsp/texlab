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
    latex::GlossaryEntryReference::cast(group.syntax().parent()?)?;

    for document in &context.project.documents {
        let DocumentData::Tex(data) = &document.data else { continue };

        for node in data.root_node().descendants() {
            if let Some(name) = latex::GlossaryEntryDefinition::cast(node.clone())
                .and_then(|entry| entry.name())
                .and_then(|name| name.key())
                .map(|name| name.to_string())
            {
                builder.glossary_entry(range, name);
            } else if let Some(name) = latex::AcronymDefinition::cast(node)
                .and_then(|entry| entry.name())
                .and_then(|name| name.key())
                .map(|name| name.to_string())
            {
                builder.glossary_entry(range, name);
            }
        }
    }

    Some(())
}
