use crate::feature::{FeatureProvider, FeatureRequest};
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use futures_boxed::boxed;
use lsp_types::{Location, ReferenceParams};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BibtexEntryReferenceProvider;

impl FeatureProvider for BibtexEntryReferenceProvider {
    type Params = ReferenceParams;
    type Output = Vec<Location>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<ReferenceParams>) -> Vec<Location> {
        let mut references = Vec::new();
        if let Some(key) = Self::find_definition(request) {
            for document in request.related_documents() {
                if let SyntaxTree::Latex(tree) = &document.tree {
                    tree.citations
                        .iter()
                        .filter(|citation| citation.key().text() == key)
                        .map(|citation| Location::new(document.uri.clone(), citation.command.range))
                        .for_each(|location| references.push(location))
                }
            }
        }
        references
    }
}

impl BibtexEntryReferenceProvider {
    fn find_definition(request: &FeatureRequest<ReferenceParams>) -> Option<&str> {
        if let SyntaxTree::Bibtex(tree) = &request.document().tree {
            for entry in tree.entries() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{test_feature, FeatureSpec};
    use lsp_types::{Position, Range};

    #[test]
    fn test() {
        let references = test_feature(
            BibtexEntryReferenceProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.bib", "@article{foo, bar = {baz}}"),
                    FeatureSpec::file("bar.tex", "\\addbibresource{foo.bib}\n\\cite{foo}"),
                    FeatureSpec::file("baz.tex", "\\cite{foo}"),
                ],
                main_file: "foo.bib",
                position: Position::new(0, 9),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            references,
            vec![Location::new(
                FeatureSpec::uri("bar.tex"),
                Range::new_simple(1, 0, 1, 10)
            )]
        );
    }

    #[test]
    fn test_latex() {
        let references = test_feature(
            BibtexEntryReferenceProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "")],
                main_file: "foo.tex",
                position: Position::new(0, 0),
                ..FeatureSpec::default()
            },
        );
        assert!(references.is_empty());
    }
}
