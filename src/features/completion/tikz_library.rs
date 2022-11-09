use rowan::ast::AstNode;

use crate::{
    syntax::latex,
    util::{cursor::CursorContext, lang_data::LANGUAGE_DATA},
};

use super::builder::CompletionBuilder;

pub fn complete_tikz_libraries<'db>(
    context: &'db CursorContext,
    builder: &mut CompletionBuilder<'db>,
) -> Option<()> {
    let (_, range, group) = context.find_curly_group_word_list()?;

    let import = latex::TikzLibraryImport::cast(group.syntax().parent()?)?;

    if import.command()?.text() == "\\usepgflibrary" {
        for name in &LANGUAGE_DATA.pgf_libraries {
            builder.tikz_library(range, name);
        }
    } else {
        for name in &LANGUAGE_DATA.tikz_libraries {
            builder.tikz_library(range, name);
        }
    }

    Some(())
}
