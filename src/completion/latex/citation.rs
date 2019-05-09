use crate::completion::factory;
use crate::completion::latex::combinators::LatexCombinators;
use crate::feature::FeatureRequest;
use crate::syntax::bibtex::BibtexDeclaration;
use crate::syntax::latex::CITATION_COMMANDS;
use crate::workspace::SyntaxTree;
use lsp_types::{CompletionItem, CompletionParams};

pub struct LatexCitationCompletionProvider;

impl LatexCitationCompletionProvider {
    pub async fn execute(request: &FeatureRequest<CompletionParams>) -> Vec<CompletionItem> {
        await!(LatexCombinators::argument(
            request,
            CITATION_COMMANDS,
            0,
            async move |_| {
                let mut items = Vec::new();
                for document in &request.related_documents {
                    if let SyntaxTree::Bibtex(tree) = &document.tree {
                        for declaration in &tree.root.children {
                            if let BibtexDeclaration::Entry(entry) = declaration {
                                if let Some(key) = &entry.key {
                                    items.push(factory::create_citation(
                                        document.uri.clone(),
                                        key.text().to_owned(),
                                    ));
                                }
                            }
                        }
                    }
                }
                items
            }
        ))
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
    fn test_inside_cite() {
        let items = test_feature!(
            LatexCitationCompletionProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\addbibresource{bar.bib}\n\\cite{}"),
                    FeatureSpec::file("bar.bib", "@article{foo,}"),
                    FeatureSpec::file("baz.bib", "@article{bar,}")
                ],
                main_file: "foo.tex",
                position: Position::new(1, 6),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(items.len(), 1);
        assert_eq!("foo", items[0].label);
    }

    #[test]
    fn test_outside_cite() {
        let items = test_feature!(
            LatexCitationCompletionProvider,
            FeatureSpec {
                files: vec![
                    FeatureSpec::file("foo.tex", "\\addbibresource{bar.bib}\n\\cite{}"),
                    FeatureSpec::file("bar.bib", "@article{foo,}"),
                    FeatureSpec::file("baz.bib", "@article{bar,}")
                ],
                main_file: "foo.tex",
                position: Position::new(1, 7),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(items, Vec::new());
    }
}
