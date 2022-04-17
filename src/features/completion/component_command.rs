use lsp_types::CompletionParams;

use crate::{component_db::COMPONENT_DATABASE, features::cursor::CursorContext};

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_component_commands<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
) -> Option<()> {
    let range = context.cursor.command_range(context.offset)?;

    for component in COMPONENT_DATABASE.linked_components(&context.request.subset) {
        for command in &component.commands {
            items.push(InternalCompletionItem::new(
                range,
                InternalCompletionItemData::ComponentCommand {
                    name: &command.name,
                    image: command.image.as_deref(),
                    glyph: command.glyph.as_deref(),
                    file_names: &component.file_names,
                },
            ));
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
        complete_component_commands(&context, &mut actual_items);

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
        complete_component_commands(&context, &mut actual_items);

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
        complete_component_commands(&context, &mut actual_items);

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(1.into(), 1.into()));
        }
    }

    #[test]
    fn test_latex_simple_before() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\")])
            .main("main.tex")
            .line(0)
            .character(0)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_component_commands(&context, &mut actual_items);

        assert!(actual_items.is_empty());
    }

    #[test]
    fn test_latex_simple_package() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\\n\\usepackage{lipsum}")])
            .main("main.tex")
            .line(0)
            .character(1)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_component_commands(&context, &mut actual_items);

        assert!(!actual_items.is_empty());
        for item in &actual_items {
            assert_eq!(item.range, TextRange::new(1.into(), 1.into()));
        }

        assert!(actual_items
            .iter()
            .any(|item| item.data.label() == "lipsum"));
    }

    #[test]
    fn test_latex_simple_existing() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "\\foo")])
            .main("main.tex")
            .line(0)
            .character(2)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_component_commands(&context, &mut actual_items);

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(1.into(), 4.into()));
        }
    }

    #[test]
    fn test_bibtex_simple() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "@a{b,c={\\ }}")])
            .main("main.bib")
            .line(0)
            .character(9)
            .build()
            .completion();

        let context = CursorContext::new(request);
        let mut actual_items = Vec::new();
        complete_component_commands(&context, &mut actual_items);

        assert!(!actual_items.is_empty());
        for item in actual_items {
            assert_eq!(item.range, TextRange::new(9.into(), 10.into()));
        }
    }
}
