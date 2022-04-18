use lsp_types::CompletionParams;

use crate::features::cursor::CursorContext;

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_theorem_environments<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
) -> Option<()> {
    let (_, range) = context.find_environment_name()?;

    for document in context.request.workspace.documents_by_uri.values() {
        if let Some(data) = document.data.as_latex() {
            for environment in &data.extras.theorem_environments {
                items.push(InternalCompletionItem::new(
                    range,
                    InternalCompletionItemData::UserEnvironment {
                        name: &environment.name,
                    },
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
        complete_theorem_environments(&context, &mut actual_items);

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
        complete_theorem_environments(&context, &mut actual_items);

        assert!(actual_items.is_empty());
    }

    #[test]
    fn test_simple() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\begin{ \\newtheorem{lemma}{Lemma}")])
            .main("main.tex")
            .line(0)
            .character(7)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_theorem_environments(&context, &mut actual_items);

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(7.into(), 7.into()));
        }
    }

    #[test]
    fn test_simple_end() {
        let request = FeatureTester::builder()
            .files(vec![(
                "main.tex",
                "\\newtheorem{lemma}{Lemma}\\begin{a}\n\\end{",
            )])
            .main("main.tex")
            .line(1)
            .character(5)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_theorem_environments(&context, &mut actual_items);

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(40.into(), 40.into()));
        }
    }

    #[test]
    fn test_simple_existing() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\begin{d}\\newtheorem{lemma}{Lemma}")])
            .main("main.tex")
            .line(0)
            .character(7)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_theorem_environments(&context, &mut actual_items);

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(7.into(), 8.into()));
        }
    }
}
