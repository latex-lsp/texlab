mod bibtex_string;
mod latex_citation;
mod latex_command;
mod latex_label;

use self::bibtex_string::BibtexStringDefinitionProvider;
use self::latex_citation::LatexCitationDefinitionProvider;
use self::latex_command::LatexCommandDefinitionProvider;
use self::latex_label::LatexLabelDefinitionProvider;
use futures_boxed::boxed;
use texlab_protocol::{LocationLink, TextDocumentPositionParams};
use texlab_workspace::*;

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

impl FeatureProvider for DefinitionProvider {
    type Params = TextDocumentPositionParams;
    type Output = Vec<LocationLink>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        self.provider.execute(request).await
    }
}
