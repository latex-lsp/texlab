use crate::formatting::bibtex::{self, BibtexFormattingParams};
use futures_boxed::boxed;
use log::warn;
use lsp_types::*;
use texlab_citeproc::{render_citation, RenderCitationError};
use texlab_syntax::*;
use texlab_workspace::*;

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
        let entry = Self::get_entry(request)?;
        if entry.is_comment() {
            None
        } else {
            let entry_code = bibtex::format_entry(&entry, &BibtexFormattingParams::default());
            match render_citation(&entry_code).await {
                Ok(markdown) => Some(Hover {
                    contents: HoverContents::Markup(markdown),
                    range: None,
                }),
                Err(why) => {
                    let message = match why {
                        RenderCitationError::InitializationFailed => {
                            "Failed to initialize citeproc"
                        }
                        RenderCitationError::ScriptFaulty => "Failed to execute citeproc",
                        RenderCitationError::InvalidEntry => "Failed to render entry",
                        RenderCitationError::InvalidOutput => "Unable to decode output",
                        RenderCitationError::NodeNotInstalled => "NodeJS is not installed",
                    };
                    warn!("{}:\n{}", &message, &entry_code);
                    None
                }
            }
        }
    }
}

impl LatexCitationHoverProvider {
    fn get_entry(request: &FeatureRequest<TextDocumentPositionParams>) -> Option<&BibtexEntry> {
        let key = Self::get_key(request)?;
        for document in request.related_documents() {
            if let SyntaxTree::Bibtex(tree) = &document.tree {
                for entry in tree.entries() {
                    if let Some(current_key) = &entry.key {
                        if current_key.text() == key {
                            return Some(entry);
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
