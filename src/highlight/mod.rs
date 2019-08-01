mod latex_label;

use self::latex_label::LatexLabelHighlightProvider;
use crate::workspace::*;
use futures_boxed::boxed;
use lsp_types::{DocumentHighlight, TextDocumentPositionParams};

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
