use crate::data::label::LabelContext;
use crate::feature::{FeatureProvider, FeatureRequest};
use crate::outline::Outline;
use texlab_syntax::*;
use futures_boxed::boxed;
use lsp_types::{Hover, HoverContents, TextDocumentPositionParams};

pub struct LatexLabelHoverProvider;

impl FeatureProvider for LatexLabelHoverProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Hover>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        if let SyntaxTree::Latex(tree) = &request.document().tree {
            let reference = tree
                .labels
                .iter()
                .flat_map(LatexLabel::names)
                .find(|label| label.range().contains(request.params.position))?;

            let definition = tree
                .labels
                .iter()
                .filter(|label| label.kind == LatexLabelKind::Definition)
                .flat_map(LatexLabel::names)
                .find(|label| label.text() == reference.text())?;

            let outline = Outline::from(&request.view);
            let context = LabelContext::find(&outline, request.document(), definition.start());
            let markup = context.documentation()?;
            return Some(Hover {
                contents: HoverContents::Markup(markup),
                range: Some(reference.range()),
            });
        }
        None
    }
}
