mod latex_citation;
mod latex_label;

use crate::concat_feature;
use crate::definition::latex_citation::LatexCitationDefinitionProvider;
use crate::definition::latex_label::LatexLabelDefinitionProvider;
use crate::feature::FeatureRequest;
use lsp_types::{Location, TextDocumentPositionParams};

pub struct DefinitionProvider;

impl DefinitionProvider {
    pub async fn execute(request: &FeatureRequest<TextDocumentPositionParams>) -> Vec<Location> {
        concat_feature!(
            &request,
            LatexCitationDefinitionProvider,
            LatexLabelDefinitionProvider
        )
    }
}
