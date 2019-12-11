use crate::workspace::*;
use futures_boxed::boxed;
use texlab_protocol::RangeExt;
use texlab_protocol::{LocationLink, TextDocumentPositionParams};
use texlab_syntax::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexCitationDefinitionProvider;

impl FeatureProvider for LatexCitationDefinitionProvider {
    type Params = TextDocumentPositionParams;
    type Output = Vec<LocationLink>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut links = Vec::new();
        if let Some(reference) = Self::find_reference(&request) {
            for document in request.related_documents() {
                Self::find_definitions(&document, &reference, &mut links);
            }
        }
        links
    }
}

impl LatexCitationDefinitionProvider {
    fn find_definitions(
        document: &Document,
        reference: &LatexToken,
        links: &mut Vec<LocationLink>,
    ) {
        if let SyntaxTree::Bibtex(tree) = &document.tree {
            for entry in tree.entries() {
                if let Some(key) = &entry.key {
                    if key.text() == reference.text() {
                        let link = LocationLink {
                            origin_selection_range: Some(reference.range()),
                            target_uri: document.uri.clone().into(),
                            target_range: entry.range(),
                            target_selection_range: key.range(),
                        };
                        links.push(link);
                    }
                }
            }
        }
    }

    fn find_reference(request: &FeatureRequest<TextDocumentPositionParams>) -> Option<&LatexToken> {
        if let SyntaxTree::Latex(tree) = &request.document().tree {
            tree.citations
                .iter()
                .flat_map(LatexCitation::keys)
                .find(|key| key.range().contains(request.params.position))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use texlab_protocol::{Position, Range};

    #[test]
    fn test_has_definition() {
        let links = test_feature(
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
            links,
            vec![LocationLink {
                origin_selection_range: Some(Range::new_simple(1, 6, 1, 9)),
                target_uri: FeatureSpec::uri("baz.bib"),
                target_range: Range::new_simple(0, 0, 0, 26),
                target_selection_range: Range::new_simple(0, 9, 0, 12)
            }]
        );
    }

    #[test]
    fn test_no_definition_latex() {
        let links = test_feature(
            LatexCitationDefinitionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "")],
                main_file: "foo.tex",
                position: Position::new(0, 0),
                ..FeatureSpec::default()
            },
        );
        assert!(links.is_empty());
    }

    #[test]
    fn test_no_definition_bibtex() {
        let links = test_feature(
            LatexCitationDefinitionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "")],
                main_file: "foo.bib",
                position: Position::new(0, 0),
                ..FeatureSpec::default()
            },
        );
        assert!(links.is_empty());
    }
}
