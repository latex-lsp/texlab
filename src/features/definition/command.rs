use lsp_types::{GotoDefinitionParams, LocationLink};
use rowan::ast::AstNode;

use crate::{features::cursor::CursorContext, syntax::latex, LineIndexExt};

pub fn goto_command_definition(
    context: &CursorContext<GotoDefinitionParams>,
) -> Option<Vec<LocationLink>> {
    let main_document = context.request.main_document();

    let name = context
        .cursor
        .as_latex()
        .filter(|token| token.kind().is_command_name())?;

    let origin_selection_range = main_document
        .line_index
        .line_col_lsp_range(name.text_range());

    for document in &context.request.subset.documents {
        if let Some(data) = document.data.as_latex() {
            for node in latex::SyntaxNode::new_root(data.root.clone()).descendants() {
                if let Some(defintion) = latex::CommandDefinition::cast(node).filter(|def| {
                    def.name()
                        .and_then(|name| name.command())
                        .map_or(false, |node| node.text() == name.text())
                }) {
                    let target_selection_range = document
                        .line_index
                        .line_col_lsp_range(defintion.name()?.command()?.text_range());

                    let target_range = document
                        .line_index
                        .line_col_lsp_range(latex::small_range(&defintion));

                    return Some(vec![LocationLink {
                        origin_selection_range: Some(origin_selection_range),
                        target_uri: document.uri.as_ref().clone().into(),
                        target_range,
                        target_selection_range,
                    }]);
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use lsp_types::Range;

    use crate::{features::testing::FeatureTester, RangeExt};

    use super::*;

    #[test]
    fn test_empty_latex_document() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "")])
            .main("main.tex")
            .line(0)
            .character(0)
            .build()
            .definition();

        let context = CursorContext::new(request);

        let actual_links = goto_command_definition(&context);

        assert!(actual_links.is_none());
    }

    #[test]
    fn test_empty_bibtex_document() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "")])
            .main("main.bib")
            .line(0)
            .character(0)
            .build()
            .definition();

        let context = CursorContext::new(request);
        let actual_links = goto_command_definition(&context);

        assert!(actual_links.is_none());
    }

    #[test]
    fn test_command_definition() {
        let tester = FeatureTester::builder()
            .files(vec![(
                "main.tex",
                indoc! {
                    r#"
                        \DeclareMathOperator{\foo}{foo}
                        \foo
                    "#
                },
            )])
            .main("main.tex")
            .line(1)
            .character(2)
            .build();
        let target_uri = tester.uri("main.tex").as_ref().clone().into();

        let request = tester.definition();
        let context = CursorContext::new(request);
        let actual_links = goto_command_definition(&context).unwrap();

        let expected_links = vec![LocationLink {
            origin_selection_range: Some(Range::new_simple(1, 0, 1, 4)),
            target_uri,
            target_range: Range::new_simple(0, 0, 0, 31),
            target_selection_range: Range::new_simple(0, 21, 0, 25),
        }];

        assert_eq!(actual_links, expected_links);
    }
}
