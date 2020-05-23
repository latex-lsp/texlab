mod bibtex;
mod latex;

#[cfg(feature = "citation")]
use self::latex::citation::LatexCitationHoverProvider;

use self::{
    bibtex::{
        entry_type::BibtexEntryTypeHoverProvider, field::BibtexFieldHoverProvider,
        string_reference::BibtexStringReferenceHoverProvider,
    },
    latex::{
        component::LatexComponentHoverProvider, label::LatexLabelHoverProvider,
        preview::LatexPreviewHoverProvider,
    },
};
use crate::{
    feature::{ChoiceProvider, FeatureProvider, FeatureRequest},
    protocol::{Hover, TextDocumentPositionParams},
};
use async_trait::async_trait;

pub struct HoverProvider {
    provider: ChoiceProvider<TextDocumentPositionParams, Hover>,
}

impl HoverProvider {
    pub fn new() -> Self {
        Self {
            provider: ChoiceProvider::new(vec![
                Box::new(BibtexEntryTypeHoverProvider),
                Box::new(BibtexStringReferenceHoverProvider),
                Box::new(BibtexFieldHoverProvider),
                #[cfg(feature = "citation")]
                Box::new(LatexCitationHoverProvider),
                Box::new(LatexComponentHoverProvider),
                Box::new(LatexLabelHoverProvider),
                Box::new(LatexPreviewHoverProvider),
            ]),
        }
    }
}

impl Default for HoverProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FeatureProvider for HoverProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Hover>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        self.provider.execute(req).await
    }
}
