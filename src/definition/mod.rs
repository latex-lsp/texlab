mod latex_citation;
mod latex_command;
mod latex_label;

use self::latex_citation::LatexCitationDefinitionProvider;
use self::latex_command::LatexCommandDefinitionProvider;
use self::latex_label::LatexLabelDefinitionProvider;
use crate::feature::{ConcatProvider, FeatureProvider, FeatureRequest};
use futures_boxed::boxed;
use lsp_types::{Location, TextDocumentPositionParams};

pub struct DefinitionProvider {
    provider: ConcatProvider<TextDocumentPositionParams, Location>,
}

impl DefinitionProvider {
    pub fn new() -> Self {
        Self {
            provider: ConcatProvider::new(vec![
                Box::new(LatexCitationDefinitionProvider),
                Box::new(LatexCommandDefinitionProvider),
                Box::new(LatexLabelDefinitionProvider),
            ]),
        }
    }
}

impl FeatureProvider for DefinitionProvider {
    type Params = TextDocumentPositionParams;
    type Output = Vec<Location>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<TextDocumentPositionParams>,
    ) -> Vec<Location> {
        self.provider.execute(request).await
    }
}
