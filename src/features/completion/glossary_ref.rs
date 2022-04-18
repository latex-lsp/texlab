use lsp_types::CompletionParams;
use rowan::ast::AstNode;

use crate::{features::cursor::CursorContext, syntax::latex};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_glossary_entries<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
) -> Option<()> {
    let (_, range, group) = context.find_curly_group_word()?;
    latex::GlossaryEntryReference::cast(group.syntax().parent()?)?;

    for document in context.request.workspace.documents_by_uri.values() {
        if let Some(data) = document.data.as_latex() {
            for node in latex::SyntaxNode::new_root(data.green.clone()).descendants() {
                if let Some(name) = latex::GlossaryEntryDefinition::cast(node.clone())
                    .and_then(|entry| entry.name())
                    .and_then(|name| name.key())
                    .map(|name| name.to_string())
                {
                    items.push(InternalCompletionItem::new(
                        range,
                        InternalCompletionItemData::GlossaryEntry { name },
                    ));
                } else if let Some(name) = latex::AcronymDefinition::cast(node)
                    .and_then(|entry| entry.name())
                    .and_then(|name| name.key())
                    .map(|name| name.to_string())
                {
                    items.push(InternalCompletionItem::new(
                        range,
                        InternalCompletionItemData::Acronym { name },
                    ));
                }
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
        complete_glossary_entries(&context, &mut actual_items);

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
        complete_glossary_entries(&context, &mut actual_items);

        assert!(actual_items.is_empty());
    }

    #[test]
    fn test_simple() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}\n\\gls{f}")])
            .main("main.tex")
            .line(1)
            .character(6)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_glossary_entries(&context, &mut actual_items);

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(82.into(), 83.into()));
        }
    }

    #[test]
    fn test_open_brace() {
        let request = FeatureTester::builder()
        .files(vec![("main.tex", "\\newacronym[longplural={Frames per Second}]{fpsLabel}{FPS}{Frame per Second}\n\\gls{f")])
        .main("main.tex")
        .line(1)
        .character(6)
        .build()
        .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_glossary_entries(&context, &mut actual_items);

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(82.into(), 83.into()));
        }
    }
}
