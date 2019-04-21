use crate::feature::FeatureRequest;
use crate::range;
use crate::syntax::bibtex::ast::BibtexDeclaration;
use crate::syntax::bibtex::BibtexSyntaxTree;
use crate::syntax::latex::analysis::citation::LatexCitationAnalyzer;
use crate::syntax::latex::ast::LatexVisitor;
use crate::syntax::text::SyntaxNode;
use crate::workspace::{Document, SyntaxTree};
use lsp_types::{Location, TextDocumentPositionParams};

pub struct LatexCitationDefinitionProvider;

impl LatexCitationDefinitionProvider {
    pub async fn execute(request: &FeatureRequest<TextDocumentPositionParams>) -> Vec<Location> {
        if let Some(reference) = Self::find_reference(&request) {
            for document in &request.related_documents {
                if let Some(definition) = Self::find_definition(&document, &reference) {
                    return vec![definition];
                }
            }
        }
        Vec::new()
    }

    fn find_definition(document: &Document, reference: &str) -> Option<Location> {
        if let SyntaxTree::Bibtex(tree) = &document.tree {
            for declaration in &tree.root.children {
                if let BibtexDeclaration::Entry(entry) = declaration {
                    if let Some(key) = &entry.key {
                        if key.text() == reference {
                            return Some(Location::new(document.uri.clone(), key.range()));
                        }
                    }
                }
            }
        }
        None
    }

    fn find_reference(request: &FeatureRequest<TextDocumentPositionParams>) -> Option<&str> {
        if let SyntaxTree::Latex(tree) = &request.document.tree {
            let mut analyzer = LatexCitationAnalyzer::new();
            analyzer.visit_root(&tree.root);
            analyzer
                .citations
                .iter()
                .find(|citation| range::contains(citation.key.range(), request.params.position))
                .map(|citation| citation.key.text())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;
    use crate::workspace::WorkspaceBuilder;
    use futures::executor;

    #[test]
    fn test_has_definition() {
        let mut builder = WorkspaceBuilder::new();
        let uri1 = builder.document("foo.tex", "\\addbibresource{baz.bib}\n\\cite{foo}");
        builder.document("bar.bib", "@article{foo, bar = {baz}}");
        let uri2 = builder.document("baz.bib", "@article{foo, bar = {baz}}");
        let request = FeatureTester::new(builder.workspace, uri1, 1, 6, "").into();

        let results = executor::block_on(LatexCitationDefinitionProvider::execute(&request));
        let location = Location::new(uri2, range::create(0, 9, 0, 12));
        assert_eq!(vec![location], results)
    }

    #[test]
    fn test_no_definition_latex() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "");
        let request = FeatureTester::new(builder.workspace, uri, 0, 0, "").into();
        let results = executor::block_on(LatexCitationDefinitionProvider::execute(&request));
        assert_eq!(results, Vec::new());
    }

    #[test]
    fn test_no_definition_bibtex() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.bib", "");
        let request = FeatureTester::new(builder.workspace, uri, 0, 0, "").into();
        let results = executor::block_on(LatexCitationDefinitionProvider::execute(&request));
        assert_eq!(results, Vec::new());
    }
}
