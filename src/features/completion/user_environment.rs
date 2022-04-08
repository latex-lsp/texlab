use cancellation::CancellationToken;
use lsp_types::CompletionParams;

use crate::features::cursor::CursorContext;

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_user_environments<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
    cancellation_token: &CancellationToken,
) -> Option<()> {
    cancellation_token.result().ok()?;

    let (name, range) = context.find_environment_name()?;

    for document in &context.request.subset.documents {
        if let Some(data) = document.data.as_latex() {
            for name in data
                .extras
                .environment_names
                .iter()
                .filter(|n| n.as_str() != name)
            {
                cancellation_token.result().ok()?;
                items.push(InternalCompletionItem::new(
                    range,
                    InternalCompletionItemData::UserEnvironment { name },
                ));
            }
        }
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
        complete_user_environments(&context, &mut actual_items, CancellationToken::none());

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
        complete_user_environments(&context, &mut actual_items, CancellationToken::none());

        assert!(actual_items.is_empty());
    }

    #[test]
    fn test_latex_simple() {
        let request = FeatureTester::builder()
            .files(vec![(
                "main.tex",
                "\\begin{foo}\\end{foo} \\begin{bar}\\end{bar}",
            )])
            .main("main.tex")
            .line(0)
            .character(7)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_user_environments(&context, &mut actual_items, CancellationToken::none());

        assert_eq!(actual_items.len(), 1);
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(7.into(), 10.into()));
        }
    }
}
