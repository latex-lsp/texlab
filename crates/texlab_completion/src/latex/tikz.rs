use super::combinators::{self, Parameter};
use crate::factory;
use futures_boxed::boxed;
use texlab_protocol::*;
use texlab_syntax::LANGUAGE_DATA;
use texlab_workspace::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexPgfLibraryCompletionProvider;

impl FeatureProvider for LatexPgfLibraryCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let parameter = Parameter::new("\\usepgflibrary", 0);
        combinators::argument(request, std::iter::once(parameter), |context| {
            async move {
                let mut items = Vec::new();
                for name in &LANGUAGE_DATA.pgf_libraries {
                    let text_edit = TextEdit::new(context.range, name.into());
                    let item = factory::pgf_library(request, name, text_edit);
                    items.push(item);
                }
                items
            }
        })
        .await
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexTikzLibraryCompletionProvider;

impl FeatureProvider for LatexTikzLibraryCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let parameter = Parameter::new("\\usetikzlibrary", 0);
        combinators::argument(request, std::iter::once(parameter), |context| {
            async move {
                let mut items = Vec::new();
                for name in &LANGUAGE_DATA.tikz_libraries {
                    let text_edit = TextEdit::new(context.range, name.into());
                    let item = factory::tikz_library(request, name, text_edit);
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
    fn test_pgf_library() {
        let items = test_feature(
            LatexPgfLibraryCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\usepgflibrary{}")],
                main_file: "foo.tex",
                position: Position::new(0, 15),
                ..FeatureSpec::default()
            },
        );
        assert!(!items.is_empty());
    }

    #[test]
    fn test_tikz_library() {
        let items = test_feature(
            LatexTikzLibraryCompletionProvider,
            FeatureSpec {
                files: vec![FeatureSpec::file("foo.tex", "\\usetikzlibrary{}")],
                main_file: "foo.tex",
                position: Position::new(0, 16),
                ..FeatureSpec::default()
            },
        );
        assert!(!items.is_empty());
    }
}
