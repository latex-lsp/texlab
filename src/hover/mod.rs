mod bibtex_entry_type;
mod bibtex_field;
mod bibtex_string_reference;
mod latex_citation;
mod latex_component;
mod latex_label;
mod latex_preview;

use self::bibtex_entry_type::BibtexEntryTypeHoverProvider;
use self::bibtex_field::BibtexFieldHoverProvider;
use self::bibtex_string_reference::BibtexStringReferenceHoverProvider;
use self::latex_citation::LatexCitationHoverProvider;
use self::latex_component::LatexComponentHoverProvider;
use self::latex_label::LatexLabelHoverProvider;
use self::latex_preview::LatexPreviewHoverProvider;
use crate::workspace::*;
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
                Box::new(BibtexStringReferenceHoverProvider),
                Box::new(BibtexFieldHoverProvider),
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
