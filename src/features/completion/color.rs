use rowan::ast::AstNode;

use crate::{
    syntax::latex,
    util::{cursor::CursorContext, lang_data::LANGUAGE_DATA},
};

use super::builder::CompletionBuilder;

pub fn complete<'db>(
    context: &'db CursorContext,
    builder: &mut CompletionBuilder<'db>,
) -> Option<()> {
    let (_, range, group) = context.find_curly_group_word()?;
    latex::ColorReference::cast(group.syntax().parent()?)?;

    for name in &LANGUAGE_DATA.colors {
        builder.color(range, name);
    }

    Some(())
}
