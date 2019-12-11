use crate::citeproc::render_citation;
use texlab_protocol::RangeExt;
use crate::syntax::*;
use crate::workspace::*;
use futures_boxed::boxed;
use log::warn;
use lsp_types::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexCitationHoverProvider;

impl FeatureProvider for LatexCitationHoverProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Hover>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<TextDocumentPositionParams>,
    ) -> Option<Hover> {
        let (tree, entry) = Self::get_entry(request)?;
        if entry.is_comment() {
            None
        } else {
            let key = entry.key.as_ref().unwrap().text();
            match render_citation(&tree, key) {
                Some(markdown) => Some(Hover {
                    contents: HoverContents::Markup(markdown),
                    range: None,
                }),
                None => {
                    warn!("Failed to render entry: {}", key);
                    None
                }
            }
        }
    }
}

impl LatexCitationHoverProvider {
    fn get_entry(
        request: &FeatureRequest<TextDocumentPositionParams>,
    ) -> Option<(&BibtexSyntaxTree, &BibtexEntry)> {
        let key = Self::get_key(request)?;
        for document in request.related_documents() {
            if let SyntaxTree::Bibtex(tree) = &document.tree {
                for entry in tree.entries() {
                    if let Some(current_key) = &entry.key {
                        if current_key.text() == key {
                            return Some((tree, entry));
                        }
                    }
                }
            }
        }
        None
    }

    fn get_key(request: &FeatureRequest<TextDocumentPositionParams>) -> Option<&str> {
        match &request.document().tree {
            SyntaxTree::Latex(tree) => tree
                .citations
                .iter()
                .flat_map(LatexCitation::keys)
                .find(|citation| citation.range().contains(request.params.position))
                .map(LatexToken::text),
            SyntaxTree::Bibtex(tree) => {
                for entry in tree.entries() {
                    if let Some(key) = &entry.key {
                        if key.range().contains(request.params.position) {
                            return Some(key.text());
                        }
                    }
                }
                None
            }
        }
    }
}
