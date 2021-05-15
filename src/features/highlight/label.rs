use cancellation::CancellationToken;
use lsp_types::{DocumentHighlight, DocumentHighlightKind, DocumentHighlightParams};

use crate::{
    features::FeatureRequest,
    syntax::{latex, CstNode},
    LineIndexExt,
};

pub fn find_label_highlights(
    request: &FeatureRequest<DocumentHighlightParams>,
    token: &CancellationToken,
) -> Option<Vec<DocumentHighlight>> {
    let main_document = request.main_document();

    let offset = main_document
        .line_index
        .offset_lsp(request.params.text_document_position_params.position);

    let data = main_document.data.as_latex()?;
    let name = data.root.token_at_offset(offset).right_biased()?;
    if name.kind() != latex::WORD {
        return None;
    }

    if !matches!(
        name.parent().parent()?.kind(),
        latex::LABEL_DEFINITION | latex::LABEL_REFERENCE | latex::LABEL_REFERENCE_RANGE
    ) {
        return None;
    }

    let mut highlights = Vec::new();
    for node in data.root.descendants() {
        if token.is_canceled() {
            return None;
        }

        if let Some(label_name) = latex::LabelDefinition::cast(node)
            .and_then(|label| label.name())
            .and_then(|label_name| label_name.word())
            .filter(|label_name| label_name.text() == name.text())
        {
            let range = main_document
                .line_index
                .line_col_lsp_range(label_name.text_range());

            highlights.push(DocumentHighlight {
                range,
                kind: Some(DocumentHighlightKind::Write),
            });
        } else if let Some(label) = latex::LabelReference::cast(node) {
            for label_name in label
                .name_list()
                .into_iter()
                .flat_map(|name| name.words())
                .filter(|label_name| label_name.text() == name.text())
            {
                let range = main_document
                    .line_index
                    .line_col_lsp_range(label_name.text_range());

                highlights.push(DocumentHighlight {
                    range,
                    kind: Some(DocumentHighlightKind::Read),
                });
            }
        } else if let Some(label) = latex::LabelReferenceRange::cast(node) {
            if let Some(label_name) = label
                .from()
                .and_then(|label_name| label_name.word())
                .filter(|label_name| label_name.text() == name.text())
            {
                let range = main_document
                    .line_index
                    .line_col_lsp_range(label_name.text_range());

                highlights.push(DocumentHighlight {
                    range,
                    kind: Some(DocumentHighlightKind::Read),
                });
            }

            if let Some(label_name) = label
                .to()
                .and_then(|label_name| label_name.word())
                .filter(|label_name| label_name.text() == name.text())
            {
                let range = main_document
                    .line_index
                    .line_col_lsp_range(label_name.text_range());

                highlights.push(DocumentHighlight {
                    range,
                    kind: Some(DocumentHighlightKind::Read),
                });
            }
        }
    }

    Some(highlights)
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
            .highlight();

        let actual_links = find_label_highlights(&request, CancellationToken::none());

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
            .highlight();

        let actual_links = find_label_highlights(&request, CancellationToken::none());

        assert!(actual_links.is_none());
    }

    #[test]
    fn test_label() {
        let tester = FeatureTester::builder()
            .files(vec![("main.tex", "\\label{foo}\n\\ref{foo}\\label{bar}")])
            .main("main.tex")
            .line(0)
            .character(7)
            .build();
        let request = tester.highlight();

        let actual_highlights = find_label_highlights(&request, CancellationToken::none()).unwrap();

        let expected_highlights = vec![
            DocumentHighlight {
                range: Range::new_simple(0, 7, 0, 10),
                kind: Some(DocumentHighlightKind::Write),
            },
            DocumentHighlight {
                range: Range::new_simple(1, 5, 1, 8),
                kind: Some(DocumentHighlightKind::Read),
            },
        ];

        assert_eq!(actual_highlights, expected_highlights);
    }
}
