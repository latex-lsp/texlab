use lsp_types::CompletionParams;

use crate::features::cursor::CursorContext;

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_theorem_environments<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
) -> Option<()> {
    let (_, range) = context.find_environment_name()?;

    for document in context.request.workspace.iter() {
        if let Some(data) = document.data().as_latex() {
            for environment in &data.extras.theorem_environments {
                items.push(InternalCompletionItem::new(
                    range,
                    InternalCompletionItemData::UserEnvironment {
                        name: environment.name.clone(),
                    },
                ));
            }
        }
    }

    Some(())
}
