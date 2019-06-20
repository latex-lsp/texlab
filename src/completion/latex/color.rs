use crate::completion::factory;
use crate::completion::latex::combinators::{self, ArgumentLocation};
use crate::feature::{FeatureProvider, FeatureRequest};
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams};
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Debug, PartialEq, Clone)]
pub struct LatexColorCompletionProvider;

impl FeatureProvider for LatexColorCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<Arc<CompletionItem>>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let locations = request
            .latex_language_options
            .color_commands()
            .map(|cmd| ArgumentLocation::new(&cmd.name, cmd.index));

        combinators::argument(request, locations, async move |_| {
            request.latex_language_options.colors()
                .map(|name| factory::create_color(Cow::from(name.to_owned())))
                .map(Arc::new)
                .collect()
        }).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{test_feature, FeatureSpec};
    use lsp_types::Position;

    #[test]
    fn test_inside_color() {
        let items = test_feature(
            LatexColorCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\color{}")],
                main_file: "foo.tex",
                position: Position::new(0, 7),
                ..FeatureSpec::default()
            },
        );
        assert!(items.iter().any(|item| item.label == "black"));
    }

    #[test]
    fn test_outside_color() {
        let items = test_feature(
            LatexColorCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\color{}")],
                main_file: "foo.tex",
                position: Position::new(0, 8),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }
}
