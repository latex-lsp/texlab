mod bibtex_declaration;
mod latex_environment;
mod latex_section;

use self::bibtex_declaration::BibtexDeclarationFoldingProvider;
use self::latex_environment::LatexEnvironmentFoldingProvider;
use self::latex_section::LatexSectionFoldingProvider;
use texlab_workspace::*;
use futures_boxed::boxed;
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
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<FoldingRangeParams>,
    ) -> Vec<FoldingRange> {
        self.provider.execute(request).await
    }
}
