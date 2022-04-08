use cancellation::CancellationToken;
use lsp_types::{GotoDefinitionParams, LocationLink};

use crate::{
    features::cursor::CursorContext, find_label_definition, render_label, syntax::latex,
    LineIndexExt,
};

pub fn goto_label_definition(
    context: &CursorContext<GotoDefinitionParams>,
    cancellation_token: &CancellationToken,
) -> Option<Vec<LocationLink>> {
    let main_document = context.request.main_document();

    let (name_text, name_range) = context
        .find_label_name_key()
        .or_else(|| context.find_label_name_command())?;

    let origin_selection_range = main_document.line_index.line_col_lsp_range(name_range);

    for document in &context.request.subset.documents {
        cancellation_token.result().ok()?;
        if let Some(data) = document.data.as_latex() {
            if let Some(definition) =
                find_label_definition(&latex::SyntaxNode::new_root(data.root.clone()), &name_text)
            {
                let target_selection_range = latex::small_range(&definition.name()?.key()?);
                let target_range =
                    render_label(&context.request.subset, &name_text, Some(definition))
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

        let context = CursorContext::new(request);
        let actual_links = goto_label_definition(&context, CancellationToken::none());

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
        let actual_links = goto_label_definition(&context, CancellationToken::none());

        assert!(actual_links.is_none());
    }
}
