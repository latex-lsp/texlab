use rowan::{ast::AstNode, TextRange};
use syntax::latex;

use crate::util::cursor::CursorContext;

use super::builder::CompletionBuilder;

const MODEL_NAMES: &[&str] = &["gray", "rgb", "RGB", "HTML", "cmyk"];

pub fn complete<'db>(
    context: &'db CursorContext,
    builder: &mut CompletionBuilder<'db>,
) -> Option<()> {
    let range = check_color_definition(context).or_else(|| check_color_definition_set(context))?;

    for name in MODEL_NAMES {
        builder.color_model(range, name);
    }

    Some(())
}

fn check_color_definition(context: &CursorContext) -> Option<TextRange> {
    let (_, range, group) = context.find_curly_group_word()?;

    let definition = latex::ColorDefinition::cast(group.syntax().parent()?)?;
    definition
        .model()
        .filter(|model| model.syntax().text_range() == group.syntax().text_range())?;
    Some(range)
}

fn check_color_definition_set(context: &CursorContext) -> Option<TextRange> {
    let (_, range, group) = context.find_curly_group_word_list()?;
    let definition = latex::ColorSetDefinition::cast(group.syntax().parent()?)?;
    definition
        .model_list()
        .filter(|model| model.syntax().text_range() == group.syntax().text_range())?;
    Some(range)
}
