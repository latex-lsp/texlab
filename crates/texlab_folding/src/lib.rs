mod bibtex_decl;
mod latex_env;
mod latex_section;

use self::{
    bibtex_decl::BibtexDeclarationFoldingProvider, latex_env::LatexEnvironmentFoldingProvider,
    latex_section::LatexSectionFoldingProvider,
};
use futures_boxed::boxed;
use texlab_feature::{ConcatProvider, FeatureProvider, FeatureRequest};
use texlab_protocol::{FoldingRange, FoldingRangeParams};

pub struct FoldingProvider {
    provider: ConcatProvider<FoldingRangeParams, FoldingRange>,
}

impl FoldingProvider {
    pub fn new() -> Self {
        Self {
            provider: ConcatProvider::new(vec![
                Box::new(BibtexDeclarationFoldingProvider),
                Box::new(LatexEnvironmentFoldingProvider),
                Box::new(LatexSectionFoldingProvider),
            ]),
        }
    }
}

impl Default for FoldingProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureProvider for FoldingProvider {
    type Params = FoldingRangeParams;
    type Output = Vec<FoldingRange>;

    #[boxed]
    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        self.provider.execute(req).await
    }
}
