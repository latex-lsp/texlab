use super::combinators;
use crate::completion::factory::{self, LatexComponentId};
use crate::workspace::*;
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexBeginCommandCompletionProvider;

impl FeatureProvider for LatexBeginCommandCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        combinators::command(request, |_| {
            async move {
                let snippet = factory::command_snippet(
                    request,
                    "begin",
                    None,
                    "begin{$1}\n\t$0\n\\end{$1}",
                    &LatexComponentId::kernel(),
                );
                vec![snippet]
            }
        })
        .await
    }
}
