use cancellation::CancellationToken;
use cstree::TextRange;
use lsp_types::CompletionParams;

use crate::{
    component_db::COMPONENT_DATABASE,
    features::cursor::CursorContext,
    syntax::{latex, CstNode},
};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_component_environments<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
    cancellation_token: &CancellationToken,
) -> Option<()> {
    cancellation_token.result().ok()?;

    let token = context.cursor.as_latex()?;
    let group = latex::CurlyGroupWord::cast(token.parent())
        .filter(|group| context.is_inside_latex_curly(group))?;
    if !matches!(group.syntax().parent()?.kind(), latex::BEGIN | latex::END) {
        return None;
    }

    let range = if token.kind() == latex::WORD {
        token.text_range()
    } else {
        TextRange::empty(context.offset)
    };

    for component in COMPONENT_DATABASE.linked_components(&context.request.subset) {
        cancellation_token.result().ok()?;
        for name in &component.environments {
            items.push(InternalCompletionItem::new(
                range,
                InternalCompletionItemData::ComponentEnvironment {
                    name,
                    file_names: &component.file_names,
                },
            ));
        }
    }

    Some(())
}

#[cfg(test)]
mod tests {
    use cstree::TextRange;

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
        complete_component_environments(&context, &mut actual_items, CancellationToken::none());

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
        complete_component_environments(&context, &mut actual_items, CancellationToken::none());

        assert!(actual_items.is_empty());
    }

    #[test]
    fn test_simple() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\begin{")])
            .main("main.tex")
            .line(0)
            .character(7)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_component_environments(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(7.into(), 7.into()));
        }
    }

    #[test]
    fn test_simple_end() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\begin{a}\n\\end{")])
            .main("main.tex")
            .line(1)
            .character(5)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_component_environments(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(15.into(), 15.into()));
        }
    }

    #[test]
    fn test_simple_class() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\begin{}\n\\documentclass{article}")])
            .main("main.tex")
            .line(0)
            .character(7)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_component_environments(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in &actual_items {
            assert_eq!(item.range, TextRange::new(7.into(), 7.into()));
        }

        assert!(actual_items
            .iter()
            .any(|item| item.data.label() == "theindex"));
    }

    #[test]
    fn test_simple_existing() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\begin{d}")])
            .main("main.tex")
            .line(0)
            .character(7)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_component_environments(&context, &mut actual_items, CancellationToken::none());

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(7.into(), 8.into()));
        }
    }
}
