use crate::completion::factory;
use crate::completion::latex::combinators::{self, ArgumentLocation};
use crate::feature::{FeatureProvider, FeatureRequest};
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams};
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Debug, PartialEq, Clone)]
pub struct LatexColorModelCompletionProvider {
    items: Vec<Arc<CompletionItem>>,
}

impl LatexColorModelCompletionProvider {
    pub fn new() -> Self {
        let items = MODEL_NAMES
            .iter()
            .map(|name| Cow::from(*name))
            .map(factory::create_color_model)
            .map(Arc::new)
            .collect();
        Self { items }
    }
}

impl FeatureProvider for LatexColorModelCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<Arc<CompletionItem>>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        combinators::argument(
            &request,
            LOCATIONS.iter().map(|location| *location),
            async move |_| self.items.clone(),
        )
        .await
    }
}

const LOCATIONS: &[ArgumentLocation] = &[
    ArgumentLocation {
        name: "\\definecolor",
        index: 1,
    },
    ArgumentLocation {
        name: "\\definecolorset",
        index: 0,
    },
];

const MODEL_NAMES: &[&str] = &["gray", "rgb", "RGB", "HTML", "cmyk"];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::{test_feature, FeatureSpec};
    use lsp_types::Position;

    #[test]
    fn test_inside_define_color() {
        let items = test_feature(
            LatexColorModelCompletionProvider::new(),
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\definecolor{name}{}")],
                main_file: "foo.tex",
                position: Position::new(0, 19),
                ..FeatureSpec::default()
            },
        );
        assert!(!items.is_empty());
    }

    #[test]
    fn test_outside_define_color() {
        let items = test_feature(
            LatexColorModelCompletionProvider::new(),
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\definecolor{name}{}")],
                main_file: "foo.tex",
                position: Position::new(0, 18),
                ..FeatureSpec::default()
            },
        );
        assert!(items.is_empty());
    }

    #[test]
    fn tet_inside_define_color_set() {
        let items = test_feature(
            LatexColorModelCompletionProvider::new(),
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\definecolorset{}")],
                main_file: "foo.tex",
                position: Position::new(0, 16),
                ..FeatureSpec::default()
            },
        );
        assert!(!items.is_empty());
    }
}
