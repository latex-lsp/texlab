use crate::feature::FeatureRequest;
use crate::formatting::{BibtexFormatter, BibtexFormattingOptions};
use crate::syntax::bibtex::{BibtexDeclaration, BibtexEntry};
use crate::syntax::latex::{LatexCitationAnalyzer, LatexToken, LatexVisitor};
use crate::syntax::text::SyntaxNode;
use crate::workspace::SyntaxTree;
use lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, TextDocumentPositionParams};
use std::borrow::Cow;

pub struct LatexCitationHoverProvider;

impl LatexCitationHoverProvider {
    pub async fn execute(request: &FeatureRequest<TextDocumentPositionParams>) -> Option<Hover> {
        let entry = Self::get_entry(request)?;
        if entry.is_comment() {
            None
        } else {
            let mut formatter = BibtexFormatter::new(BibtexFormattingOptions::new(4, true, 80));
            formatter.format_entry(entry);
            Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: Cow::from(format!("```bibtex\n{}\n```", formatter.output)),
                }),
                range: None,
            })
        }
    }

    fn get_entry(request: &FeatureRequest<TextDocumentPositionParams>) -> Option<&BibtexEntry> {
        let key = Self::get_key(request)?;
        for document in &request.related_documents {
            if let SyntaxTree::Bibtex(tree) = &document.tree {
                for declaration in &tree.root.children {
                    if let BibtexDeclaration::Entry(entry) = &declaration {
                        if let Some(current_key) = &entry.key {
                            if current_key.text() == key {
                                return Some(entry);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn get_key(request: &FeatureRequest<TextDocumentPositionParams>) -> Option<&str> {
        match &request.document.tree {
            SyntaxTree::Latex(tree) => {
                let mut analyzer = LatexCitationAnalyzer::new();
                analyzer.visit_root(&tree.root);
                analyzer
                    .citations
                    .iter()
                    .find(|citation| citation.command.range.contains(request.params.position))
                    .map(|citation| citation.key.text())
            }
            SyntaxTree::Bibtex(tree) => {
                for declaration in &tree.root.children {
                    if let BibtexDeclaration::Entry(entry) = &declaration {
                        if let Some(key) = &entry.key {
                            if key.range().contains(request.params.position) {
                                return Some(key.text());
                            }
                        }
                    }
                }
                None
            }
        }
    }
}
