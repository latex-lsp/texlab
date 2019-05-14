use crate::data::component_doc::ComponentDocumentation;
use crate::feature::FeatureRequest;
use crate::syntax::latex::{LatexIncludeAnalyzer, LatexIncludeKind, LatexVisitor};
use crate::syntax::SyntaxTree;
use lsp_types::{Hover, HoverContents, TextDocumentPositionParams};

pub struct LatexComponentHoverProvider;

impl LatexComponentHoverProvider {
    pub async fn execute(request: &FeatureRequest<TextDocumentPositionParams>) -> Option<Hover> {
        if let SyntaxTree::Latex(tree) = &request.document.tree {
            let mut analyzer = LatexIncludeAnalyzer::new();
            analyzer.visit_root(&tree.root);
            let documentation = await!(analyzer
                .included_files
                .iter()
                .filter(|include| {
                    include.kind == LatexIncludeKind::Package
                        || include.kind == LatexIncludeKind::Class
                })
                .find(|include| include.command.range.contains(request.params.position))
                .map(|include| include.path.text())
                .map(|name| ComponentDocumentation::lookup(name))?)?;

            Some(Hover {
                contents: HoverContents::Markup(documentation.content),
                range: None,
            })
        } else {
            None
        }
    }
}
