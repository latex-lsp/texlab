use crate::data::bibtex_entry_type;
use crate::feature::FeatureRequest;
use crate::syntax::bibtex::BibtexDeclaration;
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use lsp_types::*;
use std::borrow::Cow;

pub struct BibtexEntryTypeHoverProvider;

impl BibtexEntryTypeHoverProvider {
    pub async fn execute(request: &FeatureRequest<TextDocumentPositionParams>) -> Option<Hover> {
        if let SyntaxTree::Bibtex(tree) = &request.document.tree {
            for declaration in &tree.root.children {
                if let BibtexDeclaration::Entry(entry) = &declaration {
                    if entry.ty.range().contains(request.params.position) {
                        let ty = &entry.ty.text()[1..];
                        if let Some(documentation) = bibtex_entry_type::get_documentation(ty) {
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
    fn test_known_entry_type() {
        let hover = test_feature!(
            BibtexEntryTypeHoverProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo,}")],
                main_file: "foo.bib",
                position: Position::new(0, 3),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(
            hover,
            Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: Cow::from(bibtex_entry_type::get_documentation("article").unwrap()),
                }),
                range: None,
            })
        );
    }

    #[test]
    fn test_unknown_entry_type() {
        let hover = test_feature!(
            BibtexEntryTypeHoverProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@foo{bar,}")],
                main_file: "foo.bib",
                position: Position::new(0, 3),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(hover, None);
    }

    #[test]
    fn test_entry_key() {
        let hover = test_feature!(
            BibtexEntryTypeHoverProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo,}")],
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
            BibtexEntryTypeHoverProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\foo")],
                main_file: "foo.tex",
                position: Position::new(0, 3),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(hover, None);
    }
}
