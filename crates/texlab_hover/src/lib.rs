mod bibtex;
mod latex;

use self::bibtex::entry_type::BibtexEntryTypeHoverProvider;
use self::bibtex::field::BibtexFieldHoverProvider;
use self::bibtex::string_reference::BibtexStringReferenceHoverProvider;
use self::latex::citation::LatexCitationHoverProvider;
use self::latex::component::LatexComponentHoverProvider;
use self::latex::include::LatexIncludeHoverProvider;
use self::latex::label::LatexLabelHoverProvider;
use self::latex::preview::LatexPreviewHoverProvider;
use futures_boxed::boxed;
use texlab_protocol::{Hover, TextDocumentPositionParams};
use texlab_workspace::*;

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
                Box::new(LatexIncludeHoverProvider),
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
