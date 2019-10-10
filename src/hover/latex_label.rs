use crate::range::RangeExt;
use crate::syntax::*;
use crate::workspace::*;
use futures_boxed::boxed;
use lsp_types::*;
use std::sync::Arc;

pub struct LatexLabelHoverProvider;

impl FeatureProvider for LatexLabelHoverProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Hover>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        if let SyntaxTree::Latex(tree) = &request.document().tree {
            let reference = Self::find_reference(tree, request.params.position)?;
            let (document, definition) = Self::find_definition(&request.view, reference)?;

            let workspace = Arc::clone(&request.view.workspace);
            let view = DocumentView::new(workspace, document);
            let outline = Outline::from(&view);
            let outline_context = OutlineContext::parse(&view, &definition, &outline)?;
            let markup = outline_context.documentation();
            Some(Hover {
                contents: HoverContents::Markup(markup),
                range: Some(reference.range()),
            })
        } else {
            None
        }
    }
}

impl LatexLabelHoverProvider {
    fn find_reference(tree: &LatexSyntaxTree, position: Position) -> Option<&LatexToken> {
        for label in &tree.structure.labels {
            let names = label.names();
            if names.len() == 1 && label.range().contains(position) {
                return Some(&label.names()[0]);
            }

            for name in &names {
                if name.range().contains(position) {
                    return Some(name);
                }
            }
        }
        None
    }

    fn find_definition<'a, 'b>(
        view: &'a DocumentView,
        reference: &'b LatexToken,
    ) -> Option<(Arc<Document>, &'a LatexLabel)> {
        for document in &view.related_documents {
            if let SyntaxTree::Latex(tree) = &document.tree {
                for label in &tree.structure.labels {
                    if label.kind == LatexLabelKind::Definition {
                        for name in label.names() {
                            if name.text() == reference.text() {
                                return Some((Arc::clone(&document), label));
                            }
                        }
                    }
                }
            }
        }
        None
    }
}
