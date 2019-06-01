use crate::data::component::ComponentDocumentation;
use crate::feature::{FeatureProvider, FeatureRequest};
use crate::syntax::latex::LatexIncludeKind;
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use futures_boxed::boxed;
use lsp_types::{Hover, HoverContents, TextDocumentPositionParams};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexComponentHoverProvider;

impl FeatureProvider for LatexComponentHoverProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Hover>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<TextDocumentPositionParams>,
    ) -> Option<Hover> {
        if let SyntaxTree::Latex(tree) = &request.document.tree {
            let documentation = tree
                .includes
                .iter()
                .filter(|include| {
                    include.kind() == LatexIncludeKind::Package
                        || include.kind() == LatexIncludeKind::Class
                })
                .find(|include| include.path().range().contains(request.params.position))
                .map(|include| include.path().text())
                .map(|name| ComponentDocumentation::lookup(name))?
                .await?;

            Some(Hover {
                contents: HoverContents::Markup(documentation.content),
                range: None,
            })
        } else {
            None
        }
    }
}
