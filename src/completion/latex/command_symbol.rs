use crate::completion::factory;
use crate::completion::factory::LatexComponentId;
use crate::completion::latex::combinators;
use crate::data::symbols::DATABASE;
use crate::feature::{FeatureProvider, FeatureRequest};
use futures_boxed::boxed;
use lsp_types::{CompletionItem, CompletionParams};
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LatexCommandSymbolCompletionProvider;

impl FeatureProvider for LatexCommandSymbolCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<Arc<CompletionItem>>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        combinators::command(&request, async move |_| {
            let mut items = Vec::new();
            let components = request
                .component_database
                .related_components(request.related_documents());

            for symbol in &DATABASE.commands {
                match &symbol.component {
                    Some(component) => {
                        if components.iter().any(|c| c.files.contains(&component)) {
                            let component = LatexComponentId::User(vec![Cow::from(component)]);
                            items.push(Arc::new(factory::create_command_symbol(
                                &symbol.command,
                                &component,
                                &symbol.image,
                            )));
                        }
                    }
                    None => {
                        items.push(Arc::new(factory::create_command_symbol(
                            &symbol.command,
                            &LatexComponentId::Kernel,
                            &symbol.image,
                        )));
                    }
                }
            }
            items
        })
        .await
    }
}
