use crate::completion::factory;
use crate::completion::factory::LatexComponentId;
use crate::completion::latex::combinators::LatexCombinators;
use crate::feature::FeatureRequest;
use lsp_types::{CompletionItem, CompletionParams};
use std::borrow::Cow;

pub struct LatexBeginCommandCompletionProvider;

impl LatexBeginCommandCompletionProvider {
    pub async fn execute(request: &FeatureRequest<CompletionParams>) -> Vec<CompletionItem> {
        LatexCombinators::command(request, async move |_| {
            let snippet = factory::create_snippet(
                Cow::from("begin"),
                &LatexComponentId::Kernel,
                Cow::from("begin{$1}\n\t$0\n\\end{$1}"),
            );
            vec![snippet]
        })
        .await
    }
}
