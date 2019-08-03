mod bibtex_entry;
mod latex_section;

use self::bibtex_entry::BibtexEntrySymbolProvider;
use self::latex_section::LatexSectionSymbolProvider;
use crate::workspace::*;
use futures_boxed::boxed;
use lsp_types::{DocumentSymbol, DocumentSymbolParams};

pub struct SymbolProvider {
    provider: ConcatProvider<DocumentSymbolParams, DocumentSymbol>,
}

impl SymbolProvider {
    pub fn new() -> Self {
        Self {
            provider: ConcatProvider::new(vec![
                Box::new(BibtexEntrySymbolProvider),
                Box::new(LatexSectionSymbolProvider),
            ]),
        }
    }
}

impl FeatureProvider for SymbolProvider {
    type Params = DocumentSymbolParams;
    type Output = Vec<DocumentSymbol>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        self.provider.execute(request).await
    }
}
