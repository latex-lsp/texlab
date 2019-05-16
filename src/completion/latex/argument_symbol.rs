use crate::completion::factory;
use crate::completion::latex::combinators::LatexCombinators;
use crate::data::symbols::DATABASE;
use crate::feature::FeatureRequest;
use lsp_types::{CompletionItem, CompletionParams};

pub struct LatexArgumentSymbolCompletionProvider;

impl LatexArgumentSymbolCompletionProvider {
    pub async fn execute(request: &FeatureRequest<CompletionParams>) -> Vec<CompletionItem> {
        let mut items = Vec::new();
        for group in &DATABASE.arguments {
            let command = format!("\\{}", group.command);
            let command_names = &[command.as_ref()];
            items.append(&mut await!(LatexCombinators::argument(
                &request,
                command_names,
                group.index,
                async move |_| {
                    let mut items = Vec::new();
                    for symbol in &group.arguments {
                        items.push(factory::create_argument_symbol(
                            &symbol.argument,
                            &symbol.image,
                        ));
                    }
                    items
                }
            )));
        }
        items
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
    fn test_inside_mathbb() {
        let items = test_feature!(
            LatexArgumentSymbolCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\mathbb{}")],
                main_file: "foo.tex",
                position: Position::new(0, 8),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(true, items.len() > 0);
    }

    #[test]
    fn test_outside_mathbb() {
        let items = test_feature!(
            LatexArgumentSymbolCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\mathbb{}")],
                main_file: "foo.tex",
                position: Position::new(0, 9),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(true, items.len() == 0);
    }
}
