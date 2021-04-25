use cancellation::CancellationToken;
use lsp_types::{Hover, HoverContents, HoverParams};

use crate::{features::FeatureRequest, render_label, syntax::latex, LineIndexExt};

pub fn find_label_hover(
    request: &FeatureRequest<HoverParams>,
    _token: &CancellationToken,
) -> Option<Hover> {
    let main_document = request.main_document();
    let data = main_document.data.as_latex()?;
    let offset = main_document
        .line_index
        .offset_lsp(request.params.text_document_position_params.position);

    let name = data
        .root
        .token_at_offset(offset)
        .right_biased()
        .filter(|token| token.kind() == latex::WORD)?;

    if !matches!(
        name.parent().parent()?.kind(),
        latex::LABEL_DEFINITION | latex::LABEL_REFERENCE | latex::LABEL_REFERENCE_RANGE
    ) {
        return None;
    }

    let label = render_label(&request.subset, name.text(), None)?;

    Some(Hover {
        range: Some(
            main_document
                .line_index
                .line_col_lsp_range(name.text_range()),
        ),
        contents: HoverContents::Markup(label.documentation()),
    })
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
            .hover();

        let actual_hover = find_label_hover(&request, CancellationToken::none());

        assert_eq!(actual_hover, None);
    }

    #[test]
    fn test_empty_bibtex_document() {
        let request = FeatureTester::builder()
            .files(vec![("main.bib", "")])
            .main("main.bib")
            .line(0)
            .character(0)
            .build()
            .hover();

        let actual_hover = find_label_hover(&request, CancellationToken::none());

        assert_eq!(actual_hover, None);
    }

    #[test]
    fn test_section() {
        let request = FeatureTester::builder()
            .files(vec![("main.tex", r#"\section{Foo}\label{sec:foo}"#)])
            .main("main.tex")
            .line(0)
            .character(23)
            .build()
            .hover();

        let actual_hover = find_label_hover(&request, CancellationToken::none()).unwrap();

        assert_eq!(actual_hover.range.unwrap(), Range::new_simple(0, 20, 0, 27));
    }
}
