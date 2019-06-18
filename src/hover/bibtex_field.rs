use crate::data::bibtex_field;
use crate::feature::{FeatureProvider, FeatureRequest};
use crate::syntax::bibtex::*;
use crate::syntax::SyntaxTree;
use futures_boxed::boxed;
use lsp_types::*;
use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BibtexFieldHoverProvider;

impl FeatureProvider for BibtexFieldHoverProvider {
    type Params = TextDocumentPositionParams;
    type Output = Option<Hover>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<TextDocumentPositionParams>,
    ) -> Option<Hover> {
        if let SyntaxTree::Bibtex(tree) = &request.document().tree {
            for node in tree.find(request.params.position) {
                if let BibtexNode::Field(field) = node {
                    let documentation = bibtex_field::get_documentation(field.name.text())?;
                    return Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: Cow::from(documentation),
                        }),
                        range: None,
                    });
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
    use lsp_types::Position;

    #[test]
    fn test_known_field() {
        let hover = test_feature(
            BibtexFieldHoverProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, author = bar}")],
                main_file: "foo.bib",
                position: Position::new(0, 15),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(
            hover,
            Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: Cow::from(bibtex_field::get_documentation("author").unwrap()),
                }),
                range: None,
            })
        );
    }

    #[test]
    fn test_unknown_field() {
        let hover = test_feature(
            BibtexFieldHoverProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, bar = baz}")],
                main_file: "foo.bib",
                position: Position::new(0, 15),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(hover, None);
    }

    #[test]
    fn test_entry_key() {
        let hover = test_feature(
            BibtexFieldHoverProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, bar = baz}")],
                main_file: "foo.bib",
                position: Position::new(0, 11),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(hover, None);
    }

    #[test]
    fn test_latex() {
        let hover = test_feature(
            BibtexFieldHoverProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "")],
                main_file: "foo.tex",
                position: Position::new(0, 0),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(hover, None);
    }
}
