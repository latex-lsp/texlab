use crate::feature::FeatureRequest;
use crate::metadata::bibtex_field;
use crate::syntax::bibtex::*;
use crate::workspace::SyntaxTree;
use lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, TextDocumentPositionParams};

pub struct BibtexFieldHoverProvider;

impl BibtexFieldHoverProvider {
    pub async fn execute(request: &FeatureRequest<TextDocumentPositionParams>) -> Option<Hover> {
        if let SyntaxTree::Bibtex(tree) = &request.document.tree {
            let mut finder = BibtexFinder::new(request.params.position);
            finder.visit_root(&tree.root);
            for node in finder.results {
                if let BibtexNode::Field(field) = node {
                    let documentation = bibtex_field::get_documentation(field.name.text())?;
                    return Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: documentation.to_owned(),
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
    use crate::completion::latex::data::types::LatexComponentDatabase;
    use crate::feature::FeatureSpec;
    use crate::test_feature;
    use lsp_types::Position;

    #[test]
    fn test_known_field() {
        let hover = test_feature!(
            BibtexFieldHoverProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, author = bar}")],
                main_file: "foo.bib",
                position: Position::new(0, 15),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(
            hover,
            Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: bibtex_field::get_documentation("author")
                        .unwrap()
                        .to_owned(),
                }),
                range: None,
            })
        );
    }

    #[test]
    fn test_unknown_field() {
        let hover = test_feature!(
            BibtexFieldHoverProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, bar = baz}")],
                main_file: "foo.bib",
                position: Position::new(0, 15),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(hover, None);
    }

    #[test]
    fn test_entry_key() {
        let hover = test_feature!(
            BibtexFieldHoverProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, bar = baz}")],
                main_file: "foo.bib",
                position: Position::new(0, 11),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(hover, None);
    }

    #[test]
    fn test_latex() {
        let hover = test_feature!(
            BibtexFieldHoverProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "")],
                main_file: "foo.tex",
                position: Position::new(0, 0),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(hover, None);
    }
}
