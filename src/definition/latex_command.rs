use crate::feature::{FeatureProvider, FeatureRequest};
use crate::syntax::latex::LatexNode;
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use futures_boxed::boxed;
use lsp_types::{Location, TextDocumentPositionParams};

pub struct LatexCommandDefinitionProvider;

impl FeatureProvider for LatexCommandDefinitionProvider {
    type Params = TextDocumentPositionParams;
    type Output = Vec<Location>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut definitions = Vec::new();
        if let SyntaxTree::Latex(tree) = &request.document().tree {
            if let Some(LatexNode::Command(command)) = tree.find(request.params.position).last() {
                for document in request.related_documents() {
                    if let SyntaxTree::Latex(tree) = &document.tree {
                        tree.command_definitions
                            .iter()
                            .filter(|def| def.name.name.text() == command.name.text())
                            .map(|def| Location::new(document.uri.clone(), def.range()))
                            .for_each(|def| definitions.push(def));
                    }
                }
            }
        }
        definitions
    }
}
