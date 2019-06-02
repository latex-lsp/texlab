use crate::completion::factory;
use crate::completion::latex::combinators::LatexCombinators;
use crate::data::symbols::DATABASE;
use crate::feature::{FeatureProvider, FeatureRequest};
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexArgumentSymbolCompletionProvider;

impl FeatureProvider for LatexArgumentSymbolCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<CompletionParams>,
    ) -> Vec<CompletionItem> {
        let mut items = Vec::new();
        for group in &DATABASE.arguments {
            let command = format!("\\{}", group.command);
            let command_names = &[command.as_ref()];
            items.append(
                &mut LatexCombinators::argument(
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
                    },
                )
                .await,
            );
        }
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{test_feature, FeatureSpec};
    use lsp_types::Position;

    #[test]
    fn test_inside_mathbb() {
        let items = test_feature(
            LatexArgumentSymbolCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\mathbb{}")],
                main_file: "foo.tex",
                position: Position::new(0, 8),
                ..FeatureSpec::default()
            },
        );
        assert!(!items.is_empty());
    }

    #[test]
    fn test_outside_mathbb() {
        let items = test_feature(
            LatexArgumentSymbolCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\mathbb{}")],
                main_file: "foo.tex",
                position: Position::new(0, 9),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }
}
