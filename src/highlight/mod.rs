mod latex_label;

use self::latex_label::LatexLabelHighlightProvider;
use crate::concat_feature;
use crate::feature::FeatureRequest;
use lsp_types::{DocumentHighlight, TextDocumentPositionParams};

pub struct HighlightProvider;

impl HighlightProvider {
    pub async fn execute(
        request: &FeatureRequest<TextDocumentPositionParams>,
    ) -> Vec<DocumentHighlight> {
        concat_feature!(&request, LatexLabelHighlightProvider)
    }
}
