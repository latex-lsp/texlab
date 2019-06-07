use crate::completion::factory;
use crate::completion::factory::LatexComponentId;
use crate::completion::latex::combinators::LatexCombinators;
use crate::feature::{FeatureProvider, FeatureRequest};
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams};
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexBeginCommandCompletionProvider;

impl FeatureProvider for LatexBeginCommandCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<Arc<CompletionItem>>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        LatexCombinators::command(request, async move |_| {
            let snippet = factory::create_snippet(
                Cow::from("begin"),
                &LatexComponentId::Kernel,
                Cow::from("begin{$1}\n\t$0\n\\end{$1}"),
            );
            vec![Arc::new(snippet)]
        })
        .await
    }
}
