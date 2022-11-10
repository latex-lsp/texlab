use rowan::{ast::AstNode, TextRange};

use crate::{
    syntax::{bibtex, latex},
    util::cursor::CursorContext,
};

use super::builder::CompletionBuilder;

pub fn complete<'db>(
    context: &'db CursorContext,
    builder: &mut CompletionBuilder<'db>,
) -> Option<()> {
    let token = context.cursor.as_tex()?;

    let range = if token.kind() == latex::WORD {
        latex::Key::cast(token.parent()?)
            .map(|key| latex::small_range(&key))
            .or_else(|| {
                token
                    .parent()
                    .and_then(latex::Text::cast)
                    .map(|text| latex::small_range(&text))
            })?
    } else {
        TextRange::empty(context.offset)
    };

    check_citation(context).or_else(|| check_acronym(context))?;
    for document in context.related() {
        if let Some(data) = document.parse(context.db).as_bib() {
            for entry in data
                .root(context.db)
                .children()
                .filter_map(bibtex::Entry::cast)
            {
                builder.citation(range, document, &entry);
            }
        }
    }

    Some(())
}

fn check_citation(context: &CursorContext) -> Option<()> {
    let (_, _, group) = context.find_curly_group_word_list()?;
    latex::Citation::cast(group.syntax().parent()?)?;
    Some(())
}

fn check_acronym(context: &CursorContext) -> Option<()> {
    let token = context.cursor.as_tex()?;

    let pair = token
        .parent_ancestors()
        .find_map(latex::KeyValuePair::cast)?;
    if pair.key()?.to_string() != "cite" {
        return None;
    }

    latex::AcronymDeclaration::cast(pair.syntax().parent()?.parent()?.parent()?)?;
    Some(())
}
