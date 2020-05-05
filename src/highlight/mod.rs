mod latex_label;

use self::latex_label::LatexLabelHighlightProvider;
use crate::{
    feature::{ConcatProvider, FeatureProvider, FeatureRequest},
    protocol::{DocumentHighlight, TextDocumentPositionParams},
};
use async_trait::async_trait;

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

#[async_trait]
impl FeatureProvider for HighlightProvider {
    type Params = TextDocumentPositionParams;
    type Output = Vec<DocumentHighlight>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        self.provider.execute(req).await
    }
}
