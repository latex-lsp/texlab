use cancellation::CancellationToken;
use lsp_types::{GotoDefinitionParams, LocationLink};

use crate::{
    features::FeatureRequest,
    syntax::{bibtex, latex, CstNode},
    LineIndexExt,
};

pub fn goto_entry_definition(
    request: &FeatureRequest<GotoDefinitionParams>,
    token: &CancellationToken,
) -> Option<Vec<LocationLink>> {
    let main_document = request.main_document();

    let offset = main_document
        .line_index
        .offset_lsp(request.params.text_document_position_params.position);

    let key = main_document
        .data
        .as_latex()?
        .root
        .token_at_offset(offset)
        .right_biased()?;
    if key.kind() != latex::WORD {
        return None;
    }
    latex::Citation::cast(key.parent().parent()?)?;

    let origin_selection_range = main_document
        .line_index
        .line_col_lsp_range(key.text_range());

    for document in &request.subset.documents {
        if let Some(data) = document.data.as_bibtex() {
            for entry in data.root.children().filter_map(bibtex::Entry::cast) {
                if token.is_canceled() {
                    return None;
                }

                if let Some(key) = entry.key().filter(|k| k.text() == key.text()) {
                    return Some(vec![LocationLink {
                        origin_selection_range: Some(origin_selection_range),
                        target_uri: document.uri.as_ref().clone().into(),
                        target_selection_range: document
                            .line_index
                            .line_col_lsp_range(key.text_range()),
                        target_range: document.line_index.line_col_lsp_range(entry.small_range()),
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

        let actual_links = goto_entry_definition(&request, CancellationToken::none());

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

        let actual_links = goto_entry_definition(&request, CancellationToken::none());

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
        let target_uri = tester.uri("baz.bib").as_ref().clone().into();

        let request = tester.definition();
        let actual_links = goto_entry_definition(&request, CancellationToken::none()).unwrap();

        let expected_links = vec![LocationLink {
            origin_selection_range: Some(Range::new_simple(1, 6, 1, 9)),
            target_uri,
            target_range: Range::new_simple(0, 0, 0, 26),
            target_selection_range: Range::new_simple(0, 9, 0, 12),
        }];

        assert_eq!(actual_links, expected_links);
    }
}
