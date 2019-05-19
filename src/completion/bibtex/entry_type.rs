use crate::completion::factory;
use crate::data::bibtex_entry_type::BIBTEX_ENTRY_TYPES;
use crate::feature::FeatureRequest;
use crate::syntax::bibtex::BibtexDeclaration;
use crate::syntax::text::SyntaxNode;
use crate::syntax::SyntaxTree;
use lsp_types::{CompletionItem, CompletionParams};

pub struct BibtexEntryTypeCompletionProvider;

impl BibtexEntryTypeCompletionProvider {
    pub async fn execute(request: &FeatureRequest<CompletionParams>) -> Vec<CompletionItem> {
        if let SyntaxTree::Bibtex(tree) = &request.document.tree {
            for declaration in &tree.root.children {
                match declaration {
                    BibtexDeclaration::Preamble(preamble) => {
                        if preamble.ty.range().contains(request.params.position) {
                            return Self::generate_items();
                        }
                    }
                    BibtexDeclaration::String(string) => {
                        if string.ty.range().contains(request.params.position) {
                            return Self::generate_items();
                        }
                    }
                    BibtexDeclaration::Entry(entry) => {
                        if entry.ty.range().contains(request.params.position) {
                            return Self::generate_items();
                        }
                    }
                    BibtexDeclaration::Comment(_) => {}
                }
            }
        }
        Vec::new()
    }

    fn generate_items() -> Vec<CompletionItem> {
        BIBTEX_ENTRY_TYPES
            .iter()
            .map(factory::create_entry_type)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::completion::LatexComponentDatabase;
    use crate::feature::FeatureSpec;
    use crate::test_feature;
    use lsp_types::Position;

    #[test]
    fn test_after_at_sign() {
        let items = test_feature!(
            BibtexEntryTypeCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@"),],
                main_file: "foo.bib",
                position: Position::new(0, 1),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(items.len() > 0, true);
    }

    #[test]
    fn test_inside_entry() {
        let items = test_feature!(
            BibtexEntryTypeCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "@article{foo,}"),],
                main_file: "foo.bib",
                position: Position::new(0, 11),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(items.len() == 0, true);
    }

    #[test]
    fn test_inside_comments() {
        let items = test_feature!(
            BibtexEntryTypeCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.bib", "foo"),],
                main_file: "foo.bib",
                position: Position::new(0, 2),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(items.len() == 0, true);
    }

    #[test]
    fn test_latex() {
        let items = test_feature!(
            BibtexEntryTypeCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "@"),],
                main_file: "foo.tex",
                position: Position::new(0, 1),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(items.len() == 0, true);
    }
}
