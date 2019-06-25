use crate::completion::factory;
use crate::completion::latex::combinators::{self, ArgumentLocation};
use crate::data::language::language_data;
use crate::feature::{FeatureProvider, FeatureRequest};
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams};
use std::borrow::Cow;
use std::sync::Arc;

pub struct LatexTikzLibraryCompletionProvider {
    items: Vec<Arc<CompletionItem>>,
}

impl LatexTikzLibraryCompletionProvider {
    pub fn new() -> Self {
        let items = language_data()
            .tikz_libraries
            .iter()
            .map(Cow::from)
            .map(factory::create_tikz_library)
            .map(Arc::new)
            .collect();
        Self { items }
    }
}

impl FeatureProvider for LatexTikzLibraryCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<Arc<CompletionItem>>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let locations = COMMANDS.iter().map(|cmd| ArgumentLocation::new(cmd, 0));
        combinators::argument(request, locations, async move |_| self.items.clone()).await
    }
}

const COMMANDS: &[&str] = &["\\usetikzlibrary"];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{test_feature, FeatureSpec};
    use lsp_types::Position;

    #[test]
    fn test() {
        let items = test_feature(
            LatexTikzLibraryCompletionProvider::new(),
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\usetikzlibrary{}")],
                main_file: "foo.tex",
                position: Position::new(0, 16),
                ..FeatureSpec::default()
            },
        );
        assert!(items.iter().any(|item| item.label == "arrows"));
    }
}
