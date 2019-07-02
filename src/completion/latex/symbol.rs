use crate::completion::factory::{self, LatexComponentId};
use crate::completion::latex::combinators::{self, Parameter};
use crate::data::symbols::DATABASE;
use crate::feature::{FeatureProvider, FeatureRequest};
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams, TextEdit};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexCommandSymbolCompletionProvider;

impl FeatureProvider for LatexCommandSymbolCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        combinators::command(&request, async move |command| {
            let edit_range = command.short_name_range();
            let mut items = Vec::new();
            let components = request
                .component_database
                .related_components(request.related_documents());

            for symbol in &DATABASE.commands {
                let component = match &symbol.component {
                    Some(component) => {
                        if components
                            .iter()
                            .any(|c| c.file_names.contains(&component.into()))
                        {
                            LatexComponentId::Component(vec![component])
                        } else {
                            continue;
                        }
                    }
                    None => LatexComponentId::Kernel,
                };

                let text_edit = TextEdit::new(edit_range, (&symbol.command).into());
                let item = factory::command_symbol(
                    request,
                    &symbol.command,
                    text_edit,
                    &component,
                    &symbol.image,
                );
                items.push(item);
            }
            items
        })
        .await
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexArgumentSymbolCompletionProvider;

impl FeatureProvider for LatexArgumentSymbolCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let mut items = Vec::new();
        for group in &DATABASE.arguments {
            let command = format!("\\{}", group.command);
            let parameter = Parameter::new(command.as_ref(), group.index);
            items.append(
                &mut combinators::argument(
                    &request,
                    std::iter::once(parameter),
                    async move |context| {
                        let mut items = Vec::new();
                        for symbol in &group.arguments {
                            let text_edit =
                                TextEdit::new(context.range, (&symbol.argument).into());
                            let item = factory::argument_symbol(
                                request,
                                &symbol.argument,
                                text_edit,
                                &symbol.image,
                            );
                            items.push(item);
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
    use lsp_types::{Position, Range};

    #[test]
    fn test_inside_mathbb_empty() {
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
        assert_eq!(
            items[0].text_edit.as_ref().map(|edit| edit.range),
            Some(Range::new_simple(0, 8, 0, 8))
        );
    }

    #[test]
    fn test_inside_mathbb_non_empty() {
        let items = test_feature(
            LatexArgumentSymbolCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\mathbb{foo}")],
                main_file: "foo.tex",
                position: Position::new(0, 8),
                ..FeatureSpec::default()
            },
        );
        assert!(!items.is_empty());
        assert_eq!(
            items[0].text_edit.as_ref().map(|edit| edit.range),
            Some(Range::new_simple(0, 8, 0, 11))
        );
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
