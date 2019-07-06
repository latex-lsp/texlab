use crate::data::component::ComponentDocumentation;
use crate::data::language::LatexIncludeKind;
use crate::feature::{FeatureProvider, FeatureRequest};
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
        if let SyntaxTree::Latex(tree) = &request.document().tree {
            for include in &tree.includes {
                if include.kind == LatexIncludeKind::Package
                    || include.kind == LatexIncludeKind::Class
                {
                    for path in include.paths() {
                        if path.range().contains(request.params.position) {
                            let documentation = ComponentDocumentation::lookup(path.text()).await?;
                            return Some(Hover {
                                contents: HoverContents::Markup(documentation.content),
                                range: Some(path.range()),
                            });
                        }
                    }
                }
            }
        }
        None
    }
}
