use crate::completion::factory;
use crate::feature::FeatureRequest;
use crate::metadata::bibtex_field::BIBTEX_FIELDS;
use crate::syntax::bibtex::{BibtexFinder, BibtexNode, BibtexVisitor};
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use lsp_types::{CompletionItem, CompletionParams};

pub struct BibtexFieldNameCompletionProvider;

impl BibtexFieldNameCompletionProvider {
    pub async fn execute(request: &FeatureRequest<CompletionParams>) -> Vec<CompletionItem> {
        if let SyntaxTree::Bibtex(tree) = &request.document.tree {
            let mut finder = BibtexFinder::new(request.params.position);
            finder.visit_root(&tree.root);
            match finder.results.last() {
                Some(BibtexNode::Field(field)) => {
                    if field.name.range().contains(request.params.position) {
                        return Self::generate_items();
                    }
                }
                Some(BibtexNode::Entry(entry)) => {
                    if !entry.is_comment() && !entry.ty.range().contains(request.params.position) {
                        if let Some(key) = &entry.key {
                            if !key.range().contains(request.params.position) {
                                return Self::generate_items();
                            }
                        } else {
                            return Self::generate_items();
                        }
                    }
                }
                _ => {}
            }
        }
        Vec::new()
    }

    fn generate_items() -> Vec<CompletionItem> {
        BIBTEX_FIELDS
            .iter()
            .map(|field| factory::create_field_name(field))
            .collect()
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
    fn test_inside_first_field() {
        let items = test_feature!(
            BibtexFieldNameCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo,\nbar}"),],
                main_file: "foo.bib",
                position: Position::new(1, 1),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(items.len() > 0, true);
    }

    #[test]
    fn test_inside_second_field() {
        let items = test_feature!(
            BibtexFieldNameCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.bib",
                    "@article{foo, bar = {baz}, qux}"
                ),],
                main_file: "foo.bib",
                position: Position::new(0, 27),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(items.len() > 0, true);
    }

    #[test]
    fn test_inside_entry() {
        let items = test_feature!(
            BibtexFieldNameCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, \n}"),],
                main_file: "foo.bib",
                position: Position::new(1, 0),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(items.len() > 0, true);
    }

    #[test]
    fn test_inside_content() {
        let items = test_feature!(
            BibtexFieldNameCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo,\nbar = {baz}}"),],
                main_file: "foo.bib",
                position: Position::new(1, 7),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(items.len() == 0, true);
    }

    #[test]
    fn test_inside_entry_type() {
        let items = test_feature!(
            BibtexFieldNameCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo,}"),],
                main_file: "foo.bib",
                position: Position::new(0, 3),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(items.len() == 0, true);
    }
}
