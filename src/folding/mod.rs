mod bibtex_decl;
mod latex_env;
mod latex_section;

use self::{
    bibtex_decl::BibtexDeclarationFoldingProvider, latex_env::LatexEnvironmentFoldingProvider,
    latex_section::LatexSectionFoldingProvider,
};
use crate::{
    feature::{ConcatProvider, FeatureProvider, FeatureRequest},
    protocol::{FoldingRange, FoldingRangeParams},
};
use futures_boxed::boxed;

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
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<FoldingRangeParams>,
    ) -> Vec<FoldingRange> {
        self.provider.execute(request).await
    }
}
