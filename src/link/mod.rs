mod latex_include;

use self::latex_include::LatexIncludeLinkProvider;
use crate::feature::{ConcatProvider, FeatureProvider, FeatureRequest};
use futures_boxed::boxed;
use texlab_protocol::{DocumentLink, DocumentLinkParams};

pub struct LinkProvider {
    provider: ConcatProvider<DocumentLinkParams, DocumentLink>,
}

impl LinkProvider {
    pub fn new() -> Self {
        Self {
            provider: ConcatProvider::new(vec![Box::new(LatexIncludeLinkProvider)]),
        }
    }
}

impl Default for LinkProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureProvider for LinkProvider {
    type Params = DocumentLinkParams;
    type Output = Vec<DocumentLink>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        self.provider.execute(req).await
    }
}
