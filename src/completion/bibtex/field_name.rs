use crate::completion::factory;
use crate::data::bibtex_field::BIBTEX_FIELDS;
use crate::feature::{FeatureProvider, FeatureRequest};
use crate::syntax::bibtex::BibtexNode;
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams};

#[derive(Debug, PartialEq, Clone)]
pub struct BibtexFieldNameCompletionProvider {
    items: Vec<CompletionItem>,
}

impl BibtexFieldNameCompletionProvider {
    pub fn new() -> Self {
        let items = BIBTEX_FIELDS
            .iter()
            .map(|field| factory::create_field_name(field))
            .collect();
        Self { items }
    }
}

impl FeatureProvider for BibtexFieldNameCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<CompletionParams>,
    ) -> Vec<CompletionItem> {
        if let SyntaxTree::Bibtex(tree) = &request.document.tree {
            match tree.find(request.params.position).last() {
                Some(BibtexNode::Field(field)) => {
                    if field.name.range().contains(request.params.position) {
                        return self.items.clone();
                    }
                }
                Some(BibtexNode::Entry(entry)) => {
                    if !entry.is_comment() && !entry.ty.range().contains(request.params.position) {
                        if let Some(key) = &entry.key {
                            if !key.range().contains(request.params.position) {
                                return self.items.clone();
                            }
                        } else {
                            return self.items.clone();
                        }
                    }
                }
                _ => {}
            }
        }
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{test_feature, FeatureSpec};
    use lsp_types::Position;

    #[test]
    fn test_inside_first_field() {
        let items = test_feature(
            BibtexFieldNameCompletionProvider::new(),
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo,\nbar}")],
                main_file: "foo.bib",
                position: Position::new(1, 1),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(items.len() > 0, true);
    }

    #[test]
    fn test_inside_second_field() {
        let items = test_feature(
            BibtexFieldNameCompletionProvider::new(),
            FeatureSpec {
                files: vec![FeatureSpec::file(
                    "foo.bib",
                    "@article{foo, bar = {baz}, qux}",
                )],
                main_file: "foo.bib",
                position: Position::new(0, 27),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(items.len() > 0, true);
    }

    #[test]
    fn test_inside_entry() {
        let items = test_feature(
            BibtexFieldNameCompletionProvider::new(),
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo, \n}")],
                main_file: "foo.bib",
                position: Position::new(1, 0),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(items.len() > 0, true);
    }

    #[test]
    fn test_inside_content() {
        let items = test_feature(
            BibtexFieldNameCompletionProvider::new(),
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo,\nbar = {baz}}")],
                main_file: "foo.bib",
                position: Position::new(1, 7),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(items.len() == 0, true);
    }

    #[test]
    fn test_inside_entry_type() {
        let items = test_feature(
            BibtexFieldNameCompletionProvider::new(),
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo,}")],
                main_file: "foo.bib",
                position: Position::new(0, 3),
                ..FeatureSpec::default()
            },
        );
        assert_eq!(items.len() == 0, true);
    }
}
