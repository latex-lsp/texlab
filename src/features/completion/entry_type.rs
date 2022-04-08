use cancellation::CancellationToken;
use lsp_types::CompletionParams;
use rowan::{TextRange, TextSize};

use crate::{features::cursor::CursorContext, LANGUAGE_DATA};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_entry_types<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
    cancellation_token: &CancellationToken,
) -> Option<()> {
    if cancellation_token.is_canceled() {
        return None;
    }

    let range = context
        .cursor
        .as_bibtex()
        .filter(|token| token.kind().is_type())
        .filter(|token| token.text_range().start() != context.offset)
        .map(|token| token.text_range())
        .map(|range| TextRange::new(range.start() + TextSize::from(1), range.end()))?;

    for ty in &LANGUAGE_DATA.entry_types {
        let data = InternalCompletionItemData::EntryType { ty };
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
        complete_entry_types(&context, &mut actual_items, CancellationToken::none());

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
        complete_entry_types(&context, &mut actual_items, CancellationToken::none());

        assert!(actual_items.is_empty());
    }

    #[test]
    fn test_at_empty() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "@")])
            .main("main.bib")
            .line(0)
            .character(1)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_entry_types(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(1.into(), 1.into()));
        }
    }

    #[test]
    fn test_before_preamble() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "@preamble")])
            .main("main.bib")
            .line(0)
            .character(1)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_entry_types(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(1.into(), 9.into()));
        }
    }

    #[test]
    fn test_before_string() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "@string")])
            .main("main.bib")
            .line(0)
            .character(1)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_entry_types(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(1.into(), 7.into()));
        }
    }

    #[test]
    fn test_before_article() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "@article")])
            .main("main.bib")
            .line(0)
            .character(1)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_entry_types(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(1.into(), 8.into()));
        }
    }

    #[test]
    fn test_after_preamble() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "@preamble{")])
            .main("main.bib")
            .line(0)
            .character(9)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_entry_types(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(1.into(), 9.into()));
        }
    }

    #[test]
    fn test_after_string() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "@string{")])
            .main("main.bib")
            .line(0)
            .character(7)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_entry_types(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(1.into(), 7.into()));
        }
    }

    #[test]
    fn test_complete_entry() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "@article{foo, author = {foo}}")])
            .main("main.bib")
            .line(0)
            .character(3)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_entry_types(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(1.into(), 8.into()));
        }
    }

    #[test]
    fn test_complete_entry_field() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "@article{foo, author = {foo}}")])
            .main("main.bib")
            .line(0)
            .character(17)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_entry_types(&context, &mut actual_items, CancellationToken::none());

        assert!(actual_items.is_empty());
    }
}
