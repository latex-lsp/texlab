use super::combinators::{self, Parameter};
use crate::{
    completion::factory,
    feature::{FeatureProvider, FeatureRequest},
    protocol::{CompletionItem, CompletionParams, TextEdit},
    syntax::LANGUAGE_DATA,
};
use async_trait::async_trait;
use std::iter;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexPgfLibraryCompletionProvider;

#[async_trait]
impl FeatureProvider for LatexPgfLibraryCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let param = Parameter {
            name: "\\usepgflibrary",
            index: 0,
        };
        combinators::argument(req, iter::once(param), |ctx| async move {
            let mut items = Vec::new();
            for name in &LANGUAGE_DATA.pgf_libraries {
                let text_edit = TextEdit::new(ctx.range, name.into());
                let item = factory::pgf_library(req, name, text_edit);
                items.push(item);
            }
            items
        })
        .await
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct LatexTikzLibraryCompletionProvider;

#[async_trait]
impl FeatureProvider for LatexTikzLibraryCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    async fn execute<'a>(&'a self, req: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let param = Parameter {
            name: "\\usetikzlibrary",
            index: 0,
        };
        combinators::argument(req, iter::once(param), |ctx| async move {
            let mut items = Vec::new();
            for name in &LANGUAGE_DATA.tikz_libraries {
                let text_edit = TextEdit::new(ctx.range, name.into());
                let item = factory::tikz_library(req, name, text_edit);
                items.push(item);
            }
            items
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;

    #[tokio::test]
    async fn empty_latex_document_pgf() {
        let actual_items = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion(LatexPgfLibraryCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document_pgf() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion(LatexPgfLibraryCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_latex_document_tikz() {
        let actual_items = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion(LatexTikzLibraryCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document_tikz() {
        let actual_items = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion(LatexTikzLibraryCompletionProvider)
            .await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn pgf_library() {
        let actual_items = FeatureTester::new()
            .file("main.tex", r#"\usepgflibrary{}"#)
            .main("main.tex")
            .position(0, 15)
            .test_completion(LatexPgfLibraryCompletionProvider)
            .await;

        assert!(!actual_items.is_empty());
    }

    #[tokio::test]
    async fn tikz_library() {
        let actual_items = FeatureTester::new()
            .file("main.tex", r#"\usetikzlibrary{}"#)
            .main("main.tex")
            .position(0, 16)
            .test_completion(LatexTikzLibraryCompletionProvider)
            .await;

        assert!(!actual_items.is_empty());
    }
}
