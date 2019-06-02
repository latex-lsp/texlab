use crate::completion::factory;
use crate::completion::latex::combinators::LatexCombinators;
use crate::feature::{FeatureProvider, FeatureRequest};
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams};
use std::borrow::Cow;

#[derive(Debug, PartialEq, Clone)]
pub struct LatexColorModelCompletionProvider {
    items: Vec<CompletionItem>,
}

impl LatexColorModelCompletionProvider {
    pub fn new() -> Self {
        let items = MODEL_NAMES
            .iter()
            .map(|name| Cow::from(*name))
            .map(factory::create_color_model)
            .collect();
        Self { items }
    }

    async fn execute_define_color<'a>(
        &'a self,
        request: &'a FeatureRequest<CompletionParams>,
    ) -> Vec<CompletionItem> {
        LatexCombinators::argument(&request, &COMMAND_NAMES[0..1], 1, async move |_| {
            self.items.clone()
        })
        .await
    }

    async fn execute_define_color_set<'a>(
        &'a self,
        request: &'a FeatureRequest<CompletionParams>,
    ) -> Vec<CompletionItem> {
        LatexCombinators::argument(&request, &COMMAND_NAMES[1..2], 0, async move |_| {
            self.items.clone()
        })
        .await
    }
}

impl FeatureProvider for LatexColorModelCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(
        &'a self,
        request: &'a FeatureRequest<CompletionParams>,
    ) -> Vec<CompletionItem> {
        let mut items = Vec::new();
        items.append(&mut self.execute_define_color(&request).await);
        items.append(&mut self.execute_define_color_set(&request).await);
        items
    }
}

const COMMAND_NAMES: &[&str] = &["\\definecolor", "\\definecolorset"];

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
