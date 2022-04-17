use lsp_types::CompletionParams;
use rowan::ast::AstNode;

use crate::{features::cursor::CursorContext, syntax::latex, LANGUAGE_DATA};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_colors<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
) -> Option<()> {
    let (_, range, group) = context.find_curly_group_word()?;
    latex::ColorReference::cast(group.syntax().parent()?)?;

    for name in &LANGUAGE_DATA.colors {
        items.push(InternalCompletionItem::new(
            range,
            InternalCompletionItemData::Color { name },
        ));
    }

    Some(())
}

#[cfg(test)]
mod tests {
    use rowan::TextRange;

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
        complete_colors(&context, &mut actual_items);

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
        complete_colors(&context, &mut actual_items);

        assert!(actual_items.is_empty());
    }

    #[test]
    fn test_latex_simple() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\color{}")])
            .main("main.tex")
            .line(0)
            .character(7)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_colors(&context, &mut actual_items);

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(7.into(), 7.into()));
        }
    }

    #[test]
    fn test_latex_open_brace() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\color{")])
            .main("main.tex")
            .line(0)
            .character(7)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_colors(&context, &mut actual_items);

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(7.into(), 7.into()));
        }
    }
}
