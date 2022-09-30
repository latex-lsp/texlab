use lsp_types::CompletionParams;

use crate::features::cursor::CursorContext;

use super::types::{InternalCompletionItem, InternalCompletionItemData};

pub fn complete_user_environments<'a>(
    context: &'a CursorContext<CompletionParams>,
    items: &mut Vec<InternalCompletionItem<'a>>,
) -> Option<()> {
    let (name, range) = context.find_environment_name()?;

    for document in context.request.workspace.iter() {
        if let Some(data) = document.data().as_latex() {
            for name in data
                .extras
                .environment_names
                .iter()
                .filter(|n| n.as_str() != name)
                .cloned()
            {
                items.push(InternalCompletionItem::new(
                    range,
                    InternalCompletionItemData::UserEnvironment { name },
                ));
            }
        }
    }

    Some(())
}
