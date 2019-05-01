mod bibtex_declaration;
mod latex_environment;
mod latex_section;

use crate::concat_feature;
use crate::feature::FeatureRequest;
use crate::folding::bibtex_declaration::BibtexDeclarationFoldingProvider;
use crate::folding::latex_environment::LatexEnvironmentFoldingProvider;
use crate::folding::latex_section::LatexSectionFoldingProvider;
use lsp_types::{FoldingRange, FoldingRangeParams};

pub struct FoldingProvider;

impl FoldingProvider {
    pub async fn execute(request: &FeatureRequest<FoldingRangeParams>) -> Vec<FoldingRange> {
        concat_feature!(
            &request,
            BibtexDeclarationFoldingProvider,
            LatexEnvironmentFoldingProvider,
            LatexSectionFoldingProvider
        )
    }
}
