mod bibtex_string;
mod latex_citation;
mod latex_cmd;
mod latex_label;

use self::{
    bibtex_string::BibtexStringDefinitionProvider, latex_citation::LatexCitationDefinitionProvider,
    latex_cmd::LatexCommandDefinitionProvider, latex_label::LatexLabelDefinitionProvider,
};
use async_trait::async_trait;
use texlab_feature::{ConcatProvider, FeatureProvider, FeatureRequest};
use texlab_protocol::{LocationLink, TextDocumentPositionParams};

pub struct DefinitionProvider {
    provider: ConcatProvider<TextDocumentPositionParams, LocationLink>,
}

impl DefinitionProvider {
    pub fn new() -> Self {
        Self {
            provider: ConcatProvider::new(vec![
                Box::new(BibtexStringDefinitionProvider),
                Box::new(LatexCitationDefinitionProvider),
                Box::new(LatexCommandDefinitionProvider),
                Box::new(LatexLabelDefinitionProvider),
            ]),
        }
    }
}

impl Default for DefinitionProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FeatureProvider for DefinitionProvider {
    type Params = TextDocumentPositionParams;
    type Output = Vec<LocationLink>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        self.provider.execute(req).await
    }
}
