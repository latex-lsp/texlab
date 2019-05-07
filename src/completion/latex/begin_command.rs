use crate::completion::factory;
use crate::completion::factory::LatexComponentId;
use crate::completion::latex::combinators::LatexCombinators;
use crate::feature::FeatureRequest;
use lsp_types::{CompletionItem, CompletionParams};

pub struct LatexBeginCommandCompletionProvider;

impl LatexBeginCommandCompletionProvider {
    pub async fn execute(request: &FeatureRequest<CompletionParams>) -> Vec<CompletionItem> {
        await!(LatexCombinators::command(request, async move |_| {
            let snippet = factory::create_snippet(
                "begin".to_owned(),
                &LatexComponentId::Kernel,
                "begin{$1}\n\t$0\n\\end{$1}".to_owned(),
            );
            vec![snippet]
        }))
    }
}
