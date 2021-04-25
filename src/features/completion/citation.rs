use std::sync::Arc;

use cancellation::CancellationToken;
use cstree::TextRange;
use lsp_types::CompletionParams;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    features::{cursor::CursorContext, lsp_kinds::Structure},
    syntax::{
        bibtex::{self, HasType},
        latex, CstNode,
    },
    BibtexEntryTypeCategory, Document, LANGUAGE_DATA,
};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_citations<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
    cancellation_token: &CancellationToken,
) -> Option<()> {
    cancellation_token.result().ok()?;
    let token = context.cursor.as_latex()?;

    let range = if token.kind() == latex::WORD {
        token.text_range()
    } else {
        TextRange::empty(context.offset)
    };

    let group = latex::CurlyGroupWordList::cast(token.parent())
        .filter(|group| context.is_inside_latex_curly(group))?;
    latex::Citation::cast(group.syntax().parent()?)?;
    for document in &context.request.subset.documents {
        if let Some(data) = document.data.as_bibtex() {
            for entry in data.root.children().filter_map(bibtex::Entry::cast) {
                cancellation_token.result().ok()?;
                if let Some(item) = make_item(document, entry, range) {
                    items.push(item);
                }
            }
        }
    }

    Some(())
}

fn make_item<'a>(
    document: &'a Document,
    entry: bibtex::Entry<'a>,
    range: TextRange,
) -> Option<InternalCompletionItem<'a>> {
    let key = entry.key()?.text();
    let ty = LANGUAGE_DATA
        .find_entry_type(&entry.ty()?.text()[1..])
        .map(|ty| Structure::Entry(ty.category))
        .unwrap_or_else(|| Structure::Entry(BibtexEntryTypeCategory::Misc));

    let entry_code = entry.syntax().text().to_string();
    let text = format!(
        "{} {}",
        key,
        WHITESPACE_REGEX
            .replace_all(
                &entry_code
                    .replace('{', "")
                    .replace('}', "")
                    .replace(',', " ")
                    .replace("=", " "),
                " "
            )
            .trim(),
    );

    Some(InternalCompletionItem::new(
        range,
        InternalCompletionItemData::Citation {
            uri: Arc::clone(&document.uri),
            key: key.into(),
            text,
            ty,
        },
    ))
}

static WHITESPACE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("\\s+").unwrap());

#[cfg(test)]
mod tests {
    use crate::features::testing::FeatureTester;

    use super::*;

    #[test]
    fn test_empty_latex_document() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "")])
            .main("main.tex")
            .line(0)
            .character(0)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_citations(&context, &mut actual_items, CancellationToken::none());

        assert!(actual_items.is_empty());
    }

    #[test]
    fn test_empty_bibtex_document() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "")])
            .main("main.bib")
            .line(0)
            .character(0)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_citations(&context, &mut actual_items, CancellationToken::none());

        assert!(actual_items.is_empty());
    }

    #[test]
    fn test_latex_simple() {
        let request = FeatureTester::builder()
            .files(vec![
                ("main.tex", "\\addbibresource{main.bib}\n\\cite{}"),
                ("main.bib", "@article{foo,}"),
            ])
            .main("main.tex")
            .line(1)
            .character(6)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_citations(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(32.into(), 32.into()));
        }
    }

    #[test]
    fn test_latex_open_brace() {
        let request = FeatureTester::builder()
            .files(vec![
                ("main.tex", "\\addbibresource{main.bib}\n\\cite{"),
                ("main.bib", "@article{foo,}"),
            ])
            .main("main.tex")
            .line(1)
            .character(6)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_citations(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(32.into(), 32.into()));
        }
    }

    #[test]
    fn test_latex_open_brace_second() {
        let request = FeatureTester::builder()
            .files(vec![
                ("main.tex", "\\addbibresource{main.bib}\n\\cite{foo,a"),
                ("main.bib", "@article{foo,}"),
            ])
            .main("main.tex")
            .line(1)
            .character(10)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_citations(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(36.into(), 37.into()));
        }
    }
}
