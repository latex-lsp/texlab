use base_db::semantics::Span;
use rowan::ast::AstNode;
use syntax::latex;

use crate::{
    util::{find_curly_group_word, find_curly_group_word_list, CompletionBuilder},
    CompletionItem, CompletionItemData, CompletionParams,
};

pub fn complete_color_models<'a>(
    params: &'a CompletionParams<'a>,
    builder: &mut CompletionBuilder<'a>,
) -> Option<()> {
    let cursor = check_color_definition(params).or_else(|| check_color_definition_set(params))?;

    for name in MODEL_NAMES {
        if let Some(score) = builder.matcher.score(name, &cursor.text) {
            let data = CompletionItemData::ColorModel(name);
            builder
                .items
                .push(CompletionItem::new_simple(score, cursor.range, data));
        }
    }

    Some(())
}

fn check_color_definition(params: &CompletionParams) -> Option<Span> {
    let (span, group) = find_curly_group_word(params)?;

    let definition = latex::ColorDefinition::cast(group.syntax().parent()?)?;
    definition
        .model()
        .filter(|model| model.syntax().text_range() == group.syntax().text_range())?;

    Some(span)
}

fn check_color_definition_set(params: &CompletionParams) -> Option<Span> {
    let (span, group) = find_curly_group_word_list(params)?;

    let definition = latex::ColorSetDefinition::cast(group.syntax().parent()?)?;
    definition
        .model_list()
        .filter(|model| model.syntax().text_range() == group.syntax().text_range())?;

    Some(span)
}

const MODEL_NAMES: &[&str] = &["gray", "rgb", "RGB", "HTML", "cmyk"];
