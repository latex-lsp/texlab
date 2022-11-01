use rowan::ast::AstNode;

use crate::{syntax::latex, util::cursor::CursorContext, LANGUAGE_DATA};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_tikz_libraries<'db>(
    context: &'db CursorContext,
    items: &mut Vec<InternalCompletionItem<'db>>,
) -> Option<()> {
    let (_, range, group) = context.find_curly_group_word_list()?;

    let import = latex::TikzLibraryImport::cast(group.syntax().parent()?)?;

    if import.command()?.text() == "\\usepgflibrary" {
        for name in &LANGUAGE_DATA.pgf_libraries {
            items.push(InternalCompletionItem::new(
                range,
                InternalCompletionItemData::PgfLibrary { name },
            ));
        }
    } else {
        for name in &LANGUAGE_DATA.tikz_libraries {
            items.push(InternalCompletionItem::new(
                range,
                InternalCompletionItemData::TikzLibrary { name },
            ));
        }
    }

    Some(())
}
