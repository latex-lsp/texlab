mod bibtex_entry;
mod latex_label;

use crate::concat_feature;
use crate::feature::FeatureRequest;
use crate::reference::bibtex_entry::BibtexEntryReferenceProvider;
use crate::reference::latex_label::LatexLabelReferenceProvider;
use lsp_types::{Location, ReferenceParams};

pub struct ReferenceProvider;

impl ReferenceProvider {
    pub async fn execute(request: &FeatureRequest<ReferenceParams>) -> Vec<Location> {
        concat_feature!(
            &request,
            BibtexEntryReferenceProvider,
            LatexLabelReferenceProvider
        )
    }
}
