use lsp_types::{DocumentLink, DocumentLinkParams};

use crate::LineIndexExt;

use super::FeatureRequest;

pub fn find_document_links(request: FeatureRequest<DocumentLinkParams>) -> Vec<DocumentLink> {
    let mut links = Vec::new();
    let main_document = request.main_document();
    if let Some(data) = main_document.data.as_latex() {
        for include in &data.extras.explicit_links {
            for target in &include.targets {
                if request
                    .workspace
                    .documents_by_uri
                    .values()
                    .any(|document| document.uri.as_ref() == target.as_ref())
                {
                    links.push(DocumentLink {
                        range: main_document
                            .line_index
                            .line_col_lsp_range(include.stem_range),
                        target: Some(target.as_ref().clone().into()),
                        tooltip: None,
                        data: None,
                    });
                    break;
                }
            }
        }
    }
    links
}

#[cfg(test)]
mod tests {
    use super::*;

    use lsp_types::Range;

    use crate::{features::testing::FeatureTester, RangeExt};

    #[test]
    fn test_empty_latex_document() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", "")])
            .main("main.tex")
            .build()
            .link();

        let items = find_document_links(request);
        assert!(items.is_empty());
    }

    #[test]
    fn test_empty_bibtex_document() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "")])
            .main("main.bib")
            .build()
            .link();

        let items = find_document_links(request);
        assert!(items.is_empty());
    }

    #[test]
    fn test_includes() {
        let tester = FeatureTester::builder()
            .files(vec![("foo.tex", r#"\input{bar.tex}"#), ("bar.tex", r#""#)])
            .main("foo.tex")
            .build();
        let target = tester.uri("bar.tex");

        let actual_items = find_document_links(tester.link());

        let expected_items = vec![DocumentLink {
            range: Range::new_simple(0, 7, 0, 14),
            target: Some(target.as_ref().clone().into()),
            tooltip: None,
            data: None,
        }];
        assert_eq!(actual_items, expected_items);
    }

    #[test]
    fn test_imports() {
        let tester = FeatureTester::builder()
            .files(vec![
                ("foo.tex", r#"\import{bar/}{baz}"#),
                ("bar/baz.tex", r#""#),
            ])
            .main("foo.tex")
            .build();
        let target = tester.uri("bar/baz.tex");

        let actual_items = find_document_links(tester.link());

        let expected_items = vec![DocumentLink {
            range: Range::new_simple(0, 14, 0, 17),
            target: Some(target.as_ref().clone().into()),
            tooltip: None,
            data: None,
        }];
        assert_eq!(actual_items, expected_items);
    }
}
