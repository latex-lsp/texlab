use crate::feature::FeatureRequest;
use crate::range;
use crate::syntax::bibtex::ast::{BibtexDeclaration, BibtexEntry};
use crate::syntax::latex::analysis::citation::LatexCitationAnalyzer;
use crate::syntax::latex::ast::LatexVisitor;
use crate::syntax::text::SyntaxNode;
use crate::workspace::SyntaxTree;
use lsp_types::{Location, ReferenceParams};

pub struct BibtexEntryReferenceProvider;

impl BibtexEntryReferenceProvider {
    pub async fn execute(request: &FeatureRequest<ReferenceParams>) -> Vec<Location> {
        let mut references = Vec::new();
        if let Some(key) = Self::find_definition(request) {
            for document in &request.related_documents {
                if let SyntaxTree::Latex(tree) = &document.tree {
                    let mut analyzer = LatexCitationAnalyzer::new();
                    analyzer.visit_root(&tree.root);
                    analyzer
                        .citations
                        .iter()
                        .filter(|citation| citation.key.text() == key)
                        .map(|citation| Location::new(document.uri.clone(), citation.command.range))
                        .for_each(|location| references.push(location))
                }
            }
        }
        references
    }

    fn find_definition(request: &FeatureRequest<ReferenceParams>) -> Option<&str> {
        if let SyntaxTree::Bibtex(tree) = &request.document.tree {
            for declaration in &tree.root.children {
                if let BibtexDeclaration::Entry(entry) = declaration {
                    if let Some(key) = &entry.key {
                        if range::contains(key.range(), request.params.position) {
                            return Some(key.text());
                        }
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;
    use crate::workspace::WorkspaceBuilder;
    use futures::executor;

    #[test]
    fn test() {
        let mut builder = WorkspaceBuilder::new();
        let uri1 = builder.document("foo.bib", "@article{foo, bar = {baz}}");
        let uri2 = builder.document("bar.tex", "\\addbibresource{foo.bib}\n\\cite{foo}");
        builder.document("baz.tex", "\\cite{foo}");
        let request = FeatureTester::new(builder.workspace, uri1, 0, 9, "").into();

        let results = executor::block_on(BibtexEntryReferenceProvider::execute(&request));
        let location = Location::new(uri2, range::create(1, 0, 1, 10));
        assert_eq!(vec![location], results);
    }

    #[test]
    fn test_latex() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "");
        let request = FeatureTester::new(builder.workspace, uri, 0, 0, "").into();

        let results = executor::block_on(BibtexEntryReferenceProvider::execute(&request));

        assert_eq!(results, Vec::new());
    }
}
