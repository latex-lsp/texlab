use lsp_types::{GotoDefinitionParams, LocationLink};
use rowan::ast::AstNode;

use crate::{
    features::cursor::CursorContext,
    syntax::{
        bibtex::{self, HasName},
        latex,
    },
    LineIndexExt,
};

pub fn goto_entry_definition(
    context: &CursorContext<GotoDefinitionParams>,
) -> Option<Vec<LocationLink>> {
    let main_document = context.request.main_document();

    let word = context
        .cursor
        .as_latex()
        .filter(|token| token.kind() == latex::WORD)?;

    let key = latex::Key::cast(word.parent()?)?;

    latex::Citation::cast(key.syntax().parent()?.parent()?)?;

    let origin_selection_range = main_document
        .line_index
        .line_col_lsp_range(latex::small_range(&key));

    for document in context.request.workspace.documents_by_uri.values() {
        if let Some(data) = document.data.as_bibtex() {
            for entry in bibtex::SyntaxNode::new_root(data.green.clone())
                .children()
                .filter_map(bibtex::Entry::cast)
            {
                if let Some(key) = entry.name_token().filter(|k| k.text() == word.text()) {
                    return Some(vec![LocationLink {
                        origin_selection_range: Some(origin_selection_range),
                        target_uri: document.uri.as_ref().clone(),
                        target_selection_range: document
                            .line_index
                            .line_col_lsp_range(key.text_range()),
                        target_range: document
                            .line_index
                            .line_col_lsp_range(entry.syntax().text_range()),
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
        let actual_links = goto_entry_definition(&context);

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
        let actual_links = goto_entry_definition(&context);

        assert!(actual_links.is_none());
    }

    #[test]
    fn test_simple() {
        let tester = FeatureTester::builder()
            .files(vec![
                (
                    "foo.tex",
                    indoc!(
                        r#"
                            \addbibresource{baz.bib}
                            \cite{foo}
                        "#
                    ),
                ),
                ("bar.bib", r#"@article{foo, bar = {baz}}"#),
                ("baz.bib", r#"@article{foo, bar = {baz}}"#),
            ])
            .main("foo.tex")
            .line(1)
            .character(6)
            .build();
        let target_uri = tester.uri("baz.bib").as_ref().clone();

        let request = tester.definition();
        let context = CursorContext::new(request);
        let actual_links = goto_entry_definition(&context).unwrap();

        let expected_links = vec![LocationLink {
            origin_selection_range: Some(Range::new_simple(1, 6, 1, 9)),
            target_uri,
            target_range: Range::new_simple(0, 0, 0, 26),
            target_selection_range: Range::new_simple(0, 9, 0, 12),
        }];

        assert_eq!(actual_links, expected_links);
    }
}
