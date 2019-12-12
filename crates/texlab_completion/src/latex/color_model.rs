use super::combinators::{self, Parameter};
use crate::factory;
use texlab_workspace::*;
use futures_boxed::boxed;
use texlab_protocol::*;
use texlab_syntax::LANGUAGE_DATA;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexColorModelCompletionProvider;

impl FeatureProvider for LatexColorModelCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let parameters = LANGUAGE_DATA
            .color_model_commands
            .iter()
            .map(|cmd| Parameter::new(&cmd.name, cmd.index));

        combinators::argument(&request, parameters, |context| {
            async move {
                let mut items = Vec::new();
                for name in MODEL_NAMES {
                    let text_edit = TextEdit::new(context.range, (*name).into());
                    let item = factory::color_model(request, name, text_edit);
                    items.push(item);
                }
                items
            }
        })
        .await
    }
}

const MODEL_NAMES: &[&str] = &["gray", "rgb", "RGB", "HTML", "cmyk"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inside_define_color() {
        let items = test_feature(
            LatexColorModelCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\definecolor{name}{}")],
                main_file: "foo.tex",
                position: Position::new(0, 19),
                ..FeatureSpec::default()
            },
        );
        assert!(!items.is_empty());
        assert_eq!(
            items[0].text_edit.as_ref().map(|edit| edit.range),
            Some(Range::new_simple(0, 19, 0, 19))
        );
    }

    #[test]
    fn test_outside_define_color() {
        let items = test_feature(
            LatexColorModelCompletionProvider,
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
            LatexColorModelCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\definecolorset{}")],
                main_file: "foo.tex",
                position: Position::new(0, 16),
                ..FeatureSpec::default()
            },
        );
        assert!(!items.is_empty());
        assert_eq!(
            items[0].text_edit.as_ref().map(|edit| edit.range),
            Some(Range::new_simple(0, 16, 0, 16))
        );
    }
}
