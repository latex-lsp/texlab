use crate::feature::FeatureRequest;
use crate::syntax::latex::{LatexCitation, LatexToken};
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use crate::workspace::Document;
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
            for entry in tree.entries() {
                if let Some(key) = &entry.key {
                    if key.text() == reference {
                        return Some(Location::new(document.uri.clone(), key.range()));
                    }
                }
            }
        }
        None
    }

    fn find_reference(request: &FeatureRequest<TextDocumentPositionParams>) -> Option<&str> {
        if let SyntaxTree::Latex(tree) = &request.document.tree {
            tree.citations
                .iter()
                .map(LatexCitation::key)
                .find(|key| key.range().contains(request.params.position))
                .map(LatexToken::text)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureSpec;
    use crate::test_feature;
    use lsp_types::{Position, Range};

    #[test]
    fn test_has_definition() {
        let locations = test_feature!(
            LatexCitationDefinitionProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\addbibresource{baz.bib}\n\\cite{foo}"),
                    FeatureSpec::file("bar.bib", "@article{foo, bar = {baz}}"),
                    FeatureSpec::file("baz.bib", "@article{foo, bar = {baz}}"),
                ],
                main_file: "foo.tex",
                position: Position::new(1, 6),
                ..FeatureSpec::default()
            }
        );
        assert_eq!(
            locations,
            vec![Location::new(
                FeatureSpec::uri("baz.bib"),
                Range::new_simple(0, 9, 0, 12)
            )]
        );
    }

    #[test]
    fn test_no_definition_latex() {
        let locations = test_feature!(
            LatexCitationDefinitionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", ""),],
                main_file: "foo.tex",
                position: Position::new(0, 0),
                ..FeatureSpec::default()
            }
        );
        assert_eq!(locations, Vec::new());
    }

    #[test]
    fn test_no_definition_bibtex() {
        let locations = test_feature!(
            LatexCitationDefinitionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", ""),],
                main_file: "foo.bib",
                position: Position::new(0, 0),
                ..FeatureSpec::default()
            }
        );
        assert_eq!(locations, Vec::new());
    }
}
