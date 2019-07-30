use futures_boxed::boxed;
use lsp_types::{Location, TextDocumentPositionParams};
use texlab_syntax::*;
use texlab_workspace::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexCitationDefinitionProvider;

impl FeatureProvider for LatexCitationDefinitionProvider {
    type Params = TextDocumentPositionParams;
    type Output = Vec<Location>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<TextDocumentPositionParams>,
    ) -> Vec<Location> {
        if let Some(reference) = Self::find_reference(&request) {
            for document in request.related_documents() {
                if let Some(definition) = Self::find_definition(&document, &reference) {
                    return vec![definition];
                }
            }
        }
        Vec::new()
    }
}

impl LatexCitationDefinitionProvider {
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
        if let SyntaxTree::Latex(tree) = &request.document().tree {
            tree.citations
                .iter()
                .flat_map(LatexCitation::keys)
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
    use lsp_types::{Position, Range};

    #[test]
    fn test_has_definition() {
        let locations = test_feature(
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
            },
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
        let locations = test_feature(
            LatexCitationDefinitionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "")],
                main_file: "foo.tex",
                position: Position::new(0, 0),
                ..FeatureSpec::default()
            },
        );
        assert!(locations.is_empty());
    }

    #[test]
    fn test_no_definition_bibtex() {
        let locations = test_feature(
            LatexCitationDefinitionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "")],
                main_file: "foo.bib",
                position: Position::new(0, 0),
                ..FeatureSpec::default()
            },
        );
        assert!(locations.is_empty());
    }
}
