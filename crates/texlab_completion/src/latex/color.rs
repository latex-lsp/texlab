use super::combinators::{self, Parameter};
use crate::factory;
use texlab_workspace::*;
use futures_boxed::boxed;
use texlab_protocol::*;
use texlab_syntax::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexColorCompletionProvider;

impl FeatureProvider for LatexColorCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let parameters = LANGUAGE_DATA
            .color_commands
            .iter()
            .map(|cmd| Parameter::new(&cmd.name, cmd.index));

        combinators::argument(request, parameters, |context| {
            async move {
                let mut items = Vec::new();
                for name in &LANGUAGE_DATA.colors {
                    let text_edit = TextEdit::new(context.range, name.into());
                    let item = factory::color(request, name, text_edit);
                    items.push(item);
                }
                items
            }
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(!items.is_empty());
        assert_eq!(
            items[0].text_edit.as_ref().map(|edit| edit.range),
            Some(Range::new_simple(0, 7, 0, 7))
        );
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
