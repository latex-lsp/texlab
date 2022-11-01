use once_cell::sync::Lazy;
use regex::Regex;
use rowan::{ast::AstNode, TextRange};

use crate::{
    db::document::Document,
    features::lsp_kinds::Structure,
    syntax::{
        bibtex::{self, HasName, HasType},
        latex,
    },
    util::cursor::CursorContext,
    BibtexEntryTypeCategory, Db, LANGUAGE_DATA,
};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_citations<'db>(
    context: &'db CursorContext,
    items: &mut Vec<InternalCompletionItem<'db>>,
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
                if let Some(item) = make_item(context.db, document, &entry, range) {
                    items.push(item);
                }
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

fn make_item(
    db: &dyn Db,
    document: Document,
    entry: &bibtex::Entry,
    range: TextRange,
) -> Option<InternalCompletionItem<'static>> {
    let key = entry.name_token()?.to_string();
    let ty = LANGUAGE_DATA
        .find_entry_type(&entry.type_token()?.text()[1..])
        .map_or_else(
            || Structure::Entry(BibtexEntryTypeCategory::Misc),
            |ty| Structure::Entry(ty.category),
        );

    let entry_code = entry.syntax().text().to_string();
    let text = format!(
        "{} {}",
        key,
        WHITESPACE_REGEX
            .replace_all(
                &entry_code
                    .replace('{', " ")
                    .replace('}', " ")
                    .replace(',', " ")
                    .replace('=', " "),
                " "
            )
            .trim(),
    );

    Some(InternalCompletionItem::new(
        range,
        InternalCompletionItemData::Citation {
            uri: document.location(db).uri(db).clone(),
            key,
            text,
            ty,
        },
    ))
}

static WHITESPACE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("\\s+").unwrap());
