use super::combinators::{self, Parameter};
use crate::completion::factory;
use crate::syntax::LatexGlossaryEntryKind::*;
use crate::syntax::*;
use crate::workspace::*;
use futures_boxed::boxed;
use texlab_protocol::{CompletionItem, CompletionParams, TextEdit};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LatexGlossaryCompletionProvider;

impl FeatureProvider for LatexGlossaryCompletionProvider {
    type Params = CompletionParams;
    type Output = Vec<CompletionItem>;

    #[boxed]
    async fn execute<'a>(&'a self, request: &'a FeatureRequest<Self::Params>) -> Self::Output {
        let parameters = LANGUAGE_DATA
            .glossary_entry_reference_commands
            .iter()
            .map(|cmd| Parameter::new(&cmd.name, cmd.index));

        combinators::argument(request, parameters, |context| {
            async move {
                let cmd_kind = LANGUAGE_DATA
                    .glossary_entry_reference_commands
                    .iter()
                    .find(|cmd| cmd.name == context.parameter.name)
                    .unwrap()
                    .kind;

                let mut items = Vec::new();
                for document in request.related_documents() {
                    if let SyntaxTree::Latex(tree) = &document.tree {
                        for entry in &tree.glossary.entries {
                            match (cmd_kind, entry.kind) {
                                (Acronym, Acronym) | (General, General) | (General, Acronym) => {
                                    let label = entry.label().text().to_owned();
                                    let text_edit = TextEdit::new(context.range, label.clone());
                                    let item = factory::glossary_entry(request, label, text_edit);
                                    items.push(item);
                                }
                                (Acronym, General) => {}
                            }
                        }
                    }
                }
                items
            }
        })
        .await
    }
}
