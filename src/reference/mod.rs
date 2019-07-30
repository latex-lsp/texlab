mod bibtex_entry;
mod latex_label;

use self::bibtex_entry::BibtexEntryReferenceProvider;
use self::latex_label::LatexLabelReferenceProvider;
use futures_boxed::boxed;
use lsp_types::{Location, ReferenceParams};
use texlab_workspace::*;

pub struct ReferenceProvider {
    provider: ConcatProvider<ReferenceParams, Location>,
}

impl ReferenceProvider {
    pub fn new() -> Self {
        ReferenceProvider {
            provider: ConcatProvider::new(vec![
                Box::new(BibtexEntryReferenceProvider),
                Box::new(LatexLabelReferenceProvider),
            ]),
        }
    }
}

impl FeatureProvider for ReferenceProvider {
    type Params = ReferenceParams;
    type Output = Vec<Location>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<ReferenceParams>) -> Vec<Location> {
        self.provider.execute(request).await
    }
}
