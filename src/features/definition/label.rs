use cancellation::CancellationToken;
use lsp_types::{GotoDefinitionParams, LocationLink};

use crate::{
    features::FeatureRequest, find_label_definition, render_label, syntax::latex, LineIndexExt,
};

pub fn goto_label_definition(
    request: &FeatureRequest<GotoDefinitionParams>,
    token: &CancellationToken,
) -> Option<Vec<LocationLink>> {
    let main_document = request.main_document();

    let offset = main_document
        .line_index
        .offset_lsp(request.params.text_document_position_params.position);

    let name = main_document
        .data
        .as_latex()?
        .root
        .token_at_offset(offset)
        .right_biased()?;
    if name.kind() != latex::WORD {
        return None;
    }

    if !matches!(
        name.parent().parent()?.kind(),
        latex::LABEL_DEFINITION | latex::LABEL_REFERENCE | latex::LABEL_REFERENCE_RANGE
    ) {
        return None;
    }

    let origin_selection_range = main_document
        .line_index
        .line_col_lsp_range(name.text_range());

    for document in &request.subset.documents {
        if token.is_canceled() {
            return None;
        }

        if let Some(data) = document.data.as_latex() {
            if let Some(definition) = find_label_definition(&data.root, name.text()) {
                let target_selection_range = definition.name()?.word()?.text_range();
                let target_range = render_label(&request.subset, name.text(), Some(definition))
                    .map(|label| label.range)
                    .unwrap_or(target_selection_range);

                return Some(vec![LocationLink {
                    origin_selection_range: Some(origin_selection_range),
                    target_uri: document.uri.as_ref().clone().into(),
                    target_range: document.line_index.line_col_lsp_range(target_range),
                    target_selection_range: document
                        .line_index
                        .line_col_lsp_range(target_selection_range),
                }]);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
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
            .definition();

        let actual_links = goto_label_definition(&request, CancellationToken::none());

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

        let actual_links = goto_label_definition(&request, CancellationToken::none());

        assert!(actual_links.is_none());
    }
}
