mod bibtex_field;

use crate::choice_feature;
use crate::feature::FeatureRequest;
use crate::hover::bibtex_field::BibtexFieldHoverProvider;
use lsp_types::{Hover, TextDocumentPositionParams};

pub struct HoverProvider;

impl HoverProvider {
    pub async fn execute(request: &FeatureRequest<TextDocumentPositionParams>) -> Option<Hover> {
        choice_feature!(&request, BibtexFieldHoverProvider)
    }
}
