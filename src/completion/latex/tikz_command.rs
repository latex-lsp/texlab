use crate::completion::factory;
use crate::completion::factory::LatexComponentId;
use crate::completion::latex::combinators;
use crate::data::language::language_data;
use crate::feature::{FeatureProvider, FeatureRequest};
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams};
use std::borrow::Cow;
use std::sync::Arc;

pub struct LatexTikzCommandCompletionProvider {
    items: Vec<Arc<CompletionItem>>,
}

impl LatexTikzCommandCompletionProvider {
    pub fn new() -> Self {
        let id = LatexComponentId::User(vec![Cow::from("tikz.sty")]);
        let items = language_data()
            .tikz_commands
            .iter()
            .map(Cow::from)
            .map(|name| factory::create_command(name, &id))
            .map(Arc::new)
            .collect();
        Self { items }
    }
}

impl FeatureProvider for LatexTikzCommandCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<Arc<CompletionItem>>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        combinators::command(request, async move |_| {
            if request
                .component_database
                .related_components(request.related_documents())
                .iter()
                .any(|component| component.file_names.iter().any(|file| file == "tikz.sty"))
            {
                self.items.clone()
            } else {
                Vec::new()
            }
        })
        .await
    }
}
