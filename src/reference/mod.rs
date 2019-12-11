mod bibtex_entry;
mod bibtex_string;
mod latex_label;

use self::bibtex_entry::BibtexEntryReferenceProvider;
use self::bibtex_string::BibtexStringReferenceProvider;
use self::latex_label::LatexLabelReferenceProvider;
use crate::workspace::*;
use futures_boxed::boxed;
use texlab_protocol::{Location, ReferenceParams};

pub struct ReferenceProvider {
    provider: ConcatProvider<ReferenceParams, Location>,
}

impl ReferenceProvider {
    pub fn new() -> Self {
        Self {
            provider: ConcatProvider::new(vec![
                Box::new(BibtexEntryReferenceProvider),
                Box::new(BibtexStringReferenceProvider),
                Box::new(LatexLabelReferenceProvider),
            ]),
        }
    }
}

impl Default for ReferenceProvider {
    fn default() -> Self {
        Self::new()
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
