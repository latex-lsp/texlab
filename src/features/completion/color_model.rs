use lsp_types::CompletionParams;
use rowan::{ast::AstNode, TextRange};

use crate::{features::cursor::CursorContext, syntax::latex};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

const MODEL_NAMES: &[&str] = &["gray", "rgb", "RGB", "HTML", "cmyk"];

pub fn complete_color_models<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
) -> Option<()> {
    let range = check_color_definition(context).or_else(|| check_color_definition_set(context))?;

    for name in MODEL_NAMES {
        items.push(InternalCompletionItem::new(
            range,
            InternalCompletionItemData::ColorModel { name },
        ));
    }

    Some(())
}

fn check_color_definition(context: &CursorContext<CompletionParams>) -> Option<TextRange> {
    let (_, range, group) = context.find_curly_group_word()?;

    let definition = latex::ColorDefinition::cast(group.syntax().parent()?)?;
    definition
        .model()
        .filter(|model| model.syntax().text_range() == group.syntax().text_range())?;
    Some(range)
}

fn check_color_definition_set(context: &CursorContext<CompletionParams>) -> Option<TextRange> {
    let (_, range, group) = context.find_curly_group_word_list()?;
    let definition = latex::ColorSetDefinition::cast(group.syntax().parent()?)?;
    definition
        .model_list()
        .filter(|model| model.syntax().text_range() == group.syntax().text_range())?;
    Some(range)
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
        complete_color_models(&context, &mut actual_items);

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
        complete_color_models(&context, &mut actual_items);

        assert!(actual_items.is_empty());
    }

    #[test]
    fn test_definition_simple() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\definecolor{foo}{}")])
            .main("main.tex")
            .line(0)
            .character(18)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_color_models(&context, &mut actual_items);

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(18.into(), 18.into()));
        }
    }

    #[test]
    fn test_definition_open_brace() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\definecolor{foo}{")])
            .main("main.tex")
            .line(0)
            .character(18)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_color_models(&context, &mut actual_items);

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(18.into(), 18.into()));
        }
    }

    #[test]
    fn test_definition_set_simple() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\definecolorset{}")])
            .main("main.tex")
            .line(0)
            .character(16)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_color_models(&context, &mut actual_items);

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(16.into(), 16.into()));
        }
    }

    #[test]
    fn test_definition_set_open_brace() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\definecolorset{")])
            .main("main.tex")
            .line(0)
            .character(16)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_color_models(&context, &mut actual_items);

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(16.into(), 16.into()));
        }
    }
}
