use cancellation::CancellationToken;
use lsp_types::CompletionParams;
use rowan::{ast::AstNode, TextRange};

use crate::{features::cursor::CursorContext, syntax::bibtex, LANGUAGE_DATA};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_fields<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
    cancellation_token: &CancellationToken,
) -> Option<()> {
    cancellation_token.result().ok()?;

    let token = context.cursor.as_bibtex()?;
    let range = if token.kind() == bibtex::WORD {
        token.text_range()
    } else {
        TextRange::empty(context.offset)
    };

    let parent = token.parent()?;
    if let Some(entry) = bibtex::Entry::cast(parent.clone()) {
        if bibtex::small_range(&entry.key()?) == token.text_range() {
            return None;
        }
    } else {
        bibtex::Field::cast(parent)?;
    }

    for field in &LANGUAGE_DATA.fields {
        let data = InternalCompletionItemData::Field { field };
        let item = InternalCompletionItem::new(range, data);
        items.push(item);
    }
    Some(())
}

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
        complete_fields(&context, &mut actual_items, CancellationToken::none());

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
        complete_fields(&context, &mut actual_items, CancellationToken::none());

        assert!(actual_items.is_empty());
    }

    #[test]
    fn test_empty_entry_open() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "@article{foo,")])
            .main("main.bib")
            .line(0)
            .character(13)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_fields(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(13.into(), 13.into()));
        }
    }

    #[test]
    fn test_empty_entry_closed() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "@article{foo,}")])
            .main("main.bib")
            .line(0)
            .character(13)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_fields(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(13.into(), 13.into()));
        }
    }

    #[test]
    fn test_entry_field_name() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "@article{foo, a")])
            .main("main.bib")
            .line(0)
            .character(15)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_fields(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(14.into(), 15.into()));
        }
    }

    #[test]
    fn test_entry_field_value() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "@article{foo, author = bar}")])
            .main("main.bib")
            .line(0)
            .character(24)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_fields(&context, &mut actual_items, CancellationToken::none());

        assert!(actual_items.is_empty());
    }

    #[test]
    fn test_entry_two_fields_name_closed() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "@article{foo, author = bar, baz}")])
            .main("main.bib")
            .line(0)
            .character(29)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_fields(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(28.into(), 31.into()));
        }
    }

    #[test]
    fn test_entry_two_fields_name_open() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "@article{foo, author = bar, baz")])
            .main("main.bib")
            .line(0)
            .character(29)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_fields(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(28.into(), 31.into()));
        }
    }
}
