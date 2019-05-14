mod bibtex_entry_type;
mod bibtex_field;
mod latex_citation;
mod latex_component;

use self::bibtex_entry_type::BibtexEntryTypeHoverProvider;
use self::bibtex_field::BibtexFieldHoverProvider;
use self::latex_citation::LatexCitationHoverProvider;
use self::latex_component::LatexComponentHoverProvider;
use crate::choice_feature;
use crate::feature::FeatureRequest;
use lsp_types::{Hover, TextDocumentPositionParams};

pub struct HoverProvider;

impl HoverProvider {
    pub async fn execute(request: &FeatureRequest<TextDocumentPositionParams>) -> Option<Hover> {
        choice_feature!(
            &request,
            BibtexEntryTypeHoverProvider,
            BibtexFieldHoverProvider,
            LatexCitationHoverProvider,
            LatexComponentHoverProvider
        )
    }
}
