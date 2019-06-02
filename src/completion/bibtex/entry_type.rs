use crate::completion::factory;
use crate::data::bibtex_entry_type::BIBTEX_ENTRY_TYPES;
use crate::feature::{FeatureProvider, FeatureRequest};
use crate::syntax::bibtex::BibtexDeclaration;
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams};

#[derive(Debug, PartialEq, Clone)]
pub struct BibtexEntryTypeCompletionProvider {
    items: Vec<CompletionItem>,
}

impl BibtexEntryTypeCompletionProvider {
    pub fn new() -> Self {
        let items = BIBTEX_ENTRY_TYPES
            .iter()
            .map(factory::create_entry_type)
            .collect();
        Self { items }
    }
}

impl FeatureProvider for BibtexEntryTypeCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<CompletionParams>,
    ) -> Vec<CompletionItem> {
        if let SyntaxTree::Bibtex(tree) = &request.document.tree {
            for declaration in &tree.root.children {
                match declaration {
                    BibtexDeclaration::Preamble(preamble) => {
                        if preamble.ty.range().contains(request.params.position) {
                            return self.items.clone();
                        }
                    }
                    BibtexDeclaration::String(string) => {
                        if string.ty.range().contains(request.params.position) {
                            return self.items.clone();
                        }
                    }
                    BibtexDeclaration::Entry(entry) => {
                        if entry.ty.range().contains(request.params.position) {
                            return self.items.clone();
                        }
                    }
                    BibtexDeclaration::Comment(_) => {}
                }
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
    fn test_after_at_sign() {
        let items = test_feature(
            BibtexEntryTypeCompletionProvider::new(),
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@")],
                main_file: "foo.bib",
                position: Position::new(0, 1),
                ..FeatureSpec::default()
            },
        );
        assert!(!items.is_empty());
    }

    #[test]
    fn test_inside_entry() {
        let items = test_feature(
            BibtexEntryTypeCompletionProvider::new(),
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo,}")],
                main_file: "foo.bib",
                position: Position::new(0, 11),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }

    #[test]
    fn test_inside_comments() {
        let items = test_feature(
            BibtexEntryTypeCompletionProvider::new(),
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "foo")],
                main_file: "foo.bib",
                position: Position::new(0, 2),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }

    #[test]
    fn test_latex() {
        let items = test_feature(
            BibtexEntryTypeCompletionProvider::new(),
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "@")],
                main_file: "foo.tex",
                position: Position::new(0, 1),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }
}
