use lsp_types::CompletionParams;

use crate::features::cursor::CursorContext;

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_begin_command(
    context: &CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem>,
) -> Option<()> {
    let range = context.cursor.command_range(context.offset)?;

    items.push(InternalCompletionItem::new(
        range,
        InternalCompletionItemData::BeginCommand,
    ));
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
        complete_begin_command(&context, &mut actual_items);

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
        complete_begin_command(&context, &mut actual_items);

        assert!(actual_items.is_empty());
    }

    #[test]
    fn test_latex_simple() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\")])
            .main("main.tex")
            .line(0)
            .character(1)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_begin_command(&context, &mut actual_items);

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(1.into(), 1.into()));
        }
    }
}
