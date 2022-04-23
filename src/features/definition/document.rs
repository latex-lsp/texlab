use lsp_types::{GotoDefinitionParams, LocationLink, Range};

use crate::{features::cursor::CursorContext, LineIndexExt, RangeExt};

pub fn goto_document_definition(
    context: &CursorContext<GotoDefinitionParams>,
) -> Option<Vec<LocationLink>> {
    let main_document = context.request.main_document();
    if let Some(data) = main_document.data.as_latex() {
        for include in data
            .extras
            .explicit_links
            .iter()
            .filter(|link| link.stem_range.contains_inclusive(context.offset))
        {
            for target in &include.targets {
                if context
                    .request
                    .workspace
                    .documents_by_uri
                    .values()
                    .any(|document| document.uri.as_ref() == target.as_ref())
                {
                    return Some(vec![LocationLink {
                        origin_selection_range: Some(
                            main_document
                                .line_index
                                .line_col_lsp_range(include.stem_range),
                        ),
                        target_uri: target.as_ref().clone(),
                        target_range: Range::new_simple(0, 0, 0, 0),
                        target_selection_range: Range::new_simple(0, 0, 0, 0),
                    }]);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
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
        let actual_links = goto_document_definition(&context);

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
        let actual_links = goto_document_definition(&context);

        assert!(actual_links.is_none());
    }

    #[test]
    fn test_simple() {
        let tester = FeatureTester::builder()
            .files(vec![
                ("foo.tex", r#"\addbibresource{baz.bib}"#),
                ("bar.bib", r#"@article{foo, bar = {baz}}"#),
                ("baz.bib", r#"@article{foo, bar = {baz}}"#),
            ])
            .main("foo.tex")
            .line(0)
            .character(18)
            .build();
        let target_uri = tester.uri("baz.bib").as_ref().clone();

        let request = tester.definition();
        let context = CursorContext::new(request);
        let actual_links = goto_document_definition(&context).unwrap();

        let expected_links = vec![LocationLink {
            origin_selection_range: Some(Range::new_simple(0, 16, 0, 23)),
            target_uri,
            target_range: Range::new_simple(0, 0, 0, 0),
            target_selection_range: Range::new_simple(0, 0, 0, 0),
        }];

        assert_eq!(actual_links, expected_links);
    }
}
