use texlab_workspace::*;
use futures_boxed::boxed;
use texlab_protocol::RangeExt;
use texlab_protocol::{Location, ReferenceParams};
use texlab_syntax::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexLabelReferenceProvider;

impl FeatureProvider for LatexLabelReferenceProvider {
    type Params = ReferenceParams;
    type Output = Vec<Location>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<ReferenceParams>) -> Vec<Location> {
        let mut references = Vec::new();
        if let Some(definition) = Self::find_name(request) {
            for document in request.related_documents() {
                if let SyntaxTree::Latex(tree) = &document.tree {
                    tree.structure
                        .labels
                        .iter()
                        .filter(|label| Self::is_included(request, label))
                        .flat_map(LatexLabel::names)
                        .filter(|label| label.text() == definition)
                        .map(|label| Location::new(document.uri.clone().into(), label.range()))
                        .for_each(|location| references.push(location))
                }
            }
        }
        references
    }
}

impl LatexLabelReferenceProvider {
    fn find_name(request: &FeatureRequest<ReferenceParams>) -> Option<&str> {
        if let SyntaxTree::Latex(tree) = &request.document().tree {
            tree.structure
                .labels
                .iter()
                .flat_map(LatexLabel::names)
                .find(|label| {
                    label
                        .range()
                        .contains(request.params.text_document_position.position)
                })
                .map(LatexToken::text)
        } else {
            None
        }
    }

    fn is_included(request: &FeatureRequest<ReferenceParams>, label: &LatexLabel) -> bool {
        match label.kind {
            LatexLabelKind::Reference(_) => true,
            LatexLabelKind::Definition => request.params.context.include_declaration,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use texlab_protocol::RangeExt;
    use texlab_protocol::{Position, Range};

    #[test]
    fn test_definition() {
        let references = test_feature(
            LatexLabelReferenceProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\label{foo}"),
                    FeatureSpec::file("bar.tex", "\\input{foo.tex}\n\\ref{foo}"),
                    FeatureSpec::file("baz.tex", "\\ref{foo}"),
                ],
                main_file: "foo.tex",
                include_declaration: false,
                position: Position::new(0, 8),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            references,
            vec![Location::new(
                FeatureSpec::uri("bar.tex"),
                Range::new_simple(1, 5, 1, 8)
            )]
        );
    }

    #[test]
    fn test_definition_include_declaration() {
        let references = test_feature(
            LatexLabelReferenceProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\label{foo}"),
                    FeatureSpec::file("bar.tex", "\\input{foo.tex}\n\\ref{foo}"),
                    FeatureSpec::file("baz.tex", "\\ref{foo}"),
                ],
                main_file: "foo.tex",
                include_declaration: true,
                position: Position::new(0, 8),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            references,
            vec![
                Location::new(FeatureSpec::uri("foo.tex"), Range::new_simple(0, 7, 0, 10)),
                Location::new(FeatureSpec::uri("bar.tex"), Range::new_simple(1, 5, 1, 8)),
            ]
        );
    }

    #[test]
    fn test_reference() {
        let references = test_feature(
            LatexLabelReferenceProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\label{foo}"),
                    FeatureSpec::file("bar.tex", "\\input{foo.tex}\n\\ref{foo}"),
                    FeatureSpec::file("baz.tex", "\\ref{foo}"),
                ],
                main_file: "bar.tex",
                position: Position::new(1, 7),
                include_declaration: false,
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            references,
            vec![Location::new(
                FeatureSpec::uri("bar.tex"),
                Range::new_simple(1, 5, 1, 8)
            ),]
        );
    }

    #[test]
    fn test_reference_include_declaration() {
        let references = test_feature(
            LatexLabelReferenceProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\label{foo}"),
                    FeatureSpec::file("bar.tex", "\\input{foo.tex}\n\\ref{foo}"),
                    FeatureSpec::file("baz.tex", "\\ref{foo}"),
                ],
                main_file: "bar.tex",
                position: Position::new(1, 7),
                include_declaration: true,
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            references,
            vec![
                Location::new(FeatureSpec::uri("bar.tex"), Range::new_simple(1, 5, 1, 8)),
                Location::new(FeatureSpec::uri("foo.tex"), Range::new_simple(0, 7, 0, 10)),
            ]
        );
    }

    #[test]
    fn test_bibtex() {
        let references = test_feature(
            LatexLabelReferenceProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "")],
                main_file: "foo.bib",
                position: Position::new(0, 0),
                ..FeatureSpec::default()
            },
        );
        assert!(references.is_empty());
    }
}
