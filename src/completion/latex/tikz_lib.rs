use super::combinators::{self, Parameter};
use crate::{
    completion::types::{Item, ItemData},
    feature::FeatureRequest,
    protocol::CompletionParams,
    syntax::LANGUAGE_DATA,
};
use std::iter;

pub async fn complete_latex_pgf_libraries<'a>(
    req: &'a FeatureRequest<CompletionParams>,
    items: &mut Vec<Item<'a>>,
) {
    let param = Parameter {
        name: "usepgflibrary",
        index: 0,
    };
    combinators::argument(req, iter::once(param), |ctx| async move {
        for name in &LANGUAGE_DATA.pgf_libraries {
            let item = Item::new(ctx.range, ItemData::PgfLibrary { name });
            items.push(item);
        }
    })
    .await;
}

pub async fn complete_latex_tikz_libraries<'a>(
    req: &'a FeatureRequest<CompletionParams>,
    items: &mut Vec<Item<'a>>,
) {
    let param = Parameter {
        name: "usetikzlibrary",
        index: 0,
    };
    combinators::argument(req, iter::once(param), |ctx| async move {
        for name in &LANGUAGE_DATA.tikz_libraries {
            let item = Item::new(ctx.range, ItemData::TikzLibrary { name });
            items.push(item);
        }
    })
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::FeatureTester;

    #[tokio::test]
    async fn empty_latex_document_pgf() {
        let req = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_pgf_libraries(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document_pgf() {
        let req = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_pgf_libraries(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_latex_document_tikz() {
        let req = FeatureTester::new()
            .file("main.tex", "")
            .main("main.tex")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_tikz_libraries(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn empty_bibtex_document_tikz() {
        let req = FeatureTester::new()
            .file("main.bib", "")
            .main("main.bib")
            .position(0, 0)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_tikz_libraries(&req, &mut actual_items).await;

        assert!(actual_items.is_empty());
    }

    #[tokio::test]
    async fn pgf_library() {
        let req = FeatureTester::new()
            .file("main.tex", r#"\usepgflibrary{}"#)
            .main("main.tex")
            .position(0, 15)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_pgf_libraries(&req, &mut actual_items).await;

        assert!(!actual_items.is_empty());
    }

    #[tokio::test]
    async fn tikz_library() {
        let req = FeatureTester::new()
            .file("main.tex", r#"\usetikzlibrary{}"#)
            .main("main.tex")
            .position(0, 16)
            .test_completion_request()
            .await;
        let mut actual_items = Vec::new();

        complete_latex_tikz_libraries(&req, &mut actual_items).await;

        assert!(!actual_items.is_empty());
    }
}
