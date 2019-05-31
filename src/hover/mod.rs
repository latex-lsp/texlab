mod bibtex_entry_type;
mod bibtex_field;
mod latex_citation;
mod latex_component;

use self::bibtex_entry_type::BibtexEntryTypeHoverProvider;
use self::bibtex_field::BibtexFieldHoverProvider;
use self::latex_citation::LatexCitationHoverProvider;
use self::latex_component::LatexComponentHoverProvider;
use crate::feature::{ChoiceProvider, FeatureProvider, FeatureRequest};
use futures::prelude::*;
use futures_boxed::boxed;
use lsp_types::{Hover, TextDocumentPositionParams};

pub struct HoverProvider {
    provider: ChoiceProvider<TextDocumentPositionParams, Hover>,
}

impl HoverProvider {
    pub fn new() -> Self {
        Self {
            provider: ChoiceProvider::new(vec![
                Box::new(BibtexEntryTypeHoverProvider),
                Box::new(BibtexFieldHoverProvider),
                Box::new(LatexCitationHoverProvider),
                Box::new(LatexComponentHoverProvider),
            ]),
        }
    }
}

impl FeatureProvider for HoverProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Hover>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<TextDocumentPositionParams>,
    ) -> Option<Hover> {
        self.provider.execute(request).await
    }
}
