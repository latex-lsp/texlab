use crate::completion::factory;
use crate::completion::latex::combinators::LatexCombinators;
use crate::feature::FeatureRequest;
use lsp_types::{CompletionItem, CompletionParams};

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
            .map(|name| factory::create_color_model((*name).to_owned()))
            .collect()
    }
}

const COMMAND_NAMES: &'static [&'static str] = &["\\definecolor", "\\definecolorset"];

const MODEL_NAMES: &'static [&'static str] = &["gray", "rgb", "RGB", "HTML", "cmyk"];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;
    use crate::workspace::WorkspaceBuilder;
    use futures::executor::block_on;

    #[test]
    fn test_inside_define_color() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "\\definecolor{name}{}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 19, "").into();

        let items = block_on(LatexColorModelCompletionProvider::execute(&request));

        assert_eq!(LatexColorModelCompletionProvider::generate_items(), items);
    }

    #[test]
    fn test_outside_define_color() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "\\definecolor{name}{}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 18, "").into();

        let items = block_on(LatexColorModelCompletionProvider::execute(&request));

        assert_eq!(items, Vec::new());
    }

    #[test]
    fn test_inside_define_color_set() {
        let mut builder = WorkspaceBuilder::new();
        let uri = builder.document("foo.tex", "\\definecolorset{}");
        let request = FeatureTester::new(builder.workspace, uri, 0, 16, "").into();

        let items = block_on(LatexColorModelCompletionProvider::execute(&request));

        assert_eq!(LatexColorModelCompletionProvider::generate_items(), items);
    }
}
