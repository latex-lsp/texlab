use crate::completion::factory;
use crate::completion::latex::combinators::LatexCombinators;
use crate::feature::FeatureRequest;
use lsp_types::{CompletionItem, CompletionParams};
use std::borrow::Cow;

pub struct LatexColorModelCompletionProvider;

impl LatexColorModelCompletionProvider {
    pub async fn execute(request: &FeatureRequest<CompletionParams>) -> Vec<CompletionItem> {
        let mut items = Vec::new();
        items.append(&mut await!(Self::execute_define_color(&request)));
        items.append(&mut await!(Self::execute_define_color_set(&request)));
        items
    }

    async fn execute_define_color(
        request: &FeatureRequest<CompletionParams>,
    ) -> Vec<CompletionItem> {
        await!(LatexCombinators::argument(
            &request,
            &COMMAND_NAMES[0..1],
            1,
            async move |_| { Self::generate_items() }
        ))
    }

    async fn execute_define_color_set(
        request: &FeatureRequest<CompletionParams>,
    ) -> Vec<CompletionItem> {
        await!(LatexCombinators::argument(
            &request,
            &COMMAND_NAMES[1..2],
            0,
            async move |_| { Self::generate_items() }
        ))
    }

    fn generate_items() -> Vec<CompletionItem> {
        MODEL_NAMES
            .iter()
            .map(|name| factory::create_color_model(Cow::from(*name)))
            .collect()
    }
}

const COMMAND_NAMES: &[&str] = &["\\definecolor", "\\definecolorset"];

const MODEL_NAMES: &[&str] = &["gray", "rgb", "RGB", "HTML", "cmyk"];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::completion::LatexComponentDatabase;
    use crate::feature::FeatureSpec;
    use crate::test_feature;
    use lsp_types::Position;

    #[test]
    fn test_inside_define_color() {
        let items = test_feature!(
            LatexColorModelCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\definecolor{name}{}")],
                main_file: "foo.tex",
                position: Position::new(0, 19),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(items, LatexColorModelCompletionProvider::generate_items());
    }

    #[test]
    fn test_outside_define_color() {
        let items = test_feature!(
            LatexColorModelCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\definecolor{name}{}")],
                main_file: "foo.tex",
                position: Position::new(0, 18),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(items, Vec::new());
    }

    #[test]
    fn tet_inside_define_color_set() {
        let items = test_feature!(
            LatexColorModelCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\definecolorset{}")],
                main_file: "foo.tex",
                position: Position::new(0, 16),
                new_name: "",
                component_database: LatexComponentDatabase::default(),
            }
        );
        assert_eq!(items, LatexColorModelCompletionProvider::generate_items());
    }
}
