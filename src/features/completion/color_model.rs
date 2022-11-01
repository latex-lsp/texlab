use rowan::{ast::AstNode, TextRange};

use crate::{syntax::latex, util::cursor::CursorContext};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

const MODEL_NAMES: &[&str] = &["gray", "rgb", "RGB", "HTML", "cmyk"];

pub fn complete_color_models<'db>(
    context: &'db CursorContext,
    items: &mut Vec<InternalCompletionItem<'db>>,
) -> Option<()> {
    let range = check_color_definition(context).or_else(|| check_color_definition_set(context))?;

    for name in MODEL_NAMES {
        items.push(InternalCompletionItem::new(
            range,
            InternalCompletionItemData::ColorModel { name },
        ));
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
