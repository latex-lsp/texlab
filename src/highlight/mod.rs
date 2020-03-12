mod latex_label;

use self::latex_label::LatexLabelHighlightProvider;
use crate::{
    feature::{ConcatProvider, FeatureProvider, FeatureRequest},
    protocol::{DocumentHighlight, TextDocumentPositionParams},
};
use futures_boxed::boxed;

pub struct HighlightProvider {
    provider: ConcatProvider<TextDocumentPositionParams, DocumentHighlight>,
}

impl HighlightProvider {
    pub fn new() -> Self {
        Self {
            provider: ConcatProvider::new(vec![Box::new(LatexLabelHighlightProvider)]),
        }
    }
}

impl Default for HighlightProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureProvider for HighlightProvider {
    type Params = TextDocumentPositionParams;
    type Output = Vec<DocumentHighlight>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<TextDocumentPositionParams>,
    ) -> Vec<DocumentHighlight> {
        self.provider.execute(request).await
    }
}
